use openframe::{point, px, Bounds, PathBuilder, Rgba, Window};

use crate::gui::theme;

use super::math::{alpha_rgba, splash_scene_width};

pub(super) fn splash_draw_arch(bounds: Bounds<openframe::Pixels>, t: f32, window: &mut Window) {
    if t <= 0.001 {
        return;
    }
    let w = f32::from(bounds.size.width);
    let h = f32::from(bounds.size.height);
    let ox = f32::from(bounds.origin.x);
    let oy = f32::from(bounds.origin.y);
    let cx = ox + w * 0.5;
    let scene_w = splash_scene_width(w, h);
    let apex_x = cx;
    let apex_y = oy + h * 0.195;
    let base_y = oy + h * 0.680;
    let left_x = cx - scene_w * 0.205;
    let right_x = cx + scene_w * 0.205;

    let fp = |x: f32, y: f32| -> openframe::Point<openframe::Pixels> {
        point(px(apex_x + (x - apex_x) * t), px(apex_y + (y - apex_y) * t))
    };

    let draw_arch = |width: f32, color: Rgba, window: &mut Window| {
        let mut pb = PathBuilder::stroke(px(width));
        pb.move_to(fp(left_x, base_y));
        pb.cubic_bezier_to(
            fp(apex_x, apex_y),
            fp(left_x + scene_w * 0.035, oy + h * 0.520),
            fp(apex_x - scene_w * 0.135, apex_y),
        );
        pb.cubic_bezier_to(
            fp(right_x, base_y),
            fp(apex_x + scene_w * 0.135, apex_y),
            fp(right_x - scene_w * 0.035, oy + h * 0.520),
        );
        if let Ok(path) = pb.build() {
            window.paint_path(path, color);
        }
    };

    let arch_width = (scene_w * 0.050).clamp(52.0, 88.0);
    draw_arch(
        arch_width * 1.85,
        alpha_rgba(theme::SPLASH_ARCH_GLOW, t * 0.040),
        window,
    );
    draw_arch(
        arch_width * 1.35,
        alpha_rgba(theme::SPLASH_ARCH_GLOW, t * 0.105),
        window,
    );
    draw_arch(
        arch_width,
        alpha_rgba(theme::SPLASH_ARCH_CORE, t.min(1.0)),
        window,
    );
}
