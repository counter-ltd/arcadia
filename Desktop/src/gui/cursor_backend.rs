//! Desktop cursor backend: returns the OS-global mouse cursor position and the size of the
//! primary display. Cross-platform via `device_query` + `display-info`.
//!
//! Wired into `arcadia_core::modules::cursor` from desktop startup.
//!
//! macOS note: `device_query::DeviceState::new()` calls `application_is_trusted_with_prompt()`
//! which surfaces the system Accessibility permission prompt and panics if the user has not
//! granted access. We must NOT initialise the underlying `DeviceState` until the user has
//! explicitly granted `cursor.global_position` inside Arcadia's permission settings — otherwise
//! the OS prompt fires on every launch even when no extension/module uses the cursor backend.
//!
//! Strategy:
//!   1. `install()` does nothing more than wire the trait object into `cursor::set_backend`.
//!   2. `DeviceState` is created lazily inside `position()`/`primary_screen_size()`, and only
//!      after `PermissionsConfig::global_allowed("cursor.global_position")` is `true`.
//!   3. We use `DeviceState::checked_new()` (returns `Option`) so a withdrawn grant never panics.

use std::sync::Mutex;

use arcadia_core::config::permissions::PermissionsConfig;
use arcadia_core::config::ConfigFile;
use arcadia_core::modules::cursor::{self, CursorBackend, CursorPosition, ScreenSize};
use device_query::{DeviceQuery, DeviceState};

const CURSOR_GLOBAL_POSITION_PERMISSION: &str = "cursor.global_position";

pub struct DesktopCursorBackend {
    device: Mutex<Option<DeviceState>>,
}

impl DesktopCursorBackend {
    pub fn new() -> Self {
        Self {
            device: Mutex::new(None),
        }
    }

    /// Lazy-init `DeviceState` on first use, gated on the global `cursor.global_position`
    /// permission. Returns `None` if the permission is off or the OS hasn't granted access.
    fn with_device<R>(&self, f: impl FnOnce(&DeviceState) -> R) -> Option<R> {
        let mut guard = self.device.lock().ok()?;
        if guard.is_none() {
            // Refuse to even touch `device_query` unless the user has flipped the
            // `cursor.global_position` global toggle in Arcadia. Without this check the OS
            // Accessibility prompt would appear at launch regardless of user intent.
            let cfg = PermissionsConfig::load_or_create().ok()?;
            if !cfg.global_allowed(CURSOR_GLOBAL_POSITION_PERMISSION) {
                return None;
            }
            // `checked_new()` returns `None` if Accessibility isn't granted by macOS instead of
            // panicking like `DeviceState::new()`.
            *guard = DeviceState::checked_new();
        }
        let dev = guard.as_ref()?;
        Some(f(dev))
    }
}

impl CursorBackend for DesktopCursorBackend {
    fn position(&self) -> Option<CursorPosition> {
        self.with_device(|dev| {
            let mouse = dev.get_mouse();
            CursorPosition {
                x: mouse.coords.0 as f64,
                y: mouse.coords.1 as f64,
            }
        })
    }

    fn primary_screen_size(&self) -> Option<ScreenSize> {
        // `display-info` does not require Accessibility and is safe to call regardless of
        // permission state — keep it independent of `with_device`.
        let infos = display_info::DisplayInfo::all().ok()?;
        let primary = infos
            .iter()
            .find(|d| d.is_primary)
            .or_else(|| infos.first())?;
        Some(ScreenSize {
            width: primary.width,
            height: primary.height,
        })
    }
}

pub fn install() {
    cursor::set_backend(Box::new(DesktopCursorBackend::new()));
}
