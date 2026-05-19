//! Animation module — the runtime wrapper around the [`opentween`] engine.
//!
//! The tween engine itself lives in the standalone `opentween` crate (no GUI
//! deps, no threads, no globals). This module owns one process-wide
//! [`AnimationEngine`] instance and a 16 ms driver thread that steps it — the
//! runtime concern that does *not* belong in a reusable crate.
//!
//! Easing curves, interpolation helpers, and the engine types are re-exported
//! so existing `animation::Easing` / `animation::tween` call sites are unchanged.

use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use opentween::{AnimationEngine, CompleteFn};

use crate::extension::{Extension, OwnedModuleCommand, OwnedModuleManifest};
use crate::modules::{ExecutionContext, ModuleCommand};
use crate::scheduling;

pub use opentween::{apply_easing, lerp_f32, lerp_rgba, lerp_u8, shake_offset, Easing, TweenId};

pub const NAME: &str = "animation";

// ─── Engine + driver state ──────────────────────────────────────────────────

static ENGINE: OnceLock<Mutex<AnimationEngine>> = OnceLock::new();

fn engine() -> &'static Mutex<AnimationEngine> {
    ENGINE.get_or_init(|| Mutex::new(AnimationEngine::new()))
}

struct DriverState {
    started: bool,
    task_id: Option<scheduling::TaskId>,
}

static DRIVER: OnceLock<Mutex<DriverState>> = OnceLock::new();

fn driver() -> &'static Mutex<DriverState> {
    DRIVER.get_or_init(|| {
        Mutex::new(DriverState {
            started: false,
            task_id: None,
        })
    })
}

/// Start the 16 ms driver if it is not already running. Idempotent.
fn ensure_driver() {
    let mut state = driver().lock().unwrap_or_else(|e| e.into_inner());
    if state.started {
        return;
    }
    state.started = true;
    drop(state);
    let id = scheduling::spawn_interval_on_lane(
        scheduling::TaskLane::Default,
        Duration::from_millis(16),
        drive,
    );
    driver().lock().unwrap_or_else(|e| e.into_inner()).task_id = Some(id);
}

/// One driver step: advance the engine, fire callbacks, stop when idle.
fn drive() {
    // Step the engine; collect callbacks. Lock released before firing them.
    let outcome = engine()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .advance(Instant::now());
    // Callbacks may chain new tweens — fire them with no lock held.
    outcome.dispatch();
    // Re-check emptiness *after* callbacks ran so a chained tween keeps the
    // driver alive. Lock order is always driver → engine.
    let stop_id = {
        let mut state = driver().lock().unwrap_or_else(|e| e.into_inner());
        if engine().lock().unwrap_or_else(|e| e.into_inner()).is_empty() {
            state.started = false;
            state.task_id.take()
        } else {
            None
        }
    };
    if let Some(id) = stop_id {
        scheduling::cancel(id);
    }
}

// ─── Public API (stable wrappers over the owned engine) ─────────────────────

/// Start a tween. `on_tick` receives eased `t ∈ [0, 1]` every ~16 ms until done.
pub fn tween(
    duration_ms: u64,
    easing: Easing,
    on_tick: impl Fn(f32) + Send + Sync + 'static,
) -> TweenId {
    tween_with_completion(duration_ms, easing, on_tick, None, None)
}

/// Like [`tween`] but fires `on_complete` once after the final tick.
/// `owner_extension` scopes [`cancel_all_for_extension`].
pub fn tween_with_completion(
    duration_ms: u64,
    easing: Easing,
    on_tick: impl Fn(f32) + Send + Sync + 'static,
    on_complete: Option<CompleteFn>,
    owner_extension: Option<String>,
) -> TweenId {
    ensure_driver();
    engine()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .start_with_completion(duration_ms, easing, on_tick, on_complete, owner_extension)
}

pub fn cancel(id: TweenId) {
    engine()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .cancel(id);
}

pub fn is_running(id: TweenId) -> bool {
    engine()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .is_running(id)
}

pub fn cancel_all() {
    engine()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .cancel_all();
}

/// Cancel all tweens owned by `ext_id`. Called when a Python extension unloads.
pub fn cancel_all_for_extension(ext_id: &str) {
    engine()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .cancel_all_for_owner(ext_id);
}

pub fn running_count() -> usize {
    engine()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .running_count()
}

// ─── Module commands ────────────────────────────────────────────────────────

fn cmd_list(_args: &[&str], _ctx: &ExecutionContext) -> String {
    let now = Instant::now();
    let snap = engine()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .snapshot(now);
    if snap.is_empty() {
        return "[]".to_string();
    }
    let entries: Vec<_> = snap
        .iter()
        .map(|s| {
            serde_json::json!({
                "id": s.id,
                "elapsed_ms": s.elapsed_ms,
                "duration_ms": s.duration_ms,
                "easing": s.easing,
                "owner": s.owner.as_deref().unwrap_or("native"),
            })
        })
        .collect();
    serde_json::to_string(&entries).unwrap_or_else(|_| "[]".to_string())
}

fn cmd_cancel(args: &[&str], _ctx: &ExecutionContext) -> String {
    let Some(id_str) = args.first() else {
        return "Usage: animation.cancel <tween_id>".to_string();
    };
    let Ok(id_num) = id_str.parse::<u64>() else {
        return format!("Invalid tween id: {id_str}");
    };
    let id = TweenId(id_num);
    let was_running = is_running(id);
    cancel(id);
    if was_running {
        format!("Cancelled tween {id_num}")
    } else {
        format!("Tween {id_num} not found")
    }
}

fn cmd_cancel_all(_args: &[&str], _ctx: &ExecutionContext) -> String {
    let count = running_count();
    cancel_all();
    format!("Cancelled {count} tweens")
}

pub fn commands() -> &'static [ModuleCommand] {
    &[
        ModuleCommand {
            name: "list",
            description:
                "JSON array of running tweens (id, elapsed_ms, duration_ms, easing, owner)",
            required_permissions: &[],
            run: cmd_list,
        },
        ModuleCommand {
            name: "cancel",
            description: "Cancel a running tween by numeric id: animation.cancel <id>",
            required_permissions: &[],
            run: cmd_cancel,
        },
        ModuleCommand {
            name: "cancel_all",
            description: "Cancel all running tweens",
            required_permissions: &[],
            run: cmd_cancel_all,
        },
    ]
}

#[derive(Default)]
pub struct AnimationExtension;

impl Extension for AnimationExtension {
    fn manifest(&self) -> OwnedModuleManifest {
        OwnedModuleManifest {
            name: NAME.to_string(),
            glyph: "animation".to_string(),
            version: "0.1.0".to_string(),
            description:
                "Shared tween engine for modules and extensions. One 16 ms driver loop services all running animations."
                    .to_string(),
            accent: String::new(),
            required_modules: Vec::new(),
            required_permissions: Vec::new(),
            workspace_permissions: Vec::new(),
            supported_platforms: Vec::new(),
            api_exports: Vec::new(),
        }
    }

    fn commands(&self) -> Vec<OwnedModuleCommand> {
        commands()
            .iter()
            .map(OwnedModuleCommand::from_static)
            .collect()
    }
}

crate::register_extension!(AnimationExtension);

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::{Arc, Mutex as StdMutex};

    static TEST_LOCK: StdMutex<()> = StdMutex::new(());

    fn with_clean_state<F: FnOnce()>(f: F) {
        let _g = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        cancel_all();
        f();
        cancel_all();
    }

    #[test]
    fn tween_registers_and_cancels() {
        with_clean_state(|| {
            let id = tween(10_000, Easing::Linear, |_| {});
            assert!(is_running(id));
            cancel(id);
            assert!(!is_running(id));
        });
    }

    #[test]
    fn cancel_all_for_extension_scoped() {
        with_clean_state(|| {
            let ext_id = tween_with_completion(
                10_000,
                Easing::Linear,
                |_| {},
                None,
                Some("my-ext".to_string()),
            );
            let native_id = tween(10_000, Easing::Linear, |_| {});
            cancel_all_for_extension("my-ext");
            assert!(!is_running(ext_id), "extension tween must be gone");
            assert!(is_running(native_id), "native tween must survive");
            cancel(native_id);
        });
    }

    #[test]
    fn running_count_tracks_active_tweens() {
        with_clean_state(|| {
            assert_eq!(running_count(), 0);
            let a = tween(10_000, Easing::Linear, |_| {});
            let b = tween(10_000, Easing::Linear, |_| {});
            assert_eq!(running_count(), 2);
            cancel(a);
            cancel(b);
        });
    }
}
