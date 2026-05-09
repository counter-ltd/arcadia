use openframe::Rgba;

/// Per-group / per-page nav accent: icon tints and row selection (see `Navigation*Definition::accent` in the core).
#[derive(Clone, Copy)]
pub struct NavAccentPalette {
    pub icon_idle: Rgba,
    pub icon_active: Rgba,
    pub row_selected: Rgba,
    pub row_hover: Rgba,
}

#[inline]
pub(super) fn rgb8(r: u8, g: u8, b: u8) -> Rgba {
    Rgba {
        r: f32::from(r) / 255.0,
        g: f32::from(g) / 255.0,
        b: f32::from(b) / 255.0,
        a: 1.0,
    }
}

pub fn nav_accent_palette(accent_key: &str, is_dark: bool) -> NavAccentPalette {
    match accent_key {
        "amber" => super::amber::palette(is_dark),
        "cyan" => super::cyan::palette(is_dark),
        "emerald" => super::emerald::palette(is_dark),
        "violet" => super::violet::palette(is_dark),
        "orange" => super::orange::palette(is_dark),
        "sky" => super::sky::palette(is_dark),
        "indigo" => super::indigo::palette(is_dark),
        "fuchsia" => super::fuchsia::palette(is_dark),
        "teal" => super::teal::palette(is_dark),
        _ => super::sky::palette(is_dark),
    }
}
