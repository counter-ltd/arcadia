//! Generic audio engine module.
//!
//! Core owns:
//! - DSP node primitives ([`dsp`])
//! - Voice graph compiler ([`graph`])
//! - Sample buffer registry ([`sample`])
//! - Voice pool + mixer ([`engine`])
//! - Backend trait ([`AudioBackend`])
//!
//! Desktop / iOS surfaces install a backend that opens a real output stream and
//! drains the mixer at audio-callback rate. Headless / unsupported surfaces have
//! no backend; voice triggers silently no-op.
//!
//! Nothing in this module names a specific consumer ("click", "typing", "keypress");
//! consumers are extensions that build voice graphs from the generic node kinds.

pub mod dsp;
pub mod engine;
pub mod graph;
pub mod sample;

use std::sync::{Mutex, OnceLock};

use crate::modules::{ExecutionContext, ModuleCommand};

pub const NAME: &str = "audio";

pub use engine::{
    active_voice_count, master_gain, mix_into, panic, play_sample, play_voice, register_voice,
    set_master_gain, unregister_voice, OWNER_DELIMITER,
};
pub use graph::{CompiledGraph, NodeKind, VoiceSpec};
pub use sample::{load_sample, sample_meta, unload_sample, SampleBuffer};

pub trait AudioBackend: Send + Sync {
    /// Start the output stream. Idempotent — repeat calls return `Ok` once started.
    fn start(&self) -> Result<(), String>;
    fn stop(&self);
    fn is_active(&self) -> bool {
        false
    }
    /// Output sample rate. Implementations should match this in their callback; the
    /// mixer renders at this rate so DSP coefficient calculation can use it.
    fn sample_rate(&self) -> u32 {
        48_000
    }
}

fn backend_slot() -> &'static Mutex<Option<Box<dyn AudioBackend>>> {
    static B: OnceLock<Mutex<Option<Box<dyn AudioBackend>>>> = OnceLock::new();
    B.get_or_init(|| Mutex::new(None))
}

pub fn set_backend(b: Box<dyn AudioBackend>) {
    if let Ok(mut slot) = backend_slot().lock() {
        *slot = Some(b);
    }
}

pub fn start_engine() -> Result<(), String> {
    let slot = backend_slot()
        .lock()
        .map_err(|e| format!("audio backend lock poisoned: {e}"))?;
    match slot.as_ref() {
        Some(b) => b.start(),
        None => Err("Audio backend not available on this surface.".to_string()),
    }
}

pub fn stop_engine() {
    if let Ok(slot) = backend_slot().lock() {
        if let Some(b) = slot.as_ref() {
            b.stop();
        }
    }
}

pub fn engine_active() -> bool {
    backend_slot()
        .lock()
        .ok()
        .and_then(|slot| slot.as_ref().map(|b| b.is_active()))
        .unwrap_or(false)
}

pub fn sample_rate() -> u32 {
    backend_slot()
        .lock()
        .ok()
        .and_then(|slot| slot.as_ref().map(|b| b.sample_rate()))
        .unwrap_or(48_000)
}

// ─── Commands ────────────────────────────────────────────────────────────────

fn cmd_state(_args: &[&str], _ctx: &ExecutionContext) -> String {
    format!(
        "active={} sample_rate={} master_gain={:.2} active_voices={}",
        engine_active(),
        sample_rate(),
        master_gain(),
        active_voice_count(None),
    )
}

fn cmd_panic(_args: &[&str], _ctx: &ExecutionContext) -> String {
    panic(None);
    "All voices stopped.".to_string()
}

pub fn commands() -> &'static [ModuleCommand] {
    &[
        ModuleCommand {
            name: "state",
            description: "Print audio engine state: active, sample_rate, master_gain, active_voices.",
            required_permissions: &["audio.output"],
            run: cmd_state,
        },
        ModuleCommand {
            name: "panic",
            description: "Stop all currently playing voices.",
            required_permissions: &["audio.output"],
            run: cmd_panic,
        },
    ]
}

// ─── Extension self-registration ─────────────────────────────────────────────

#[derive(Default)]
pub struct AudioExtension;

impl crate::extension::Extension for AudioExtension {
    fn manifest(&self) -> crate::extension::OwnedModuleManifest {
        crate::extension::OwnedModuleManifest {
            name: NAME.to_string(),
            glyph: "audio".to_string(),
            version: "0.1.0".to_string(),
            description:
                "Low-latency audio output with DSP node graphs and sample buffer playback."
                    .to_string(),
            accent: String::new(),
            required_modules: Vec::new(),
            required_permissions: vec!["audio.output".to_string()],
            workspace_permissions: Vec::new(),
            supported_platforms: vec![
                crate::platform::PLATFORM_MACOS.to_string(),
                crate::platform::PLATFORM_WINDOWS.to_string(),
                crate::platform::PLATFORM_LINUX.to_string(),
            ],
            api_exports: Vec::new(),
        }
    }

    fn commands(&self) -> &'static [crate::modules::ModuleCommand] {
        commands()
    }

    fn permissions(&self) -> &'static [crate::config::permissions::PermissionDefinition] {
        use crate::config::permissions::PermissionDefinition;
        static PERMS: &[PermissionDefinition] = &[PermissionDefinition {
            id: "audio.output",
            title: "Play audio",
            description:
                "Generate and play sound through the system audio output.",
            default_global: false,
            system_grant: None,
        }];
        PERMS
    }

    fn shutdown(&self) {
        stop_engine();
        engine::clear_all();
    }
}

crate::register_extension!(AudioExtension);
