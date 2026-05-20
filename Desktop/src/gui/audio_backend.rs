//! Desktop audio backend: cpal output stream that drains the core mixer.
//!
//! Wired into `arcadia_core::modules::audio` from desktop startup via [`install`].
//!
//! Pattern follows the cursor / tray backends: core owns the engine state, this
//! file owns the OS resources. The audio thread invokes
//! [`arcadia_core::modules::audio::engine::mix_into`] each callback.

use std::sync::Mutex;

use arcadia_core::config::permissions::PermissionsConfig;
use arcadia_core::config::ConfigFile;
use arcadia_core::modules::audio::{self, engine, AudioBackend};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

const AUDIO_OUTPUT_PERMISSION: &str = "audio.output";

fn audio_output_enabled() -> bool {
    PermissionsConfig::load_or_create()
        .ok()
        .map(|cfg| cfg.global_allowed(AUDIO_OUTPUT_PERMISSION))
        .unwrap_or(false)
}

pub struct DesktopAudioBackend {
    inner: Mutex<Option<RunningStream>>,
    sample_rate: Mutex<u32>,
}

/// `cpal::Stream` is not Send on all platforms (macOS in particular). Park it on the
/// dedicated audio-owner thread and never move it across threads after creation.
struct RunningStream {
    _stream: cpal::Stream,
}

impl DesktopAudioBackend {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(None),
            sample_rate: Mutex::new(48_000),
        }
    }
}

unsafe impl Send for DesktopAudioBackend {}
unsafe impl Sync for DesktopAudioBackend {}

impl AudioBackend for DesktopAudioBackend {
    fn start(&self) -> Result<(), String> {
        let mut guard = self.inner.lock().map_err(|e| e.to_string())?;
        if guard.is_some() {
            return Ok(());
        }
        if !audio_output_enabled() {
            return Err(
                "Permission denied: audio.output (toggle on in Permissions settings)".into(),
            );
        }
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| "no default audio output device".to_string())?;
        let cfg_supp = device
            .default_output_config()
            .map_err(|e| format!("default output config: {e}"))?;
        let sample_rate = cfg_supp.sample_rate().0;
        let channels = cfg_supp.channels() as usize;
        if let Ok(mut sr) = self.sample_rate.lock() {
            *sr = sample_rate;
        }
        let cfg: cpal::StreamConfig = cfg_supp.config();

        let err_fn = |e| eprintln!("audio stream error: {e}");
        let stream = match cfg_supp.sample_format() {
            cpal::SampleFormat::F32 => device.build_output_stream(
                &cfg,
                move |buf: &mut [f32], _| {
                    engine::mix_into(buf, channels);
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::I16 => device.build_output_stream(
                &cfg,
                move |buf: &mut [i16], _| {
                    let mut tmp = vec![0.0f32; buf.len()];
                    engine::mix_into(&mut tmp, channels);
                    for (dst, src) in buf.iter_mut().zip(tmp.iter()) {
                        *dst = (src.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
                    }
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::U16 => device.build_output_stream(
                &cfg,
                move |buf: &mut [u16], _| {
                    let mut tmp = vec![0.0f32; buf.len()];
                    engine::mix_into(&mut tmp, channels);
                    for (dst, src) in buf.iter_mut().zip(tmp.iter()) {
                        let s = ((src.clamp(-1.0, 1.0) + 1.0) * 0.5 * u16::MAX as f32) as u16;
                        *dst = s;
                    }
                },
                err_fn,
                None,
            ),
            fmt => return Err(format!("unsupported sample format: {fmt:?}")),
        }
        .map_err(|e| format!("build output stream: {e}"))?;

        stream.play().map_err(|e| format!("play stream: {e}"))?;
        *guard = Some(RunningStream { _stream: stream });
        Ok(())
    }

    fn stop(&self) {
        if let Ok(mut g) = self.inner.lock() {
            *g = None;
        }
    }

    fn is_active(&self) -> bool {
        self.inner.lock().map(|g| g.is_some()).unwrap_or(false)
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate.lock().map(|s| *s).unwrap_or(48_000)
    }
}

pub fn install() {
    audio::set_backend(Box::new(DesktopAudioBackend::new()));
    // Eagerly try to start the stream from the install thread (main / GUI startup
    // thread). Without this, the first `audio_play_voice` from a Python keyboard
    // handler would create the cpal Stream from a background thread — a source
    // of subtle Core Audio thread-affinity bugs. If the permission isn't granted
    // yet, `start_engine` returns `Err`; we retry on next touch via the binding.
    let _ = audio::start_engine();
}
