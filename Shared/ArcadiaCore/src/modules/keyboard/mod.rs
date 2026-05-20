//! Global keyboard / pointer event module — generic OS input stream.
//!
//! Exposes a single, neutral event stream (`KeyboardEvent`) that any extension can
//! subscribe to. Core defines the API + a backend trait; the desktop surface installs
//! a platform-specific implementation at startup. Headless / unsupported surfaces →
//! `start_observer` returns `Err`, handlers never fire.
//!
//! This module owns no product semantics — no "click", "press", "release-click", no
//! per-app filtering. Extensions interpret the raw stream. See `Extensions/clonk/` for
//! one consumer.

use std::sync::{Mutex, OnceLock, RwLock};

use serde::{Deserialize, Serialize};

use crate::modules::{ExecutionContext, ModuleCommand};

pub const NAME: &str = "keyboard";

/// Tagged event variant emitted by the OS-level observer thread. `kind` discriminates;
/// fields are flattened in the JSON form for ergonomic Python `event["keycode"]`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum KeyboardEvent {
    KeyDown {
        keycode: u32,
        modifiers: u32,
        repeat: bool,
        timestamp_ns: u64,
    },
    KeyUp {
        keycode: u32,
        modifiers: u32,
        timestamp_ns: u64,
    },
    MouseDown {
        button: u8,
        timestamp_ns: u64,
    },
    MouseUp {
        button: u8,
        timestamp_ns: u64,
    },
    ScrollWheel {
        dx: f32,
        dy: f32,
        timestamp_ns: u64,
    },
}

/// Backend trait — desktop surfaces install one matching the host OS. The backend
/// runs its own listener thread and pushes each event through [`dispatch_event`].
/// `stop` is best-effort — backends that cannot detach (e.g. CGEventTap on a runloop)
/// simply leave the thread parked until process exit.
pub trait KeyboardBackend: Send + Sync {
    fn start(&self) -> Result<(), String>;
    fn stop(&self);
    fn is_active(&self) -> bool {
        false
    }
    /// Whether the OS-level grant (macOS Input Monitoring etc.) has been awarded to
    /// this process. `None` if the backend cannot determine grant state cheaply.
    fn permission_granted(&self) -> Option<bool> {
        None
    }
}

fn backend_slot() -> &'static Mutex<Option<Box<dyn KeyboardBackend>>> {
    static B: OnceLock<Mutex<Option<Box<dyn KeyboardBackend>>>> = OnceLock::new();
    B.get_or_init(|| Mutex::new(None))
}

pub fn set_backend(b: Box<dyn KeyboardBackend>) {
    if let Ok(mut slot) = backend_slot().lock() {
        *slot = Some(b);
    }
}

/// Start the installed backend. Idempotent — repeat calls return `Ok` once started.
/// Returns `Err` when no backend is installed (headless surface) or the OS rejects
/// the listener (permission not granted, runloop unavailable).
pub fn start_observer() -> Result<(), String> {
    let slot = backend_slot()
        .lock()
        .map_err(|e| format!("keyboard backend lock poisoned: {e}"))?;
    match slot.as_ref() {
        Some(b) => b.start(),
        None => Err("Keyboard backend not available on this surface.".to_string()),
    }
}

pub fn stop_observer() {
    if let Ok(slot) = backend_slot().lock() {
        if let Some(b) = slot.as_ref() {
            b.stop();
        }
    }
}

pub fn observer_active() -> bool {
    backend_slot()
        .lock()
        .ok()
        .and_then(|slot| slot.as_ref().map(|b| b.is_active()))
        .unwrap_or(false)
}

pub fn permission_granted() -> Option<bool> {
    backend_slot()
        .lock()
        .ok()
        .and_then(|slot| slot.as_ref().and_then(|b| b.permission_granted()))
}

// ─── Handler registry ────────────────────────────────────────────────────────
//
// Subscribers are stored as `Arc<dyn Fn(KeyboardEvent) + Send + Sync>` keyed by a
// caller-supplied id (typically the extension id). The Python binding installs one
// handler per extension and uses the id to unregister on disable.

pub type Handler = std::sync::Arc<dyn Fn(KeyboardEvent) + Send + Sync>;

fn handlers() -> &'static RwLock<Vec<(String, Handler)>> {
    static H: OnceLock<RwLock<Vec<(String, Handler)>>> = OnceLock::new();
    H.get_or_init(|| RwLock::new(Vec::new()))
}

/// Register `handler` under `subscriber_id`. Replaces an existing handler with the
/// same id so a Python extension reload doesn't leak callbacks.
pub fn register_handler(subscriber_id: String, handler: Handler) {
    if let Ok(mut list) = handlers().write() {
        list.retain(|(id, _)| id != &subscriber_id);
        list.push((subscriber_id, handler));
    }
}

pub fn unregister_handler(subscriber_id: &str) {
    if let Ok(mut list) = handlers().write() {
        list.retain(|(id, _)| id != subscriber_id);
    }
}

pub fn subscriber_count() -> usize {
    handlers().read().map(|l| l.len()).unwrap_or(0)
}

/// Backend entrypoint. Fan out to every registered handler. Each handler must be
/// non-blocking; an extension that needs to do work should hand the event to its
/// own thread or queue.
pub fn dispatch_event(ev: KeyboardEvent) {
    let snapshot: Vec<(String, Handler)> = {
        let Ok(list) = handlers().read() else {
            return;
        };
        list.iter().map(|(id, h)| (id.clone(), h.clone())).collect()
    };
    // Per-handler panic guard: a panic must not propagate to the backend's C-FFI
    // callback (CGEventTap → SIGTRAP). Each subscriber's failure is isolated.
    for (sid, h) in snapshot {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| h(ev)));
        if result.is_err() {
            eprintln!("keyboard subscriber {sid} panicked; event dropped");
        }
    }
}

// ─── Commands ────────────────────────────────────────────────────────────────

fn cmd_state(_args: &[&str], _ctx: &ExecutionContext) -> String {
    let active = observer_active();
    let granted = permission_granted();
    let granted_s = match granted {
        Some(true) => "true",
        Some(false) => "false",
        None => "unknown",
    };
    format!(
        "active={active} permission_granted={granted_s} subscribers={}",
        subscriber_count()
    )
}

fn cmd_start(_args: &[&str], _ctx: &ExecutionContext) -> String {
    match start_observer() {
        Ok(()) => "Keyboard observer started.".to_string(),
        Err(e) => format!("Failed to start keyboard observer: {e}"),
    }
}

fn cmd_stop(_args: &[&str], _ctx: &ExecutionContext) -> String {
    stop_observer();
    "Keyboard observer stopped.".to_string()
}

pub fn commands() -> &'static [ModuleCommand] {
    &[
        ModuleCommand {
            name: "state",
            description: "Print observer state: active, permission_granted, subscriber count.",
            required_permissions: &["keyboard.global_events"],
            run: cmd_state,
        },
        ModuleCommand {
            name: "start",
            description: "Start the OS-level keyboard / pointer observer.",
            required_permissions: &["keyboard.global_events"],
            run: cmd_start,
        },
        ModuleCommand {
            name: "stop",
            description: "Stop the OS-level keyboard / pointer observer.",
            required_permissions: &["keyboard.global_events"],
            run: cmd_stop,
        },
    ]
}

// ─── Extension self-registration ─────────────────────────────────────────────

#[derive(Default)]
pub struct KeyboardExtension;

impl crate::extension::Extension for KeyboardExtension {
    fn manifest(&self) -> crate::extension::OwnedModuleManifest {
        crate::extension::OwnedModuleManifest {
            name: NAME.to_string(),
            glyph: "keyboard".to_string(),
            version: "0.1.0".to_string(),
            description:
                "Generic OS-global keyboard, mouse, and scroll event stream for extensions."
                    .to_string(),
            accent: String::new(),
            required_modules: Vec::new(),
            required_permissions: vec!["keyboard.global_events".to_string()],
            workspace_permissions: Vec::new(),
            supported_platforms: vec![crate::platform::PLATFORM_MACOS.to_string()],
            api_exports: Vec::new(),
        }
    }

    fn commands(&self) -> &'static [crate::modules::ModuleCommand] {
        commands()
    }

    fn permissions(&self) -> &'static [crate::config::permissions::PermissionDefinition] {
        use crate::config::permissions::{PermissionDefinition, SystemGrant};
        static PERMS: &[PermissionDefinition] = &[PermissionDefinition {
            id: "keyboard.global_events",
            title: "Global keyboard & pointer events",
            description: "Receive notifications of key presses, mouse clicks, and scroll events anywhere on the OS, even when Arcadia is not focused (macOS: Accessibility — same gate as cursor.global_position). Event payload includes keycode and modifier mask.",
            default_global: false,
            system_grant: Some(SystemGrant::Accessibility),
        }];
        PERMS
    }

    fn shutdown(&self) {
        stop_observer();
        if let Ok(mut list) = handlers().write() {
            list.clear();
        }
    }
}

crate::register_extension!(KeyboardExtension);

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    fn test_lock() -> std::sync::MutexGuard<'static, ()> {
        static M: OnceLock<Mutex<()>> = OnceLock::new();
        M.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    }

    fn reset_all_handlers() {
        if let Ok(mut list) = handlers().write() {
            list.clear();
        }
    }

    #[test]
    fn handler_register_and_dispatch_round_trip() {
        let _g = test_lock();
        reset_all_handlers();
        let count = Arc::new(AtomicUsize::new(0));
        let c1 = count.clone();
        register_handler(
            "test-a".to_string(),
            Arc::new(move |_ev| {
                c1.fetch_add(1, Ordering::SeqCst);
            }),
        );
        let c2 = count.clone();
        register_handler(
            "test-b".to_string(),
            Arc::new(move |_ev| {
                c2.fetch_add(10, Ordering::SeqCst);
            }),
        );
        dispatch_event(KeyboardEvent::KeyDown {
            keycode: 7,
            modifiers: 0,
            repeat: false,
            timestamp_ns: 0,
        });
        assert_eq!(count.load(Ordering::SeqCst), 11);
        unregister_handler("test-a");
        dispatch_event(KeyboardEvent::KeyUp {
            keycode: 7,
            modifiers: 0,
            timestamp_ns: 0,
        });
        assert_eq!(count.load(Ordering::SeqCst), 21);
        unregister_handler("test-b");
    }

    #[test]
    fn duplicate_id_replaces_previous_handler() {
        let _g = test_lock();
        reset_all_handlers();
        let count = Arc::new(AtomicUsize::new(0));
        let c1 = count.clone();
        register_handler(
            "dup".to_string(),
            Arc::new(move |_ev| {
                c1.fetch_add(1, Ordering::SeqCst);
            }),
        );
        let c2 = count.clone();
        register_handler(
            "dup".to_string(),
            Arc::new(move |_ev| {
                c2.fetch_add(100, Ordering::SeqCst);
            }),
        );
        dispatch_event(KeyboardEvent::MouseDown {
            button: 0,
            timestamp_ns: 0,
        });
        assert_eq!(count.load(Ordering::SeqCst), 100);
        unregister_handler("dup");
    }
}
