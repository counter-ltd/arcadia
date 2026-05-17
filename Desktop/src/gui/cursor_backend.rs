//! Desktop cursor backend: OS-global mouse cursor position and button state only.
//! Cross-platform via `device_query`.
//!
//! Wired into `arcadia_core::modules::cursor` from desktop startup via [`install`].
//!
//! Display geometry and platform metadata live in [`super::platform_backend`].
//!
//! macOS note: `device_query::DeviceState::new()` calls `application_is_trusted_with_prompt()`
//! which surfaces the system Accessibility permission prompt and panics if the user has not
//! granted access. We must NOT initialise the underlying `DeviceState` until the user has turned
//! on at least one of `cursor.global_position` or `cursor.global_mouse_buttons` in Arcadia's
//! permission settings — otherwise the OS prompt fires on every launch even when no
//! extension/module uses the cursor backend.
//!
//! Strategy:
//!   1. `install()` does nothing more than wire the trait object into `cursor::set_backend`.
//!   2. `DeviceState` is created lazily inside `position()` / `snapshot()`, and only after
//!      `PermissionsConfig::global_allowed("cursor.global_position")` or
//!      `global_allowed("cursor.global_mouse_buttons")` is `true`.
//!   3. We use `DeviceState::checked_new()` (returns `Option`) so a withdrawn grant never panics.

use std::sync::Mutex;

use arcadia_core::config::permissions::PermissionsConfig;
use arcadia_core::config::ConfigFile;
use arcadia_core::modules::cursor::{self, CursorBackend, CursorPosition, CursorSnapshot};
use device_query::{DeviceQuery, DeviceState};

const CURSOR_GLOBAL_POSITION_PERMISSION: &str = "cursor.global_position";
const CURSOR_GLOBAL_MOUSE_BUTTONS_PERMISSION: &str = "cursor.global_mouse_buttons";

fn cursor_os_input_global_enabled() -> bool {
    let Ok(cfg) = PermissionsConfig::load_or_create() else {
        return false;
    };
    cfg.global_allowed(CURSOR_GLOBAL_POSITION_PERMISSION)
        || cfg.global_allowed(CURSOR_GLOBAL_MOUSE_BUTTONS_PERMISSION)
}

pub struct DesktopCursorBackend {
    device: Mutex<Option<DeviceState>>,
}

impl DesktopCursorBackend {
    pub fn new() -> Self {
        Self {
            device: Mutex::new(None),
        }
    }

    /// Lazy-init `DeviceState` on first use, gated on global cursor input permissions.
    /// Returns `None` if both globals are off or the OS hasn't granted access.
    fn with_device<R>(&self, f: impl FnOnce(&DeviceState) -> R) -> Option<R> {
        let mut guard = self.device.lock().ok()?;
        if guard.is_none() {
            // Refuse to touch `device_query` unless the user enabled at least one global
            // cursor-input permission. Without this check the OS Accessibility prompt would
            // appear at launch regardless of user intent.
            if !cursor_os_input_global_enabled() {
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
        self.snapshot().map(|s| CursorPosition { x: s.x, y: s.y })
    }

    fn snapshot(&self) -> Option<CursorSnapshot> {
        self.with_device(|dev| {
            let m = dev.get_mouse();
            CursorSnapshot {
                x: m.coords.0 as f64,
                y: m.coords.1 as f64,
                left_button: m.button_pressed.get(1).copied().unwrap_or(false),
                right_button: m.button_pressed.get(2).copied().unwrap_or(false),
            }
        })
    }
}

pub fn install() {
    cursor::set_backend(Box::new(DesktopCursorBackend::new()));
}
