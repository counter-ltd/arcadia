//! Voice pool + mixer. Owns the active-voice list and the master gain. Backends
//! call [`mix_into`] from their audio callback to fill an output buffer.
//!
//! The list is guarded by a `Mutex`. Backends should keep the locked window short:
//! drain pending triggers from a separate SPSC queue if needed. v1 keeps it simple
//! — Python triggers acquire the mutex directly, which is fine at typing rates.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock};

use super::graph::{CompiledGraph, VoiceInstance, VoiceSpec};
use super::sample::{get_sample, SampleVoice};

pub const OWNER_DELIMITER: char = '|';
const MAX_ACTIVE_VOICES: usize = 64;


// ─── Registered voices ───────────────────────────────────────────────────────

struct RegisteredVoice {
    owner: String,
    graph: Arc<CompiledGraph>,
}

fn voices() -> &'static RwLock<BTreeMap<String, RegisteredVoice>> {
    static V: OnceLock<RwLock<BTreeMap<String, RegisteredVoice>>> = OnceLock::new();
    V.get_or_init(|| RwLock::new(BTreeMap::new()))
}

fn next_voice_id() -> String {
    static N: AtomicU64 = AtomicU64::new(1);
    let n = N.fetch_add(1, Ordering::Relaxed);
    format!("voice-{n}")
}

pub fn register_voice(owner: &str, spec: VoiceSpec) -> Result<String, String> {
    let sample_rate = super::sample_rate();
    let graph = CompiledGraph::compile(&spec, sample_rate)?;
    let id = next_voice_id();
    let entry = RegisteredVoice {
        owner: owner.to_string(),
        graph: Arc::new(graph),
    };
    if let Ok(mut v) = voices().write() {
        v.insert(id.clone(), entry);
    }
    Ok(id)
}

pub fn unregister_voice(owner: &str, id: &str) -> Result<(), String> {
    let mut v = voices().write().map_err(|e| e.to_string())?;
    match v.get(id) {
        Some(e) if e.owner == owner => {
            v.remove(id);
            Ok(())
        }
        Some(_) => Err("voice owned by different extension".into()),
        None => Err("voice not found".into()),
    }
}

pub fn unregister_owner(owner: &str) {
    if let Ok(mut v) = voices().write() {
        let to_remove: Vec<String> = v
            .iter()
            .filter(|(_, e)| e.owner == owner)
            .map(|(k, _)| k.clone())
            .collect();
        for k in to_remove {
            v.remove(&k);
        }
    }
}

// ─── Active voices (mixer state) ─────────────────────────────────────────────

enum ActiveVoice {
    Synth { owner: String, inst: VoiceInstance },
    Sample { owner: String, inst: SampleVoice },
}

impl ActiveVoice {
    fn owner(&self) -> &str {
        match self {
            ActiveVoice::Synth { owner, .. } | ActiveVoice::Sample { owner, .. } => owner,
        }
    }
    fn finished(&self) -> bool {
        match self {
            ActiveVoice::Synth { inst, .. } => inst.finished(),
            ActiveVoice::Sample { inst, .. } => inst.finished(),
        }
    }
    fn tick(&mut self) -> f32 {
        match self {
            ActiveVoice::Synth { inst, .. } => inst.tick(),
            ActiveVoice::Sample { inst, .. } => inst.tick(),
        }
    }
}

fn active() -> &'static Mutex<Vec<ActiveVoice>> {
    static A: OnceLock<Mutex<Vec<ActiveVoice>>> = OnceLock::new();
    A.get_or_init(|| Mutex::new(Vec::with_capacity(MAX_ACTIVE_VOICES)))
}

fn master() -> &'static Mutex<f32> {
    static M: OnceLock<Mutex<f32>> = OnceLock::new();
    M.get_or_init(|| Mutex::new(0.7))
}

pub fn master_gain() -> f32 {
    master().lock().map(|g| *g).unwrap_or(0.7)
}

pub fn set_master_gain(value: f32) {
    if let Ok(mut g) = master().lock() {
        *g = value.clamp(0.0, 1.5);
    }
}

pub fn play_voice(owner: &str, voice_id: &str, velocity: f32, seed: u32) -> Result<(), String> {
    let graph = {
        let v = voices().read().map_err(|e| e.to_string())?;
        let entry = v.get(voice_id).ok_or_else(|| "voice not found".to_string())?;
        if entry.owner != owner {
            return Err("voice owned by different extension".into());
        }
        entry.graph.clone()
    };
    let inst = VoiceInstance::spawn(graph, velocity, seed);
    let mut list = active().lock().map_err(|e| e.to_string())?;
    if list.len() >= MAX_ACTIVE_VOICES {
        list.remove(0);
    }
    list.push(ActiveVoice::Synth {
        owner: owner.to_string(),
        inst,
    });
    Ok(())
}

pub fn play_sample(
    owner: &str,
    sample_id: &str,
    gain: f32,
    pitch: f32,
) -> Result<(), String> {
    let buf = get_sample(sample_id).ok_or_else(|| "sample not found".to_string())?;
    if buf.owner != owner {
        return Err("sample owned by different extension".into());
    }
    let inst = SampleVoice::new(buf, super::sample_rate(), pitch, gain);
    let mut list = active().lock().map_err(|e| e.to_string())?;
    if list.len() >= MAX_ACTIVE_VOICES {
        list.remove(0);
    }
    list.push(ActiveVoice::Sample {
        owner: owner.to_string(),
        inst,
    });
    Ok(())
}

pub fn active_voice_count(owner: Option<&str>) -> usize {
    let Ok(list) = active().lock() else {
        return 0;
    };
    match owner {
        Some(o) => list.iter().filter(|v| v.owner() == o).count(),
        None => list.len(),
    }
}

/// Stop all voices, optionally restricted to one owner.
pub fn panic(owner: Option<&str>) {
    if let Ok(mut list) = active().lock() {
        match owner {
            Some(o) => list.retain(|v| v.owner() != o),
            None => list.clear(),
        }
    }
}

/// Backend audio-callback entrypoint. `out` is interleaved stereo; we duplicate the
/// mono mix to both channels. Length must be a multiple of `channels`.
pub fn mix_into(out: &mut [f32], channels: usize) {
    let gain = master_gain();
    let frames = out.len() / channels.max(1);
    let Ok(mut list) = active().lock() else {
        for s in out.iter_mut() {
            *s = 0.0;
        }
        return;
    };
    for frame in 0..frames {
        let mut sum = 0.0f32;
        for v in list.iter_mut() {
            sum += v.tick();
        }
        sum = (sum * gain).clamp(-1.0, 1.0);
        for ch in 0..channels {
            out[frame * channels + ch] = sum;
        }
    }
    list.retain(|v| !v.finished());
}

/// Wipe every voice + registered graph. Called on module shutdown.
pub fn clear_all() {
    if let Ok(mut list) = active().lock() {
        list.clear();
    }
    if let Ok(mut v) = voices().write() {
        v.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::audio::graph::{NodeKind, VariationSpec};

    fn test_lock() -> std::sync::MutexGuard<'static, ()> {
        static M: OnceLock<Mutex<()>> = OnceLock::new();
        M.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    }

    fn tiny_spec() -> VoiceSpec {
        VoiceSpec {
            nodes: vec![
                NodeKind::NoiseBurst {
                    id: "n".into(),
                    duration_ms: 1.0,
                    shape: "exp".into(),
                },
                NodeKind::EnvAr {
                    id: "env".into(),
                    attack_ms: 0.1,
                    release_ms: 1.0,
                },
            ],
            edges: vec![("n".into(), "env".into())],
            output: "env".into(),
            variations: VariationSpec::default(),
        }
    }

    #[test]
    fn register_play_and_owner_unregister() {
        let _g = test_lock();
        clear_all();
        let vid = register_voice("ext-a", tiny_spec()).unwrap();
        play_voice("ext-a", &vid, 0.8, 1).unwrap();
        assert_eq!(active_voice_count(Some("ext-a")), 1);
        let mut buf = vec![0.0; 256];
        mix_into(&mut buf, 2);
        // After mixing, the voice should be cleaned up (finished).
        assert_eq!(active_voice_count(Some("ext-a")), 0);
        unregister_owner("ext-a");
        assert!(voices().read().unwrap().is_empty());
    }

    #[test]
    fn play_voice_rejects_wrong_owner() {
        let _g = test_lock();
        clear_all();
        let vid = register_voice("ext-a", tiny_spec()).unwrap();
        let r = play_voice("ext-b", &vid, 1.0, 1);
        assert!(r.is_err());
    }

    #[test]
    fn panic_clears_only_owner() {
        let _g = test_lock();
        clear_all();
        let va = register_voice("ext-a", tiny_spec()).unwrap();
        let vb = register_voice("ext-b", tiny_spec()).unwrap();
        play_voice("ext-a", &va, 1.0, 1).unwrap();
        play_voice("ext-b", &vb, 1.0, 1).unwrap();
        panic(Some("ext-a"));
        assert_eq!(active_voice_count(Some("ext-a")), 0);
        assert_eq!(active_voice_count(Some("ext-b")), 1);
        panic(None);
        assert_eq!(active_voice_count(None), 0);
    }
}
