//! Platform / display information backend.
//!
//! Provides display geometry (screen size, scale factor) and platform-specific UI metadata
//! (e.g. macOS menu bar height). Separated from the cursor module so cursor remains
//! cursor-input-only.
//!
//! A desktop surface installs a [`PlatformBackend`] at startup via [`set_backend`].
//! Headless / iOS without a backend → all queries return `None`.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use crate::config::permissions::SystemGrant;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenSize {
    /// Physical pixel width of the primary display.
    pub width: u32,
    /// Physical pixel height of the primary display.
    pub height: u32,
    /// HiDPI scale factor (e.g. 2.0 on Retina). Logical pixels = physical / scale_factor.
    pub scale_factor: f32,
}

pub trait PlatformBackend: Send + Sync {
    fn primary_screen_size(&self) -> Option<ScreenSize>;
    /// Menu bar height in logical points (macOS only). Returns `None` on other platforms.
    fn menu_bar_height(&self) -> Option<f32> {
        None
    }
    /// On notch-equipped Macs returns `(left_section_end_x, right_section_start_x)` in logical
    /// points, representing the boundary between the app-menu area and the status-item area.
    /// Returns `None` on non-notch Macs and non-macOS platforms.
    fn menu_bar_notch_widths(&self) -> Option<(f32, f32)> {
        None
    }
    /// Returns `(left_frac, right_frac)` — fractions of logical screen width in [0, 1].
    /// Multiply by the caller's logical screen width to get pixel coordinates.
    /// Sources: AX API (any Mac with Accessibility permission) → notch geometry (notch Macs).
    /// Returns `None` when neither source is available.
    fn menu_bar_content_widths(&self) -> Option<(f32, f32)> {
        self.menu_bar_notch_widths()
    }
    /// `true` when Mission Control / Exposé is the active space (CGS space type 4).
    /// Always `false` on non-macOS platforms.
    fn is_mission_control_active(&self) -> bool {
        false
    }
    /// Launch the OS-level grant flow for `grant` (e.g. macOS Accessibility consent dialog
    /// and Settings pane). Returns `true` if the backend handled it.
    fn prompt_system_grant(&self, _grant: SystemGrant) -> bool {
        false
    }
    /// `true` when the OS-level grant for `grant` is currently active (e.g. macOS Accessibility
    /// permission is granted to this process). `false` headless / unsupported.
    fn is_system_grant_active(&self, _grant: SystemGrant) -> bool {
        false
    }
}

// ── Space-change notification callbacks ───────────────────────────────────────

fn space_handlers() -> &'static Mutex<HashMap<String, Arc<dyn Fn() + Send + Sync>>> {
    static H: OnceLock<Mutex<HashMap<String, Arc<dyn Fn() + Send + Sync>>>> = OnceLock::new();
    H.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn register_space_change_handler(owner: String, cb: Arc<dyn Fn() + Send + Sync>) {
    if let Ok(mut m) = space_handlers().lock() {
        m.insert(owner, cb);
    }
}

pub fn unregister_space_change_handler(owner: &str) {
    if let Ok(mut m) = space_handlers().lock() {
        m.remove(owner);
    }
}

/// Called by the platform backend when a space transition is detected (e.g. Mission Control
/// open/close). Clones handler Arcs before calling to avoid holding the lock across callbacks.
pub fn fire_space_change_handlers() {
    let cbs: Vec<Arc<dyn Fn() + Send + Sync>> = {
        let Ok(m) = space_handlers().lock() else { return };
        m.values().cloned().collect()
    };
    for cb in cbs {
        cb();
    }
}

fn backend_slot() -> &'static Mutex<Option<Box<dyn PlatformBackend>>> {
    static B: OnceLock<Mutex<Option<Box<dyn PlatformBackend>>>> = OnceLock::new();
    B.get_or_init(|| Mutex::new(None))
}

pub fn set_backend(b: Box<dyn PlatformBackend>) {
    if let Ok(mut slot) = backend_slot().lock() {
        *slot = Some(b);
    }
}

pub fn primary_screen_size() -> Option<ScreenSize> {
    backend_slot()
        .lock()
        .ok()
        .and_then(|slot| slot.as_ref().and_then(|b| b.primary_screen_size()))
}

pub fn menu_bar_height() -> Option<f32> {
    backend_slot()
        .lock()
        .ok()
        .and_then(|slot| slot.as_ref().and_then(|b| b.menu_bar_height()))
}

/// On notch-equipped Macs returns `(left_section_end_x, right_section_start_x)` in logical
/// points. Returns `None` on non-notch Macs and non-macOS platforms.
pub fn menu_bar_notch_widths() -> Option<(f32, f32)> {
    backend_slot()
        .lock()
        .ok()
        .and_then(|slot| slot.as_ref().and_then(|b| b.menu_bar_notch_widths()))
}

// Set by the platform backend notification observer when expose.awake / expose.willStop fires.
// Reading is lock-free; writing happens from a single background thread.
static MC_ACTIVE: AtomicBool = AtomicBool::new(false);

pub fn set_mission_control_active(active: bool) {
    MC_ACTIVE.store(active, Ordering::Relaxed);
}

/// `true` while Mission Control / Exposé overlay is active.
/// State is driven by `expose.awake` / `expose.willStop` distributed notifications —
/// no polling, no CGS SPI timing dependency.
pub fn is_mission_control_active() -> bool {
    MC_ACTIVE.load(Ordering::Relaxed)
}

/// Returns `(left_frac, right_frac)` in [0, 1] via AX or notch geometry.
/// Multiply by logical screen width to get pixel coordinates. `None` when no source available.
pub fn menu_bar_content_widths() -> Option<(f32, f32)> {
    backend_slot()
        .lock()
        .ok()
        .and_then(|slot| slot.as_ref().and_then(|b| b.menu_bar_content_widths()))
}

/// Launch the OS-level grant flow for `grant` (e.g. the macOS Accessibility consent dialog and
/// Settings pane). Returns `true` if a backend handled it; `false` headless / unsupported.
pub fn prompt_system_grant(grant: SystemGrant) -> bool {
    backend_slot()
        .lock()
        .ok()
        .map(|slot| {
            slot.as_ref()
                .map(|b| b.prompt_system_grant(grant))
                .unwrap_or(false)
        })
        .unwrap_or(false)
}

/// `true` when the OS-level grant for `grant` is currently active. `false` headless / unsupported.
pub fn is_system_grant_active(grant: SystemGrant) -> bool {
    backend_slot()
        .lock()
        .ok()
        .map(|slot| {
            slot.as_ref()
                .map(|b| b.is_system_grant_active(grant))
                .unwrap_or(false)
        })
        .unwrap_or(false)
}
