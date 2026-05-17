//! Root view for the shared HUD overlay window (transparent; multi-sprite, any anchor).
//!
//! Each extension registers its sprite under its own owner key. All active sprites are
//! composited as full-screen flex layers stacked via absolute positioning — the same flex
//! approach used before multi-sprite support, one transparent layer per owner.
//!
//! Sprite pixels are pulled from [`arcadia_core::modules::overlay_hud_sprite`] inside
//! [`Render::render`] on the UI thread so we never call [`openframe::AsyncApp::update`] for
//! sprite changes (avoids re-entrant `App` `borrow_mut` deadlock with the foreground timer loop).

use std::collections::HashMap;
use std::sync::Arc;

use arcadia_core::modules::overlay_hud_sprite;
use image::{Frame, ImageBuffer, Rgba};
use openframe::{div, img, prelude::*, px, IntoElement, Render, RenderImage, Window};

struct CachedSprite {
    image: Arc<RenderImage>,
    anchor: String,
    pad_x: f32,
    pad_y: f32,
    display_width: Option<f32>,
    display_height: Option<f32>,
}

pub struct OverlayHudRoot {
    last_sprite_version: u64,
    sprites: HashMap<String, CachedSprite>,
    stacking_filter: String,
}

impl OverlayHudRoot {
    pub fn new(_cx: &mut Context<Self>, stacking_filter: &str) -> Self {
        Self {
            last_sprite_version: u64::MAX,
            sprites: HashMap::new(),
            stacking_filter: stacking_filter.to_string(),
        }
    }

    fn sync_sprites_from_core(&mut self) {
        let Some((ver, payloads)) =
            overlay_hud_sprite::clone_if_newer_than(self.last_sprite_version)
        else {
            return;
        };
        self.last_sprite_version = ver;
        self.sprites.clear();
        for (owner, p) in payloads {
            if p.stacking != self.stacking_filter {
                continue;
            }
            let Some(image) = render_image_from_rgba(p.rgba, p.width, p.height) else {
                continue;
            };
            self.sprites.insert(
                owner,
                CachedSprite {
                    image,
                    anchor: p.anchor,
                    pad_x: p.pad_x,
                    pad_y: p.pad_y,
                    display_width: p.display_width,
                    display_height: p.display_height,
                },
            );
        }
    }
}

/// Build GPU image from straight RGBA (swaps to BGRA like the asset pipeline).
pub fn render_image_from_rgba(
    mut rgba: Vec<u8>,
    width: u32,
    height: u32,
) -> Option<Arc<RenderImage>> {
    for px in rgba.chunks_exact_mut(4) {
        px.swap(0, 2);
    }
    let buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(width, height, rgba)?;
    Some(Arc::new(RenderImage::new(vec![Frame::new(buffer)])))
}

/// Render one sprite as a full-screen transparent flex layer positioned at its anchor.
fn sprite_layer(s: &CachedSprite) -> impl IntoElement {
    let anchor = s.anchor.as_str();
    let pad_x = s.pad_x;
    let pad_y = s.pad_y;

    let (iw, ih) = {
        let sz = s.image.size(0);
        (sz.width.0 as f32, sz.height.0 as f32)
    };

    let is_full = matches!(anchor, "top-full" | "bottom-full");
    let is_top = anchor.starts_with("top");
    let is_right = matches!(anchor, "top-right" | "bottom-right");
    let is_center = matches!(anchor, "top-center" | "bottom-center");

    let dw = s.display_width.unwrap_or(iw);
    let dh = s.display_height.unwrap_or(ih);

    // Inner element: full-width image for *-full anchors, fixed-size otherwise.
    let inner = if is_full {
        div()
            .flex_none()
            .w_full()
            .child(img(s.image.clone()).w_full().h(px(dh)))
    } else {
        div()
            .flex_none()
            .child(img(s.image.clone()).w(px(dw)).h(px(dh)))
    };

    // Full-screen transparent flex layer — same approach as the original single-sprite render.
    let layer = div()
        .absolute()
        .top(px(0.))
        .left(px(0.))
        .size_full()
        .flex();

    let layer = if is_top { layer.items_start() } else { layer.items_end() };
    let layer = if is_right {
        layer.justify_end()
    } else if is_center {
        layer.justify_center()
    } else {
        layer.justify_start()
    };

    let layer = if is_top { layer.pt(px(pad_y)) } else { layer.pb(px(pad_y)) };
    let layer = if !is_full && is_right {
        layer.pr(px(pad_x))
    } else if !is_full && !is_center {
        layer.pl(px(pad_x))
    } else {
        layer
    };

    layer.child(inner)
}

impl Render for OverlayHudRoot {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_sprites_from_core();

        // Sort owners for deterministic z-order.
        let mut owners: Vec<&str> = self.sprites.keys().map(|s| s.as_str()).collect();
        owners.sort_unstable();

        let mut container = div().size_full().relative();
        for owner in owners {
            container = container.child(sprite_layer(&self.sprites[owner]));
        }
        container
    }
}
