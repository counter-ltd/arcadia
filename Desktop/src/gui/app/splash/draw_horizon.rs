use openframe::{fill, point, px, size, Bounds, Window};

use crate::gui::theme;

use super::math::{alpha_rgba, lerp_f32};

pub(super) fn splash_draw_horizon_glow(bounds: Bounds<openframe::Pixels>, t: f32, window: &mut Window) {
    let w = f32::from(bounds.size.width);
    let h = f32::from(bounds.size.height);
    let ox = f32::from(bounds.origin.x);
    let oy = f32::from(bounds.origin.y);
    let cx = ox + w * 0.5;
    let cy = oy + h * 0.665;

    for i in 0..14 {
        let u = i as f32 / 13.0;
        let gw = w * lerp_f32(0.18, 0.92, u);
        let gh = h * lerp_f32(0.08, 0.34, u);
        let alpha = (1.0 - u).powi(2) * 0.11 * t;
        let color = if i < 5 {
            theme::SPLASH_HORIZON_GOLD
        } else {
            theme::SPLASH_HORIZON_PINK
        };
        let gb = Bounds {
            origin: point(px(cx - gw * 0.5), px(cy - gh * 0.5)),
            size: size(px(gw), px(gh)),
        };
        window.paint_quad(fill(gb, alpha_rgba(color, alpha)).corner_radii(px(gh * 0.5)));
    }
}
