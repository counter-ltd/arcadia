//! Root view for the shared HUD overlay window (transparent; bottom-right extension sprite).
//!
//! Sprite pixels are pulled from [`arcadia_core::modules::overlay_hud_sprite`] inside
//! [`Render::render`] on the UI thread so we never call [`openframe::AsyncApp::update`] for
//! sprite changes (avoids re-entrant `App` `borrow_mut` deadlock with the foreground timer loop).

use std::sync::Arc;

use arcadia_core::modules::overlay_hud_sprite;
use image::{Frame, ImageBuffer, Rgba};
use openframe::{div, img, prelude::*, px, IntoElement, Render, RenderImage, Window};

pub struct OverlayHudRoot {
    last_sprite_version: u64,
    sprite: Option<Arc<RenderImage>>,
    pad_right: f32,
    pad_bottom: f32,
    display_width: Option<f32>,
    display_height: Option<f32>,
}

impl OverlayHudRoot {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            last_sprite_version: u64::MAX,
            sprite: None,
            pad_right: 24.,
            pad_bottom: 24.,
            display_width: None,
            display_height: None,
        }
    }

    fn sync_sprite_from_core(&mut self) {
        let Some((ver, payload)) =
            overlay_hud_sprite::clone_if_newer_than(self.last_sprite_version)
        else {
            return;
        };
        self.last_sprite_version = ver;
        match payload {
            Some(p) => {
                self.pad_right = p.pad_right;
                self.pad_bottom = p.pad_bottom;
                self.display_width = p.display_width;
                self.display_height = p.display_height;
                self.sprite = render_image_from_rgba(p.rgba, p.width, p.height);
            }
            None => {
                self.sprite = None;
            }
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

impl Render for OverlayHudRoot {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_sprite_from_core();

        let pad_r = self.pad_right;
        let pad_b = self.pad_bottom;
        let display_width = self.display_width;
        let display_height = self.display_height;
        let inner = if let Some(sprite) = self.sprite.clone() {
            let (iw, ih) = {
                let s = sprite.size(0);
                (s.width.0 as f32, s.height.0 as f32)
            };
            div().flex_none().child(
                img(sprite)
                    .w(px(display_width.unwrap_or(iw)))
                    .h(px(display_height.unwrap_or(ih))),
            )
        } else {
            div().flex_none()
        };

        div()
            .size_full()
            .flex()
            .items_end()
            .justify_end()
            .pb(px(pad_b))
            .pr(px(pad_r))
            .child(inner)
    }
}
