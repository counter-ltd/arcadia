use openframe::Rgba;

/// Max width for glyph-style `GlyphBorder` panels (modules, Python extensions). Keeps box-drawing
/// borders at a scale where corners read cleanly instead of stretching edge-to-edge on wide windows.
pub const GLYPH_PANEL_CONTENT_MAX_W_PX: f32 = 768.;

pub fn module_panel_bg(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.095,
            g: 0.115,
            b: 0.145,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.965,
            g: 0.978,
            b: 0.995,
            a: 1.0,
        }
    }
}

pub fn module_panel_stroke(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.190,
            g: 0.230,
            b: 0.290,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.840,
            g: 0.885,
            b: 0.945,
            a: 1.0,
        }
    }
}

/// Inset “tray” behind late.sh ASCII bonsai art.
pub fn late_bonsai_well_bg(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.051,
            g: 0.063,
            b: 0.082,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.918,
            g: 0.929,
            b: 0.945,
            a: 1.0,
        }
    }
}

pub fn late_bonsai_well_stroke(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.145,
            g: 0.175,
            b: 0.220,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.765,
            g: 0.805,
            b: 0.865,
            a: 1.0,
        }
    }
}

/// Terracotta-ish band suggesting pot rim above soil area.
pub fn late_bonsai_pot_band(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.290,
            g: 0.165,
            b: 0.125,
            a: 0.92,
        }
    } else {
        Rgba {
            r: 0.725,
            g: 0.435,
            b: 0.330,
            a: 0.55,
        }
    }
}

/// Foliage tint for monospace tree glyphs.
pub fn late_bonsai_foliage_text(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.620,
            g: 0.980,
            b: 0.710,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.118,
            g: 0.358,
            b: 0.184,
            a: 1.0,
        }
    }
}
