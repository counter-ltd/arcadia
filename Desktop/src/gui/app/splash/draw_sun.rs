use openframe::{fill, point, px, size, Bounds, Window};

use crate::gui::theme;

use super::math::{alpha_rgba, lerp_f32, splash_scene_width};

pub(super) fn splash_draw_sun(bounds: Bounds<openframe::Pixels>, t: f32, window: &mut Window) {
    let w = f32::from(bounds.size.width);
    let h = f32::from(bounds.size.height);
    let ox = f32::from(bounds.origin.x);
    let oy = f32::from(bounds.origin.y);
    let cx = ox + w * 0.5;
    let base_y = oy + h * 0.652;
    let cy = lerp_f32(oy + h, base_y, t);
    let r = (splash_scene_width(w, h) * 0.050).max(34.0);
    for i in 0..16 {
        let u = i as f32 / 15.0;
        let rm = lerp_f32(6.0, 1.35, u);
        let alpha_mult = (1.0 - u).powf(1.7) * 0.038 + 0.006;
        let base_color = if i < 9 {
            theme::SPLASH_HORIZON_PINK
        } else {
            theme::SPLASH_HORIZON_GOLD
        };
        let gr = r * rm;
        let gb = Bounds {
            origin: point(px(cx - gr), px(cy - gr)),
            size: size(px(gr * 2.0), px(gr * 2.0)),
        };
        window.paint_quad(fill(gb, alpha_rgba(base_color, alpha_mult * t)).corner_radii(px(gr)));
    }

    let soft_edge_layers = [
        (1.28, 0.22, theme::SPLASH_HORIZON_GOLD),
        (1.12, 0.34, theme::SPLASH_SUN_LAYERS[3].2),
    ];
    for (rm, alpha_mult, base_color) in soft_edge_layers {
        let gr = r * rm;
        let gb = Bounds {
            origin: point(px(cx - gr), px(cy - gr)),
            size: size(px(gr * 2.0), px(gr * 2.0)),
        };
        window.paint_quad(fill(gb, alpha_rgba(base_color, alpha_mult * t)).corner_radii(px(gr)));
    }

    let core_bounds = Bounds {
        origin: point(px(cx - r), px(cy - r)),
        size: size(px(r * 2.0), px(r * 2.0)),
    };
    window.paint_quad(
        fill(core_bounds, alpha_rgba(theme::SPLASH_SUN_LAYERS[4].2, t)).corner_radii(px(r)),
    );
}
