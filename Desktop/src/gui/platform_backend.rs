//! Desktop platform backend: primary display size + macOS UI metadata (menu bar height).
//!
//! Uses `display-info` for cross-platform display geometry (no Accessibility permission needed)
//! and `NSScreen` on macOS for the menu bar height derived from `visibleFrame`.
//!
//! Wired into `arcadia_core::modules::platform` from desktop startup via [`install`].

use arcadia_core::modules::platform::{self, PlatformBackend, ScreenSize};

pub struct DesktopPlatformBackend;

impl PlatformBackend for DesktopPlatformBackend {
    fn primary_screen_size(&self) -> Option<ScreenSize> {
        let infos = display_info::DisplayInfo::all().ok()?;
        let primary = infos
            .iter()
            .find(|d| d.is_primary)
            .or_else(|| infos.first())?;
        Some(ScreenSize {
            width: primary.width,
            height: primary.height,
            scale_factor: primary.scale_factor,
        })
    }

    fn menu_bar_height(&self) -> Option<f32> {
        #[cfg(target_os = "macos")]
        {
            use objc2_app_kit::NSStatusBar;
            // NSStatusBar::systemStatusBar() is thread-safe (no MainThreadMarker required).
            // thickness() returns CGFloat in logical points — the authoritative menu bar height.
            let h = NSStatusBar::systemStatusBar().thickness() as f32;
            if h > 0.0 { Some(h) } else { None }
        }
        #[cfg(not(target_os = "macos"))]
        {
            None
        }
    }
}

pub fn install() {
    platform::set_backend(Box::new(DesktopPlatformBackend));
}
