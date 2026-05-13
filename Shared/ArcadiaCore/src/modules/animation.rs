//! Single-driver tween engine shared by native modules and Python extensions.
//!
//! One 16 ms interval drives all running tweens — no per-animation timer overhead.
//! Callers receive a normalized eased `t ∈ [0.0, 1.0]` in their `on_tick` callback.
//! The driver starts lazily on the first [`tween`] or [`tween_with_completion`] call.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::modules::{ExecutionContext, ModuleCommand};
use crate::scheduling;

pub const NAME: &str = "animation";

// ─── Public types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TweenId(pub u64);

#[derive(Debug, Clone, Copy)]
pub enum Easing {
    Linear,
    EaseOutCubic,
    EaseInCubic,
    EaseInOutCubic,
    EaseOutElastic,
}

impl Easing {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "linear" => Some(Easing::Linear),
            "ease_out_cubic" => Some(Easing::EaseOutCubic),
            "ease_in_cubic" => Some(Easing::EaseInCubic),
            "ease_in_out_cubic" => Some(Easing::EaseInOutCubic),
            "ease_out_elastic" => Some(Easing::EaseOutElastic),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Easing::Linear => "linear",
            Easing::EaseOutCubic => "ease_out_cubic",
            Easing::EaseInCubic => "ease_in_cubic",
            Easing::EaseInOutCubic => "ease_in_out_cubic",
            Easing::EaseOutElastic => "ease_out_elastic",
        }
    }
}

// ─── Math utilities ───────────────────────────────────────────────────────────

pub fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).round() as u8
}

/// Lerp each channel of an RGBA `[u8; 4]` value.
pub fn lerp_rgba(a: [u8; 4], b: [u8; 4], t: f32) -> [u8; 4] {
    [
        lerp_u8(a[0], b[0], t),
        lerp_u8(a[1], b[1], t),
        lerp_u8(a[2], b[2], t),
        lerp_u8(a[3], b[3], t),
    ]
}

/// Map raw `t ∈ [0.0, 1.0]` through the chosen easing curve.
pub fn apply_easing(easing: Easing, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    match easing {
        Easing::Linear => t,
        Easing::EaseOutCubic => 1.0 - (1.0 - t).powi(3),
        Easing::EaseInCubic => t * t * t,
        Easing::EaseInOutCubic => {
            if t < 0.5 {
                4.0 * t * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
            }
        }
        Easing::EaseOutElastic => {
            if t == 0.0 || t == 1.0 {
                return t;
            }
            let c4 = (2.0 * std::f32::consts::PI) / 3.0;
            2.0_f32.powf(-10.0 * t) * ((t * 10.0 - 10.75) * c4).sin() + 1.0
        }
    }
}

// ─── Engine internals ─────────────────────────────────────────────────────────

struct TweenEntry {
    start_instant: Instant,
    duration_ms: u64,
    easing: Easing,
    on_tick: Arc<dyn Fn(f32) + Send + Sync + 'static>,
    on_complete: Option<Box<dyn FnOnce() + Send + 'static>>,
    owner_extension: Option<String>,
}

struct EngineInner {
    tweens: BTreeMap<TweenId, TweenEntry>,
    next_id: u64,
    driver_started: bool,
    driver_task_id: Option<scheduling::TaskId>,
}

static ENGINE: OnceLock<Mutex<EngineInner>> = OnceLock::new();

fn engine() -> &'static Mutex<EngineInner> {
    ENGINE.get_or_init(|| {
        Mutex::new(EngineInner {
            tweens: BTreeMap::new(),
            next_id: 1,
            driver_started: false,
            driver_task_id: None,
        })
    })
}

fn ensure_driver() {
    let mut inner = engine().lock().unwrap_or_else(|e| e.into_inner());
    if inner.driver_started {
        return;
    }
    inner.driver_started = true;
    drop(inner);
    let id = scheduling::spawn_interval_on_lane(
        scheduling::TaskLane::Default,
        Duration::from_millis(16),
        tick,
    );
    engine().lock().unwrap_or_else(|e| e.into_inner()).driver_task_id = Some(id);
}

fn tick() {
    let now = Instant::now();

    // Phase 1: snapshot callbacks and mark completions — lock held briefly.
    let ticks: Vec<(Arc<dyn Fn(f32) + Send + Sync>, f32)>;
    let completed: Vec<TweenId>;
    {
        let inner = engine().lock().unwrap_or_else(|e| e.into_inner());
        let mut t_list = Vec::new();
        let mut done = Vec::new();
        for (id, entry) in &inner.tweens {
            let raw_t = if entry.duration_ms == 0 {
                1.0
            } else {
                (now.duration_since(entry.start_instant).as_millis() as f32
                    / entry.duration_ms as f32)
                    .min(1.0)
            };
            let eased_t = apply_easing(entry.easing, raw_t);
            t_list.push((Arc::clone(&entry.on_tick), eased_t));
            if raw_t >= 1.0 {
                done.push(*id);
            }
        }
        ticks = t_list;
        completed = done;
    }

    // Phase 2: fire on_tick callbacks without holding the lock so callers can call back into the
    // engine (e.g. chain a new tween on completion).
    for (cb, t) in ticks {
        cb(t);
    }

    // Phase 3: extract on_complete closures — brief lock.
    let complete_cbs: Vec<Box<dyn FnOnce() + Send + 'static>>;
    {
        let mut inner = engine().lock().unwrap_or_else(|e| e.into_inner());
        complete_cbs = completed
            .into_iter()
            .filter_map(|id| inner.tweens.remove(&id).and_then(|e| e.on_complete))
            .collect();
    }

    // Phase 4: fire on_complete without holding the lock.
    for cb in complete_cbs {
        cb();
    }

    // Phase 5: stop the recurring driver when all tweens have finished.
    let cancel_id = {
        let mut inner = engine().lock().unwrap_or_else(|e| e.into_inner());
        if inner.tweens.is_empty() {
            inner.driver_started = false;
            inner.driver_task_id.take()
        } else {
            None
        }
    };
    if let Some(id) = cancel_id {
        scheduling::cancel(id);
    }
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Start a tween. `on_tick` receives eased `t ∈ [0.0, 1.0]` every ~16 ms until completion.
/// The driver is started on the first call; subsequent calls share the same interval.
pub fn tween(
    duration_ms: u64,
    easing: Easing,
    on_tick: impl Fn(f32) + Send + Sync + 'static,
) -> TweenId {
    tween_with_completion(duration_ms, easing, on_tick, None, None)
}

/// Like [`tween`] but fires `on_complete` once after the last `on_tick(1.0)` call.
/// `owner_extension` is set by Python bindings so [`cancel_all_for_extension`] can scope cancels.
pub fn tween_with_completion(
    duration_ms: u64,
    easing: Easing,
    on_tick: impl Fn(f32) + Send + Sync + 'static,
    on_complete: Option<Box<dyn FnOnce() + Send + 'static>>,
    owner_extension: Option<String>,
) -> TweenId {
    ensure_driver();
    let mut inner = engine().lock().unwrap_or_else(|e| e.into_inner());
    let id = TweenId(inner.next_id);
    inner.next_id += 1;
    inner.tweens.insert(
        id,
        TweenEntry {
            start_instant: Instant::now(),
            duration_ms,
            easing,
            on_tick: Arc::new(on_tick),
            on_complete,
            owner_extension,
        },
    );
    id
}

pub fn cancel(id: TweenId) {
    let mut inner = engine().lock().unwrap_or_else(|e| e.into_inner());
    inner.tweens.remove(&id);
}

pub fn is_running(id: TweenId) -> bool {
    let inner = engine().lock().unwrap_or_else(|e| e.into_inner());
    inner.tweens.contains_key(&id)
}

pub fn cancel_all() {
    let mut inner = engine().lock().unwrap_or_else(|e| e.into_inner());
    inner.tweens.clear();
}

/// Cancel all tweens owned by `ext_id`. Called when a Python extension is unloaded.
pub fn cancel_all_for_extension(ext_id: &str) {
    let mut inner = engine().lock().unwrap_or_else(|e| e.into_inner());
    inner
        .tweens
        .retain(|_, e| e.owner_extension.as_deref() != Some(ext_id));
}

pub fn running_count() -> usize {
    let inner = engine().lock().unwrap_or_else(|e| e.into_inner());
    inner.tweens.len()
}

// ─── Module commands ──────────────────────────────────────────────────────────

fn cmd_list(_args: &[&str], _ctx: &ExecutionContext) -> String {
    let inner = engine().lock().unwrap_or_else(|e| e.into_inner());
    if inner.tweens.is_empty() {
        return "[]".to_string();
    }
    let now = Instant::now();
    let entries: Vec<_> = inner
        .tweens
        .iter()
        .map(|(id, e)| {
            let elapsed_ms = now.duration_since(e.start_instant).as_millis();
            serde_json::json!({
                "id": id.0,
                "elapsed_ms": elapsed_ms,
                "duration_ms": e.duration_ms,
                "easing": e.easing.name(),
                "owner": e.owner_extension.as_deref().unwrap_or("native"),
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
            description: "JSON array of running tweens (id, elapsed_ms, duration_ms, easing, owner)",
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

// ─── Tests ────────────────────────────────────────────────────────────────────

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
    fn lerp_f32_midpoint() {
        assert!((lerp_f32(0.0, 10.0, 0.5) - 5.0).abs() < 0.001);
        assert_eq!(lerp_f32(0.0, 10.0, 0.0), 0.0);
        assert_eq!(lerp_f32(0.0, 10.0, 1.0), 10.0);
    }

    #[test]
    fn lerp_u8_full_range() {
        assert_eq!(lerp_u8(0, 255, 0.0), 0);
        assert_eq!(lerp_u8(0, 255, 1.0), 255);
        assert_eq!(lerp_u8(0, 100, 0.5), 50);
    }

    #[test]
    fn lerp_rgba_identity() {
        let c = [255u8, 128, 64, 200];
        assert_eq!(lerp_rgba(c, c, 0.5), c);
    }

    #[test]
    fn apply_easing_boundary_values() {
        for easing in [
            Easing::Linear,
            Easing::EaseOutCubic,
            Easing::EaseInCubic,
            Easing::EaseInOutCubic,
            Easing::EaseOutElastic,
        ] {
            let t0 = apply_easing(easing, 0.0);
            let t1 = apply_easing(easing, 1.0);
            assert!(
                t0.abs() < 0.001,
                "{:?} at t=0 should be ~0, got {t0}",
                easing
            );
            assert!(
                (t1 - 1.0).abs() < 0.001,
                "{:?} at t=1 should be ~1, got {t1}",
                easing
            );
        }
    }

    #[test]
    fn easing_monotonic_in_midrange() {
        for easing in [Easing::Linear, Easing::EaseOutCubic, Easing::EaseInCubic] {
            let t_mid = apply_easing(easing, 0.5);
            assert!(
                t_mid > 0.0 && t_mid < 1.0,
                "{:?} midpoint {t_mid} must be in (0,1)",
                easing
            );
        }
    }

    #[test]
    fn tween_fires_on_tick() {
        with_clean_state(|| {
            let counter = Arc::new(AtomicU32::new(0));
            let c2 = Arc::clone(&counter);
            // duration_ms = 0 → raw_t = 1.0 immediately (instant-complete path)
            let id = tween(0, Easing::Linear, move |_t| {
                c2.fetch_add(1, Ordering::Relaxed);
            });
            tick();
            assert!(!is_running(id), "tween must complete after elapsed > duration");
            assert!(counter.load(Ordering::Relaxed) > 0, "on_tick must have fired");
        });
    }

    #[test]
    fn cancel_removes_tween() {
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
    fn on_complete_fires_exactly_once() {
        with_clean_state(|| {
            let fired = Arc::new(AtomicU32::new(0));
            let f2 = Arc::clone(&fired);
            tween_with_completion(
                0,
                Easing::Linear,
                |_| {},
                Some(Box::new(move || {
                    f2.fetch_add(1, Ordering::Relaxed);
                })),
                None,
            );
            tick();
            assert_eq!(fired.load(Ordering::Relaxed), 1, "on_complete must fire once");
            tick();
            assert_eq!(
                fired.load(Ordering::Relaxed),
                1,
                "on_complete must not fire again"
            );
        });
    }

    #[test]
    fn easing_from_str_round_trips() {
        for name in [
            "linear",
            "ease_out_cubic",
            "ease_in_cubic",
            "ease_in_out_cubic",
            "ease_out_elastic",
        ] {
            let e = Easing::from_str(name).unwrap_or_else(|| panic!("missing easing: {name}"));
            assert_eq!(e.name(), name);
        }
        assert!(Easing::from_str("bogus").is_none());
    }
}
