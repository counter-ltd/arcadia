//! Wires `arcadia_core::modules::overlay` to two OpenFrame overlay windows on the GUI thread.
//!
//! Two windows are managed:
//!   - HUD (`WindowStacking::Hud`, level 101) — general-purpose overlays, pets, etc.
//!     Visibility is controlled by `overlay.show` / `overlay.hide` commands.
//!   - Below-menu-bar (`WindowStacking::BelowMenuBar`, level 24) — bar backgrounds.
//!     Fully auto-managed: shows when any sprite with stacking="below_menu_bar" exists,
//!     hides when the last one is cleared.
//!     Suppressed when a fullscreen application is active: in fullscreen spaces, the app
//!     menu items (left side) are drawn at a lower window level than our overlay, so they
//!     would appear behind the gradient.  The status-bar items (right side) remain visible,
//!     but covering the app menus makes the menu bar unusable.
//!
//! Sprite updates are pulled inside `OverlayHudRoot::render` — no `AsyncApp::update` for pixels.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;

use arcadia_core::modules::overlay::{self, OverlayBackend, OverlayStackingToken};
use arcadia_core::modules::overlay_hud_sprite;
use arcadia_core::modules::platform as core_platform;
use openframe::{AnyWindowHandle, AsyncApp};

// ── HUD window ────────────────────────────────────────────────────────────────
static OVERLAY_HANDLE_HUD: Mutex<Option<AnyWindowHandle>> = Mutex::new(None);
static OVERLAY_VISIBLE_HUD: AtomicBool = AtomicBool::new(false);
static OVERLAY_DIRTY_HUD: AtomicBool = AtomicBool::new(false);

// ── Below-menu-bar window ─────────────────────────────────────────────────────
static OVERLAY_HANDLE_BMB: Mutex<Option<AnyWindowHandle>> = Mutex::new(None);
static OVERLAY_VISIBLE_BMB: AtomicBool = AtomicBool::new(false);
static OVERLAY_DIRTY_BMB: AtomicBool = AtomicBool::new(false);

/// Last [`overlay_hud_sprite::version`] we requested a full window refresh for.
static SPRITE_REFRESHED_AT_VERSION: AtomicU64 = AtomicU64::new(u64::MAX);

/// Last [`overlay_hud_sprite::vibrancy_version`] we applied vibrancy state for.
static VIBRANCY_REFRESHED_AT_VERSION: AtomicU64 = AtomicU64::new(u64::MAX);

/// Whether the BMB window currently has vibrancy (`NSVisualEffectView`) active.
static OVERLAY_VIBRANCY_BMB: AtomicBool = AtomicBool::new(false);

/// Tracks the last-seen fullscreen state so we can detect transitions and dirty the BMB flag.
static LAST_FULLSCREEN_STATE: AtomicBool = AtomicBool::new(false);

/// Returns `true` when the frontmost application is running in a fullscreen space.
///
/// Uses `[NSApplication currentSystemPresentationOptions]` which reflects the active
/// application's presentation options.  `NSApplicationPresentationFullScreen` (1 << 10)
/// is set while any app occupies a fullscreen space.
#[cfg(target_os = "macos")]
fn is_any_display_fullscreen() -> bool {
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSApplication, NSApplicationPresentationOptions};
    // SAFETY: poll_overlay_visibility is only ever called from the GUI (main) thread
    // via the AsyncApp pump, so MainThreadMarker::new_unchecked() is sound here.
    unsafe {
        let mtm = MainThreadMarker::new_unchecked();
        let app = NSApplication::sharedApplication(mtm);
        let opts = app.currentSystemPresentationOptions();
        opts.contains(NSApplicationPresentationOptions::FullScreen)
    }
}

#[cfg(not(target_os = "macos"))]
fn is_any_display_fullscreen() -> bool {
    false
}

struct DesktopOverlayBackendThunk;

impl OverlayBackend for DesktopOverlayBackendThunk {
    fn set_visible(&self, visible: bool) {
        // overlay.show / overlay.hide affect the HUD window only (backward compat).
        OVERLAY_VISIBLE_HUD.store(visible, Ordering::Release);
        OVERLAY_DIRTY_HUD.store(true, Ordering::Release);
    }

    fn is_visible(&self) -> bool {
        OVERLAY_VISIBLE_HUD.load(Ordering::Relaxed)
    }

    fn set_stacking(&self, _token: OverlayStackingToken) -> Result<(), String> {
        // Stacking is fixed at window creation time — no-op.
        Ok(())
    }
}

/// Register the core backend before opening windows; then call [`register_overlay_window`].
pub fn init_overlay_module() {
    overlay::set_backend(Box::new(DesktopOverlayBackendThunk));
}

/// Store a window handle in the appropriate slot based on `stacking`.
pub fn register_overlay_window(
    handle: openframe::WindowHandle<super::overlay_hud::OverlayHudRoot>,
    stacking: OverlayStackingToken,
) {
    let any: AnyWindowHandle = handle.into();
    if stacking == OverlayStackingToken::BelowMenuBar {
        if let Ok(mut g) = OVERLAY_HANDLE_BMB.lock() {
            *g = Some(any);
        }
    } else {
        if let Ok(mut g) = OVERLAY_HANDLE_HUD.lock() {
            *g = Some(any);
        }
        SPRITE_REFRESHED_AT_VERSION.store(u64::MAX, Ordering::Release);
        overlay::set_stacking_status_label(OverlayStackingToken::Hud.as_str());
    }
}

/// Apply pending visibility and sprite-refresh work from the foreground `AsyncApp` pump.
pub fn poll_overlay(async_app: &mut AsyncApp) {
    poll_overlay_visibility(async_app);
    poll_overlay_sprite_refresh(async_app);
    poll_overlay_vibrancy(async_app);
}

fn poll_overlay_visibility(async_app: &mut AsyncApp) {
    if OVERLAY_DIRTY_HUD.swap(false, Ordering::AcqRel) {
        let visible = OVERLAY_VISIBLE_HUD.load(Ordering::Relaxed);
        let handle = OVERLAY_HANDLE_HUD.lock().ok().and_then(|g| *g);
        if let Some(handle) = handle {
            let _ = async_app.update(move |app| {
                let _ = handle.update(app, |_root, window, _| -> Result<(), ()> {
                    window.set_visibility(visible);
                    Ok(())
                });
            });
        }
    }

    // Detect fullscreen transitions: if the state changed, dirty the BMB flag so
    // the visibility update below will run even if the sprite set hasn't changed.
    let fullscreen_now = is_any_display_fullscreen();
    let fullscreen_prev = LAST_FULLSCREEN_STATE.swap(fullscreen_now, Ordering::AcqRel);
    if fullscreen_now != fullscreen_prev {
        OVERLAY_DIRTY_BMB.store(true, Ordering::Release);
    }

    if OVERLAY_DIRTY_BMB.swap(false, Ordering::AcqRel) {
        // Suppress the BMB overlay while any fullscreen app is active.
        let visible = OVERLAY_VISIBLE_BMB.load(Ordering::Relaxed) && !fullscreen_now;
        let handle = OVERLAY_HANDLE_BMB.lock().ok().and_then(|g| *g);
        if let Some(handle) = handle {
            let _ = async_app.update(move |app| {
                let _ = handle.update(app, |_root, window, _| -> Result<(), ()> {
                    window.set_visibility(visible);
                    Ok(())
                });
            });
        }
    }
}

/// When sprite revision changes, auto-manage window visibility and schedule a redraw.
///
/// HUD: auto-hides when all HUD sprites are cleared.
/// BMB: auto-shows when first BMB sprite or vibrancy request appears;
///      auto-hides when both are absent.
fn poll_overlay_sprite_refresh(async_app: &AsyncApp) {
    let v = overlay_hud_sprite::version();
    let prev = SPRITE_REFRESHED_AT_VERSION.load(Ordering::Acquire);
    if v == prev {
        return;
    }
    SPRITE_REFRESHED_AT_VERSION.store(v, Ordering::Release);

    if !overlay_hud_sprite::has_sprites_for_stacking(OverlayStackingToken::Hud) {
        OVERLAY_VISIBLE_HUD.store(false, Ordering::Release);
        OVERLAY_DIRTY_HUD.store(true, Ordering::Release);
    }

    let bmb_has = overlay_hud_sprite::has_sprites_for_stacking(OverlayStackingToken::BelowMenuBar)
        || overlay_hud_sprite::has_any_vibrancy();
    let bmb_visible = OVERLAY_VISIBLE_BMB.load(Ordering::Relaxed);
    if bmb_has && !bmb_visible {
        OVERLAY_VISIBLE_BMB.store(true, Ordering::Release);
        OVERLAY_DIRTY_BMB.store(true, Ordering::Release);
    } else if !bmb_has && bmb_visible {
        OVERLAY_VISIBLE_BMB.store(false, Ordering::Release);
        OVERLAY_DIRTY_BMB.store(true, Ordering::Release);
    }

    let _ = async_app.refresh();
}

/// Toggle `NSVisualEffectView` sections on the BMB window when the vibrancy owner set changes.
fn poll_overlay_vibrancy(async_app: &mut AsyncApp) {
    let v = overlay_hud_sprite::vibrancy_version();
    let prev = VIBRANCY_REFRESHED_AT_VERSION.load(Ordering::Acquire);
    if v == prev {
        return;
    }
    VIBRANCY_REFRESHED_AT_VERSION.store(v, Ordering::Release);

    let want_vibrancy = overlay_hud_sprite::has_any_vibrancy();
    OVERLAY_VIBRANCY_BMB.store(want_vibrancy, Ordering::Release);

    // Resolve logical screen width for None x_start/x_end entries.
    let screen_w_logical = core_platform::primary_screen_size()
        .map(|s| s.width as f32 / s.scale_factor)
        .unwrap_or(1280.0);

    // Build concrete sections: resolve None edges, sort by x_start.
    let mut sections: Vec<(f32, f32, f32, String)> = overlay_hud_sprite::vibrancy_sections()
        .into_iter()
        .map(|(x0, x1, h, mat)| {
            (
                x0.unwrap_or(0.0),
                x1.unwrap_or(screen_w_logical),
                h,
                mat.unwrap_or_else(|| "sidebar".to_string()),
            )
        })
        .collect();
    sections.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let handle = OVERLAY_HANDLE_BMB.lock().ok().and_then(|g| *g);
    if let Some(handle) = handle {
        let _ = async_app.update(move |app| {
            let _ = handle.update(app, |_root, window, _| -> Result<(), ()> {
                window.set_vibrancy_sections(&sections);
                Ok(())
            });
        });
    }

    // Ensure BMB window visibility matches combined state.
    let bmb_has =
        overlay_hud_sprite::has_sprites_for_stacking(OverlayStackingToken::BelowMenuBar) || want_vibrancy;
    let bmb_visible = OVERLAY_VISIBLE_BMB.load(Ordering::Relaxed);
    if bmb_has != bmb_visible {
        OVERLAY_VISIBLE_BMB.store(bmb_has, Ordering::Release);
        OVERLAY_DIRTY_BMB.store(true, Ordering::Release);
    }
}
