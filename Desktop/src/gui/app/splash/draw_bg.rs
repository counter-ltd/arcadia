use openframe::{fill, point, px, size, Bounds, Window};

use super::math::splash_gradient_color;

pub(super) fn splash_draw_bg(bounds: Bounds<openframe::Pixels>, window: &mut Window) {
    let w = f32::from(bounds.size.width);
    let h = f32::from(bounds.size.height);
    let ox = f32::from(bounds.origin.x);
    let oy = f32::from(bounds.origin.y);
    let strips = 180u32;
    for i in 0..strips {
        let t = i as f32 / (strips - 1) as f32;
        let strip_h = h / strips as f32;
        let strip_bounds = Bounds {
            origin: point(px(ox), px(oy + i as f32 * strip_h)),
            size: size(px(w), px(strip_h + 1.0)),
        };
        window.paint_quad(fill(strip_bounds, splash_gradient_color(t)));
    }
}
