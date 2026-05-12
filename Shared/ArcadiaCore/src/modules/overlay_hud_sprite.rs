//! Shared RGBA sprite for the desktop HUD overlay window (fed by Python via `arcadia`).
//!
//! Desktop root view reads [`clone_if_newer_than`] inside paint (see `overlay_hud.rs`).

use std::sync::Mutex;

#[derive(Clone, Debug)]
pub struct OverlayHudSpritePayload {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub pad_right: f32,
    pub pad_bottom: f32,
    pub display_width: Option<f32>,
    pub display_height: Option<f32>,
}

struct Inner {
    version: u64,
    sprite: Option<OverlayHudSpritePayload>,
}

impl Inner {
    const fn new() -> Self {
        Self {
            version: 0,
            sprite: None,
        }
    }
}

static STATE: Mutex<Inner> = Mutex::new(Inner::new());

/// Monotonic counter bumped on every [`set_sprite`] / [`clear_sprite`].
pub fn version() -> u64 {
    STATE
        .lock()
        .map(|g| g.version)
        .unwrap_or(0)
}

pub fn set_sprite(payload: OverlayHudSpritePayload) -> Result<(), String> {
    let expected = (payload.width as usize)
        .checked_mul(payload.height as usize)
        .and_then(|n| n.checked_mul(4))
        .ok_or_else(|| "overlay sprite: dimension overflow".to_string())?;
    if payload.rgba.len() != expected {
        return Err(format!(
            "overlay sprite: rgba length {} != width*height*4 ({expected})",
            payload.rgba.len()
        ));
    }
    let mut g = STATE
        .lock()
        .map_err(|_| "overlay sprite: state lock poisoned".to_string())?;
    g.sprite = Some(payload);
    g.version = g.version.wrapping_add(1);
    Ok(())
}

pub fn clear_sprite() {
    if let Ok(mut g) = STATE.lock() {
        g.sprite = None;
        g.version = g.version.wrapping_add(1);
    }
}

/// Latest `(version, sprite)` — clones RGBA when present.
pub fn snapshot() -> (u64, Option<OverlayHudSpritePayload>) {
    let Ok(g) = STATE.lock() else {
        return (0, None);
    };
    (g.version, g.sprite.clone())
}

/// If [`version()`] differs from `since`, returns new version and a clone of the payload.
pub fn clone_if_newer_than(since: u64) -> Option<(u64, Option<OverlayHudSpritePayload>)> {
    let Ok(g) = STATE.lock() else {
        return None;
    };
    if g.version == since {
        return None;
    }
    Some((g.version, g.sprite.clone()))
}
