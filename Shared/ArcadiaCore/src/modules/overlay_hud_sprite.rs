//! Shared RGBA sprites for the desktop HUD overlay window (fed by Python via `arcadia`).
//!
//! Multiple extensions may hold sprites simultaneously — each keyed by owner id.
//! Desktop root view reads [`clone_if_newer_than`] inside paint (see `overlay_hud.rs`).

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Anchor positions for a HUD sprite within the full-screen overlay window.
///
/// `pad_x` is the offset from the anchor's horizontal edge;
/// `pad_y` is the offset from the anchor's vertical edge.
/// For `*-full` variants the sprite stretches to fill the window width — `pad_x` is ignored.
///
/// Valid values: `"bottom-right"` (default) · `"bottom-left"` · `"bottom-center"` · `"bottom-full"`
///               `"top-right"` · `"top-left"` · `"top-center"` · `"top-full"`
#[derive(Clone, Debug)]
pub struct OverlayHudSpritePayload {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub anchor: String,
    pub stacking: String,
    pub pad_x: f32,
    pub pad_y: f32,
    pub display_width: Option<f32>,
    pub display_height: Option<f32>,
}

struct Inner {
    version: u64,
    sprites: HashMap<String, OverlayHudSpritePayload>,
}

static STATE: OnceLock<Mutex<Inner>> = OnceLock::new();

fn state() -> &'static Mutex<Inner> {
    STATE.get_or_init(|| {
        Mutex::new(Inner {
            version: 0,
            sprites: HashMap::new(),
        })
    })
}

/// Monotonic counter bumped on every [`set_sprite`] / [`clear_sprite`] call.
pub fn version() -> u64 {
    state().lock().map(|g| g.version).unwrap_or(0)
}

/// True when at least one owner has an active sprite.
pub fn has_sprites() -> bool {
    state()
        .lock()
        .map(|g| !g.sprites.is_empty())
        .unwrap_or(false)
}

pub fn has_sprites_for_stacking(stacking: &str) -> bool {
    state()
        .lock()
        .map(|g| g.sprites.values().any(|p| p.stacking == stacking))
        .unwrap_or(false)
}

pub fn set_sprite(owner: String, payload: OverlayHudSpritePayload) -> Result<(), String> {
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
    let mut g = state()
        .lock()
        .map_err(|_| "overlay sprite: state lock poisoned".to_string())?;
    g.sprites.insert(owner, payload);
    g.version = g.version.wrapping_add(1);
    Ok(())
}

/// Clear all sprites regardless of owner.
pub fn clear_sprite() {
    if let Ok(mut g) = state().lock() {
        g.sprites.clear();
        g.version = g.version.wrapping_add(1);
    }
}

/// Clear only the sprite owned by `owner`. No-op if owner has no sprite.
pub fn clear_sprite_for_owner(owner: &str) {
    if let Ok(mut g) = state().lock() {
        if g.sprites.remove(owner).is_some() {
            g.version = g.version.wrapping_add(1);
        }
    }
}

/// If [`version()`] differs from `since`, returns new version and a clone of all sprites.
pub fn clone_if_newer_than(
    since: u64,
) -> Option<(u64, HashMap<String, OverlayHudSpritePayload>)> {
    let Ok(g) = state().lock() else {
        return None;
    };
    if g.version == since {
        return None;
    }
    Some((g.version, g.sprites.clone()))
}
