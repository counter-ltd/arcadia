//! Shared RGBA sprites for the desktop HUD overlay window (fed by Python via `arcadia`).
//!
//! Multiple extensions may hold sprites simultaneously — each keyed by owner id.
//! Desktop root view reads [`clone_if_newer_than`] inside paint (see `overlay_hud.rs`).
//!
//! A separate vibrancy store tracks owners that have requested native blur (no pixel data).
//! The overlay backend shows/hides the system-edge window's native blur backend based
//! on whether any owner has active vibrancy.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::modules::overlay::OverlayStackingToken;

/// Anchor position for a HUD sprite within the full-screen overlay window.
///
/// `pad_x` is the horizontal offset from the anchor edge;
/// `pad_y` is the vertical offset from the anchor edge.
/// `*Full` variants stretch the sprite to fill the window width — `pad_x` is ignored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpriteAnchor {
    BottomRight,
    BottomLeft,
    BottomCenter,
    BottomFull,
    TopRight,
    TopLeft,
    TopCenter,
    TopFull,
}

impl SpriteAnchor {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s.trim() {
            "bottom-right" | "" => Ok(Self::BottomRight),
            "bottom-left" => Ok(Self::BottomLeft),
            "bottom-center" => Ok(Self::BottomCenter),
            "bottom-full" => Ok(Self::BottomFull),
            "top-right" => Ok(Self::TopRight),
            "top-left" => Ok(Self::TopLeft),
            "top-center" => Ok(Self::TopCenter),
            "top-full" => Ok(Self::TopFull),
            other => Err(format!(
                "Unknown sprite anchor '{other}' (expected bottom-right|bottom-left|bottom-center|bottom-full|top-right|top-left|top-center|top-full)"
            )),
        }
    }

    pub fn is_full(self) -> bool {
        matches!(self, Self::TopFull | Self::BottomFull)
    }

    pub fn is_top(self) -> bool {
        matches!(self, Self::TopRight | Self::TopLeft | Self::TopCenter | Self::TopFull)
    }

    pub fn is_right(self) -> bool {
        matches!(self, Self::TopRight | Self::BottomRight)
    }

    pub fn is_center(self) -> bool {
        matches!(self, Self::TopCenter | Self::BottomCenter)
    }
}

impl Default for SpriteAnchor {
    fn default() -> Self {
        Self::BottomRight
    }
}

#[derive(Clone, Debug)]
pub struct OverlayHudSpritePayload {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub anchor: SpriteAnchor,
    pub stacking: OverlayStackingToken,
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

pub fn has_sprites_for_stacking(stacking: OverlayStackingToken) -> bool {
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

// ── Vibrancy store ────────────────────────────────────────────────────────────
//
// Tracks which extension owners have requested native blur (no pixel data needed).
// When any owner is present the overlay backend enables the native blur/vibrancy backend on the
// SystemEdge window.

struct VibrancyEntry {
    height_px: f32,
    material: Option<String>,
}

struct VibrancyInner {
    version: u64,
    owners: HashMap<String, VibrancyEntry>,
}

static VIBRANCY: OnceLock<Mutex<VibrancyInner>> = OnceLock::new();

fn vibrancy_state() -> &'static Mutex<VibrancyInner> {
    VIBRANCY.get_or_init(|| {
        Mutex::new(VibrancyInner {
            version: 0,
            owners: HashMap::new(),
        })
    })
}

/// Monotonic counter bumped on every [`set_vibrancy_for_owner`] / [`clear_vibrancy_for_owner`].
pub fn vibrancy_version() -> u64 {
    vibrancy_state().lock().map(|g| g.version).unwrap_or(0)
}

/// `true` when at least one owner has active vibrancy.
pub fn has_any_vibrancy() -> bool {
    vibrancy_state()
        .lock()
        .map(|g| !g.owners.is_empty())
        .unwrap_or(false)
}

/// Maximum `height_px` across all active vibrancy owners (0.0 when none).
pub fn vibrancy_height_px() -> f32 {
    vibrancy_state()
        .lock()
        .map(|g| {
            g.owners
                .values()
                .map(|e| e.height_px)
                .fold(0.0_f32, f32::max)
        })
        .unwrap_or(0.0)
}

/// Material string of the owner with the greatest `height_px`, or `None` when no owners active.
pub fn vibrancy_material() -> Option<String> {
    vibrancy_state()
        .lock()
        .ok()
        .and_then(|g| {
            g.owners
                .values()
                .max_by(|a, b| a.height_px.partial_cmp(&b.height_px).unwrap_or(std::cmp::Ordering::Equal))
                .and_then(|e| e.material.clone())
        })
}

/// Register `owner` as wanting native vibrancy at `height_px` logical pixels.
/// Always bumps version so the overlay backend re-applies even if already active
/// (height or material may have changed).
pub fn set_vibrancy_for_owner(owner: String, height_px: f32, material: Option<String>) {
    if let Ok(mut g) = vibrancy_state().lock() {
        g.owners.insert(owner, VibrancyEntry { height_px, material });
        g.version = g.version.wrapping_add(1);
    }
}

/// Remove `owner`'s vibrancy request. No-op if not registered.
pub fn clear_vibrancy_for_owner(owner: &str) {
    if let Ok(mut g) = vibrancy_state().lock() {
        if g.owners.remove(owner).is_some() {
            g.version = g.version.wrapping_add(1);
        }
    }
}
