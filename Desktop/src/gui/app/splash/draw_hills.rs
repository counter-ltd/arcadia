use openframe::{point, px, Bounds, PathBuilder, Window};

use crate::gui::theme;

use super::math::alpha_rgba;

const SPLASH_HILLS_FINAL_DROP_PX: f32 = 52.0;

pub(super) fn splash_draw_hills(bounds: Bounds<openframe::Pixels>, t: f32, window: &mut Window) {
    let w = f32::from(bounds.size.width);
    let h = f32::from(bounds.size.height);
    let ox = f32::from(bounds.origin.x);
    let oy = f32::from(bounds.origin.y);
    let offset = SPLASH_HILLS_FINAL_DROP_PX + (1.0 - t) * h * 0.35;
    let p = |fx: f32, fy: f32| -> openframe::Point<openframe::Pixels> {
        point(px(ox + fx * w), px(oy + fy * h + offset))
    };

    {
        let mut pb = PathBuilder::fill();
        pb.move_to(p(-0.05, 0.97));
        pb.cubic_bezier_to(p(0.50, 0.770), p(0.15, 0.920), p(0.34, 0.760));
        pb.cubic_bezier_to(p(1.05, 0.97), p(0.66, 0.760), p(0.85, 0.920));
        pb.line_to(p(1.05, 1.10));
        pb.line_to(p(-0.05, 1.10));
        pb.close();
        if let Ok(path) = pb.build() {
            window.paint_path(path, alpha_rgba(theme::SPLASH_HILL_BACK, 0.72));
        }
    }

    {
        let mut pb = PathBuilder::fill();
        pb.move_to(p(-0.06, 0.905));
        pb.cubic_bezier_to(p(0.28, 0.665), p(0.06, 0.850), p(0.16, 0.690));
        pb.cubic_bezier_to(p(0.50, 0.710), p(0.38, 0.650), p(0.45, 0.690));
        pb.cubic_bezier_to(p(1.06, 0.930), p(0.66, 0.760), p(0.86, 0.900));
        pb.line_to(p(1.06, 1.10));
        pb.line_to(p(-0.06, 1.10));
        pb.close();
        if let Ok(path) = pb.build() {
            window.paint_path(path, alpha_rgba(theme::SPLASH_HILL_LEFT, 0.82));
        }
    }

    {
        let mut pb = PathBuilder::fill();
        pb.move_to(p(1.06, 0.905));
        pb.cubic_bezier_to(p(0.72, 0.665), p(0.94, 0.850), p(0.84, 0.690));
        pb.cubic_bezier_to(p(0.50, 0.710), p(0.62, 0.650), p(0.55, 0.690));
        pb.cubic_bezier_to(p(-0.06, 0.930), p(0.34, 0.760), p(0.14, 0.900));
        pb.line_to(p(-0.06, 1.10));
        pb.line_to(p(1.0, 1.10));
        pb.close();
        if let Ok(path) = pb.build() {
            window.paint_path(path, alpha_rgba(theme::SPLASH_HILL_RIGHT, 0.78));
        }
    }

    {
        let mut pb = PathBuilder::fill();
        pb.move_to(p(-0.05, 1.025));
        pb.cubic_bezier_to(p(1.05, 1.025), p(0.25, 0.885), p(0.75, 0.885));
        pb.line_to(p(1.05, 1.10));
        pb.line_to(p(-0.05, 1.10));
        pb.close();
        if let Ok(path) = pb.build() {
            window.paint_path(path, alpha_rgba(theme::SPLASH_HILL_FRONT, 0.55));
        }
    }

    {
        let mut pb = PathBuilder::fill();
        pb.move_to(p(-0.05, 1.085));
        pb.cubic_bezier_to(p(1.05, 1.085), p(0.28, 0.985), p(0.72, 0.985));
        pb.line_to(p(1.05, 1.10));
        pb.line_to(p(-0.05, 1.10));
        pb.close();
        if let Ok(path) = pb.build() {
            window.paint_path(path, alpha_rgba(theme::SPLASH_HILL_FRONT, 0.85));
        }
    }
}
