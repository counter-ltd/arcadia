//! Platform / display information backend.
//!
//! Provides display geometry (screen size, scale factor) and platform-specific UI metadata
//! (e.g. macOS menu bar height). Separated from the cursor module so cursor remains
//! cursor-input-only.
//!
//! A desktop surface installs a [`PlatformBackend`] at startup via [`set_backend`].
//! Headless / iOS without a backend → all queries return `None`.

use std::sync::{Mutex, OnceLock};

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
