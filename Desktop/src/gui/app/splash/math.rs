use openframe::Rgba;

use crate::gui::theme;

pub(super) fn splash_phase(elapsed_ms: f32, start_ms: f32, duration_ms: f32) -> f32 {
    ((elapsed_ms - start_ms) / duration_ms).clamp(0.0, 1.0)
}

pub(super) fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

pub(super) fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub(super) fn lerp_rgba(a: Rgba, b: Rgba, t: f32) -> Rgba {
    Rgba {
        r: lerp_f32(a.r, b.r, t),
        g: lerp_f32(a.g, b.g, t),
        b: lerp_f32(a.b, b.b, t),
        a: lerp_f32(a.a, b.a, t),
    }
}

pub(super) fn alpha_rgba(color: Rgba, alpha: f32) -> Rgba {
    Rgba {
        a: color.a * alpha,
        ..color
    }
}

pub(super) fn splash_scene_width(w: f32, h: f32) -> f32 {
    w.min(h * 1.52)
}

pub(super) fn splash_gradient_color(t: f32) -> Rgba {
    if t < 0.45 {
        lerp_rgba(theme::SPLASH_BG_TOP, theme::SPLASH_BG_MID, t / 0.45)
    } else if t < 0.72 {
        lerp_rgba(
            theme::SPLASH_BG_MID,
            theme::SPLASH_BG_HORIZON,
            (t - 0.45) / 0.27,
        )
    } else {
        lerp_rgba(
            theme::SPLASH_BG_HORIZON,
            theme::SPLASH_BG_BOTTOM,
            (t - 0.72) / 0.28,
        )
    }
}
