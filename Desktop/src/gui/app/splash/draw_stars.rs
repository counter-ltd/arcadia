use openframe::{fill, point, px, size, Bounds, PathBuilder, Window};

use crate::gui::theme;

use super::math::alpha_rgba;

pub(super) fn splash_draw_stars(bounds: Bounds<openframe::Pixels>, t: f32, window: &mut Window) {
    let w = f32::from(bounds.size.width);
    let h = f32::from(bounds.size.height);
    let ox = f32::from(bounds.origin.x);
    let oy = f32::from(bounds.origin.y);

    let stars: &[(f32, f32, f32, f32, bool)] = &[
        (0.500, 0.380, 5.0, 0.00, true),
        (0.460, 0.295, 1.8, 0.15, false),
        (0.525, 0.275, 1.4, 0.25, false),
        (0.572, 0.345, 1.4, 0.35, false),
        (0.442, 0.418, 1.2, 0.10, false),
        (0.551, 0.448, 1.2, 0.20, false),
        (0.610, 0.318, 1.5, 0.30, false),
        (0.398, 0.362, 1.2, 0.40, false),
    ];

    for &(fx, fy, star_r, delay, is_sparkle) in stars {
        let local_t = ((t - delay) / (1.0 - delay.min(0.9))).clamp(0.0, 1.0);
        if local_t <= 0.0 {
            continue;
        }
        let cx = ox + fx * w;
        let cy = oy + fy * h;
        let alpha = local_t;

        if is_sparkle {
            splash_draw_sparkle(cx, cy, star_r, alpha, window);
        } else {
            let sb = Bounds {
                origin: point(px(cx - star_r), px(cy - star_r)),
                size: size(px(star_r * 2.0), px(star_r * 2.0)),
            };
            window.paint_quad(
                fill(sb, alpha_rgba(theme::SPLASH_STAR, alpha)).corner_radii(px(star_r)),
            );
        }
    }
}

fn splash_draw_sparkle(cx: f32, cy: f32, r: f32, alpha: f32, window: &mut Window) {
    for angle_offset in [0.0_f32, std::f32::consts::PI / 4.0] {
        let mut pb = PathBuilder::fill();
        let inner = r * 0.18;
        let pts = 4usize;
        for i in 0..(pts * 2) {
            let angle = angle_offset + (i as f32 * std::f32::consts::PI) / pts as f32
                - std::f32::consts::FRAC_PI_2;
            let rad = if i % 2 == 0 { r } else { inner };
            let x = cx + angle.cos() * rad;
            let y = cy + angle.sin() * rad;
            if i == 0 {
                pb.move_to(point(px(x), px(y)));
            } else {
                pb.line_to(point(px(x), px(y)));
            }
        }
        pb.close();
        if let Ok(path) = pb.build() {
            window.paint_path(path, alpha_rgba(theme::SPLASH_STAR, alpha));
        }
    }
    let glow_r = r * 1.6;
    let gb = Bounds {
        origin: point(px(cx - glow_r), px(cy - glow_r)),
        size: size(px(glow_r * 2.0), px(glow_r * 2.0)),
    };
    window.paint_quad(
        fill(gb, alpha_rgba(theme::SPLASH_STAR, alpha * 0.18)).corner_radii(px(glow_r)),
    );
}
