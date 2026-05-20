//! Sample buffer registry. WAV decode via `hound`; AIFF / FLAC out of scope for v1.
//!
//! Buffers are stored mono-mixed at the buffer's native rate. The mixer resamples by
//! integer-step playback (no pitch shift) — pitch shift is parameterised at trigger
//! time. Resampling is linear, sufficient for short percussive samples (~50ms).

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug)]
pub struct SampleBuffer {
    pub id: String,
    pub owner: String,
    pub data: Arc<[f32]>,
    pub sample_rate: u32,
}

fn registry() -> &'static RwLock<BTreeMap<String, SampleBuffer>> {
    static R: std::sync::OnceLock<RwLock<BTreeMap<String, SampleBuffer>>> =
        std::sync::OnceLock::new();
    R.get_or_init(|| RwLock::new(BTreeMap::new()))
}

fn next_id() -> String {
    static N: AtomicU64 = AtomicU64::new(1);
    let n = N.fetch_add(1, Ordering::Relaxed);
    format!("sample-{n}")
}

/// Load a WAV file. Returns the assigned sample id.
pub fn load_sample(owner: &str, path: &Path) -> Result<String, String> {
    let data_and_rate = decode_wav(path)?;
    let id = next_id();
    let buf = SampleBuffer {
        id: id.clone(),
        owner: owner.to_string(),
        data: data_and_rate.0.into(),
        sample_rate: data_and_rate.1,
    };
    if let Ok(mut r) = registry().write() {
        r.insert(id.clone(), buf);
    }
    Ok(id)
}

pub fn unload_sample(owner: &str, id: &str) -> Result<(), String> {
    let mut r = registry().write().map_err(|e| e.to_string())?;
    match r.get(id) {
        Some(s) if s.owner == owner => {
            r.remove(id);
            Ok(())
        }
        Some(_) => Err("sample owned by different extension".into()),
        None => Err("sample not found".into()),
    }
}

pub fn get_sample(id: &str) -> Option<SampleBuffer> {
    registry().read().ok()?.get(id).cloned()
}

pub fn sample_meta(id: &str) -> Option<(u32, usize)> {
    let r = registry().read().ok()?;
    r.get(id).map(|s| (s.sample_rate, s.data.len()))
}

/// Drop every sample owned by `owner`. Called on extension disable.
pub fn unload_owner(owner: &str) {
    if let Ok(mut r) = registry().write() {
        let to_remove: Vec<String> = r
            .iter()
            .filter(|(_, s)| s.owner == owner)
            .map(|(k, _)| k.clone())
            .collect();
        for k in to_remove {
            r.remove(&k);
        }
    }
}

fn decode_wav(path: &Path) -> Result<(Vec<f32>, u32), String> {
    let reader = hound::WavReader::open(path)
        .map_err(|e| format!("open wav {}: {e}", path.display()))?;
    let spec = reader.spec();
    let channels = spec.channels.max(1) as usize;
    let rate = spec.sample_rate;
    let mut samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader
            .into_samples::<f32>()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("decode wav: {e}"))?,
        hound::SampleFormat::Int => {
            let bits = spec.bits_per_sample as i32;
            let max = (1i64 << (bits - 1)) as f32;
            reader
                .into_samples::<i32>()
                .map(|r| r.map(|i| (i as f32) / max))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("decode wav: {e}"))?
        }
    };
    if channels > 1 {
        let mono: Vec<f32> = samples
            .chunks(channels)
            .map(|c| c.iter().sum::<f32>() / channels as f32)
            .collect();
        samples = mono;
    }
    Ok((samples, rate))
}

// ─── Sample voice instance ───────────────────────────────────────────────────

pub struct SampleVoice {
    buf: SampleBuffer,
    cursor: f32,
    step: f32,
    gain: f32,
}

impl SampleVoice {
    pub fn new(buf: SampleBuffer, output_rate: u32, pitch: f32, gain: f32) -> Self {
        let step = (buf.sample_rate as f32 / output_rate as f32) * pitch.max(0.01);
        Self {
            buf,
            cursor: 0.0,
            step,
            gain: gain.clamp(0.0, 4.0),
        }
    }

    pub fn finished(&self) -> bool {
        self.cursor as usize >= self.buf.data.len()
    }

    pub fn tick(&mut self) -> f32 {
        let len = self.buf.data.len();
        if len == 0 {
            return 0.0;
        }
        let i = self.cursor as usize;
        if i >= len {
            return 0.0;
        }
        let frac = self.cursor - i as f32;
        let a = self.buf.data[i];
        let b = if i + 1 < len { self.buf.data[i + 1] } else { 0.0 };
        let y = a + (b - a) * frac;
        self.cursor += self.step;
        y * self.gain
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_test_wav(path: &Path, samples: &[i16], rate: u32) {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut w = hound::WavWriter::create(path, spec).unwrap();
        for s in samples {
            w.write_sample(*s).unwrap();
        }
        w.finalize().unwrap();
    }

    #[test]
    fn load_and_play_sample_round_trip() {
        let dir = std::env::temp_dir().join("arcadia-audio-test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("a.wav");
        write_test_wav(&path, &[0, 16384, -16384, 0, 8192, -8192], 48_000);
        let id = load_sample("ext1", &path).unwrap();
        let meta = sample_meta(&id).unwrap();
        assert_eq!(meta.0, 48_000);
        assert_eq!(meta.1, 6);
        let buf = get_sample(&id).unwrap();
        let mut v = SampleVoice::new(buf, 48_000, 1.0, 1.0);
        let mut n = 0;
        while !v.finished() && n < 16 {
            let _ = v.tick();
            n += 1;
        }
        assert!(v.finished());
        unload_owner("ext1");
        assert!(get_sample(&id).is_none());
    }
}
