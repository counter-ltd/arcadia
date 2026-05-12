//! Wires `arcadia_core::modules::overlay` to the OpenFrame overlay window on the GUI thread.
//!
//! Visibility updates are driven from the same `AsyncApp` pump as tray scheduling (see
//! [`crate::gui::app::entry::spawn_main_thread_pump`]) so we never store `AsyncApp` in a `Send`
//! mutex. Closing the main Arcadia window tears down the `App`, which drops all windows including
//! this overlay. macOS notch/menu-bar stacking may need a higher window level later.
//!
//! HUD **sprite** updates are **not** applied via `AsyncApp::update` — the overlay root view pulls
//! [`arcadia_core::modules::overlay_hud_sprite`] inside [`crate::gui::overlay_hud::OverlayHudRoot::render`]
//! to avoid nested `App::borrow_mut` when the timer pump overlaps an in-flight UI update.

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};
use std::sync::Mutex;

use arcadia_core::modules::overlay::{self, OverlayBackend, OverlayStackingToken};
use arcadia_core::modules::overlay_hud_sprite;
use openframe::{AnyWindowHandle, AsyncApp, WindowStacking};

static OVERLAY_HANDLE: Mutex<Option<AnyWindowHandle>> = Mutex::new(None);
static OVERLAY_VISIBLE: AtomicBool = AtomicBool::new(false);
static OVERLAY_DIRTY: AtomicBool = AtomicBool::new(false);

const STACK_PENDING_NONE: u8 = 0;
const STACK_PENDING_NORMAL: u8 = 1;
const STACK_PENDING_FLOATING: u8 = 2;
const STACK_PENDING_HUD: u8 = 3;
const STACK_PENDING_SYSTEM_UI: u8 = 4;

static OVERLAY_STACKING_PENDING: AtomicU8 = AtomicU8::new(STACK_PENDING_NONE);
static OVERLAY_STACKING_DIRTY: AtomicBool = AtomicBool::new(false);

/// Last [`overlay_hud_sprite::version`] we requested a full window refresh for.
static SPRITE_REFRESHED_AT_VERSION: AtomicU64 = AtomicU64::new(u64::MAX);

struct DesktopOverlayBackendThunk;

impl OverlayBackend for DesktopOverlayBackendThunk {
    fn set_visible(&self, visible: bool) {
        OVERLAY_VISIBLE.store(visible, Ordering::Release);
        OVERLAY_DIRTY.store(true, Ordering::Release);
    }

    fn is_visible(&self) -> bool {
        OVERLAY_VISIBLE.load(Ordering::Relaxed)
    }

    fn set_stacking(&self, token: OverlayStackingToken) -> Result<(), String> {
        let code = match token {
            OverlayStackingToken::Normal => STACK_PENDING_NORMAL,
            OverlayStackingToken::Floating => STACK_PENDING_FLOATING,
            OverlayStackingToken::Hud => STACK_PENDING_HUD,
            OverlayStackingToken::SystemUi => STACK_PENDING_SYSTEM_UI,
        };
        OVERLAY_STACKING_PENDING.store(code, Ordering::Release);
        OVERLAY_STACKING_DIRTY.store(true, Ordering::Release);
        Ok(())
    }
}

fn pending_u8_to_openframe(code: u8) -> Option<(WindowStacking, &'static str)> {
    Some(match code {
        STACK_PENDING_NORMAL => (WindowStacking::Normal, "normal"),
        STACK_PENDING_FLOATING => (WindowStacking::Floating, "floating"),
        STACK_PENDING_HUD => (WindowStacking::Hud, "hud"),
        STACK_PENDING_SYSTEM_UI => (WindowStacking::SystemUi, "system_ui"),
        _ => return None,
    })
}

/// Register the core backend before opening windows; then call [`register_overlay_window`].
pub fn init_overlay_module() {
    overlay::set_backend(Box::new(DesktopOverlayBackendThunk));
}

pub fn register_overlay_window(handle: openframe::WindowHandle<super::overlay_hud::OverlayHudRoot>) {
    let any: AnyWindowHandle = handle.into();
    if let Ok(mut g) = OVERLAY_HANDLE.lock() {
        *g = Some(any);
    }
    SPRITE_REFRESHED_AT_VERSION.store(u64::MAX, Ordering::Release);
    overlay::set_stacking_status_label("hud");
}

/// Apply pending `overlay.show` / `overlay.hide` / `overlay.set-stacking` from the foreground `AsyncApp` pump.
pub fn poll_overlay(async_app: &mut AsyncApp) {
    poll_overlay_visibility(async_app);
    poll_overlay_stacking(async_app);
    poll_overlay_sprite_refresh(async_app);
}

fn poll_overlay_stacking(async_app: &mut AsyncApp) {
    if !OVERLAY_STACKING_DIRTY.swap(false, Ordering::AcqRel) {
        return;
    }
    let code = OVERLAY_STACKING_PENDING.load(Ordering::Relaxed);
    let Some((stacking, label)) = pending_u8_to_openframe(code) else {
        return;
    };
    let handle = match OVERLAY_HANDLE.lock() {
        Ok(g) => *g,
        Err(_) => return,
    };
    let Some(handle) = handle else {
        return;
    };
    let _ = async_app.update(move |app| {
        let _ = handle.update(app, |_root, window, _| -> Result<(), ()> {
            window.set_stacking(stacking);
            Ok(())
        });
    });
    overlay::set_stacking_status_label(label);
}

/// Apply pending `overlay.show` / `overlay.hide` from the foreground `AsyncApp` pump.
pub fn poll_overlay_visibility(async_app: &mut AsyncApp) {
    if !OVERLAY_DIRTY.swap(false, Ordering::AcqRel) {
        return;
    }
    let visible = OVERLAY_VISIBLE.load(Ordering::Relaxed);
    let handle = match OVERLAY_HANDLE.lock() {
        Ok(g) => *g,
        Err(_) => return,
    };
    let Some(handle) = handle else {
        return;
    };
    let _ = async_app.update(move |app| {
        let _ = handle.update(app, |_root, window, _| -> Result<(), ()> {
            window.set_visibility(visible);
            Ok(())
        });
    });
}

/// When Python (or any thread) bumps the HUD sprite revision, schedule a redraw. Uses
/// [`AsyncApp::refresh`] only (no root `update`) to reduce nested `App::borrow_mut` pressure vs
/// pushing pixels through `WindowHandle::update`.
fn poll_overlay_sprite_refresh(async_app: &AsyncApp) {
    let v = overlay_hud_sprite::version();
    let prev = SPRITE_REFRESHED_AT_VERSION.load(Ordering::Acquire);
    if v == prev {
        return;
    }
    SPRITE_REFRESHED_AT_VERSION.store(v, Ordering::Release);
    let _ = async_app.refresh();
}
