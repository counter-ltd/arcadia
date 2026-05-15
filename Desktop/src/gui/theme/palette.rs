//! Canonical desktop **theme palette**: one snapshot resolves semantic colors for `is_dark` and the
//! optional Python **glyph** / TUI style ([`GlyphStyleConfig`]).
//!
//! ## Usage
//!
//! Snapshot once per view when painting many controls:
//!
//! ```ignore
//! let p = theme::theme_palette(cx, is_dark);
//! div().bg(p.surface).text_color(p.ui_text)
//! ```
//!
//! Helpers such as [`crate::gui::theme::ui_surface`] delegate to [`theme_palette`] so call sites can
//! migrate gradually.
//!
//! ## Roles
//!
//! | Field | Role |
//! |-------|------|
//! | `canvas` | Root background |
//! | `surface` / `surface_elevated` | Primary and secondary fills |
//! | `border` | Strokes and dividers |
//! | `ui_text` / `ui_subtext` | Shell, menus, generic chrome |
//! | `content_title` / `content_meta` / `content_body` | Modules, services, settings copy |
//! | `accent` / `on_accent` | Accent fills and foreground on accent |
//! | `chrome_pill_*` | Compact top-bar / toolbar pills |
//! | `panel_*` / `row_*` | Card surfaces when no glyph (distinct neutrals) |
//! | `toggle_knob_off` | Toggle track knob (off) |
//! | `danger` | Destructive labels |
//! | `badge_*` | Non-accent status chips |

use openframe::{rgb, App, Rgba};

use super::glyph_snapshot;
use super::modules::{
    module_description_text, module_meta_text, module_panel_bg, module_panel_stroke, module_row_bg,
    module_row_stroke, module_title_text,
};
use super::GlyphStyleConfig;

/// Resolved colors and radius — see module-level documentation.
#[derive(Clone, Copy, Debug)]
pub struct ThemePalette {
    pub canvas: Rgba,
    pub surface: Rgba,
    pub surface_elevated: Rgba,
    pub border: Rgba,
    pub radius_md: f32,

    /// Shell chrome, menus, context menus ([`super::ui_text`]).
    pub ui_text: Rgba,
    pub ui_subtext: Rgba,

    /// Modules / services / settings pages ([`super::content_emphasis_text`] path).
    pub content_title: Rgba,
    pub content_meta: Rgba,
    pub content_body: Rgba,

    pub accent: Rgba,
    pub on_accent: Rgba,

    pub chrome_pill_bg: Rgba,
    pub chrome_pill_fg: Rgba,
    pub chrome_pill_hover_bg: Rgba,

    /// Module / service cards (defaults align with [`super::modules`] neutrals).
    pub panel_bg: Rgba,
    pub panel_border: Rgba,
    pub row_bg: Rgba,
    pub row_border: Rgba,

    pub toggle_knob_off: Rgba,

    pub danger: Rgba,

    pub badge_info_bg: Rgba,
    pub badge_info_fg: Rgba,
    pub badge_muted_bg: Rgba,
    pub badge_muted_fg: Rgba,
}

/// Resolve the full palette (cheap; inner glyph snapshot is `Copy`).
pub fn theme_palette(cx: &App, is_dark: bool) -> ThemePalette {
    ThemePalette::resolve(glyph_snapshot(cx), is_dark)
}

impl ThemePalette {
    pub fn resolve(g: Option<GlyphStyleConfig>, is_dark: bool) -> Self {
        let canvas = g.map(|x| x.bg).unwrap_or_else(|| {
            if is_dark {
                rgb(0x0f1115)
            } else {
                rgb(0xffffff)
            }
        });

        let surface = g.map(|x| x.surface).unwrap_or_else(|| {
            if is_dark {
                rgb(0x111827)
            } else {
                rgb(0xf9fafb)
            }
        });

        let surface_elevated = g.map(|x| x.surface2).unwrap_or_else(|| {
            if is_dark {
                rgb(0x1f2937)
            } else {
                rgb(0xf3f4f6)
            }
        });

        let border = g.map(|x| x.border).unwrap_or_else(|| {
            if is_dark {
                rgb(0x374151)
            } else {
                rgb(0xd1d5db)
            }
        });

        let radius_md = g.map(|x| x.border_radius).unwrap_or(8.0);

        let ui_text = g.map(|x| x.text).unwrap_or_else(|| {
            if is_dark {
                rgb(0xe5e7eb)
            } else {
                rgb(0x111827)
            }
        });

        let ui_subtext = g.map(|x| x.dim).unwrap_or_else(|| {
            if is_dark {
                rgb(0x9ca3af)
            } else {
                rgb(0x6b7280)
            }
        });

        let content_title = g
            .map(|x| x.text)
            .unwrap_or_else(|| module_title_text(is_dark));
        let content_meta = g
            .map(|x| x.dim)
            .unwrap_or_else(|| module_meta_text(is_dark));
        let content_body = g
            .map(|x| x.dim)
            .unwrap_or_else(|| module_description_text(is_dark));

        let accent = g.map(|x| x.accent).unwrap_or(rgb(0x10b981));
        let on_accent = g.map(|x| x.bg).unwrap_or(rgb(0xffffff));

        let chrome_pill_bg = g
            .map(|x| x.surface2)
            .unwrap_or_else(|| fallback_chrome_pill_bg(is_dark));
        let chrome_pill_fg = g
            .map(|x| x.text)
            .unwrap_or_else(|| fallback_chrome_pill_fg(is_dark));
        let chrome_pill_hover_bg = g
            .map(|x| x.border)
            .unwrap_or_else(|| fallback_chrome_pill_hover_bg(is_dark));

        let panel_bg = g
            .map(|x| x.surface)
            .unwrap_or_else(|| module_panel_bg(is_dark));
        let panel_border = g
            .map(|x| x.border)
            .unwrap_or_else(|| module_panel_stroke(is_dark));
        let row_bg = g
            .map(|x| x.surface2)
            .unwrap_or_else(|| module_row_bg(is_dark));
        let row_border = g
            .map(|x| x.border)
            .unwrap_or_else(|| module_row_stroke(is_dark));

        let toggle_knob_off = g.map(|x| x.bg).unwrap_or_else(|| {
            if is_dark {
                rgb(0xd1d5db)
            } else {
                rgb(0xf8fafc)
            }
        });

        let danger = if is_dark {
            rgb(0xf87171)
        } else {
            rgb(0xdc2626)
        };

        let badge_info_bg = g.map(|x| x.surface).unwrap_or_else(|| {
            if is_dark {
                rgb(0x1e3a8a)
            } else {
                rgb(0xeff6ff)
            }
        });
        let badge_info_fg = g.map(|x| x.accent).unwrap_or_else(|| {
            if is_dark {
                rgb(0xbfdbfe)
            } else {
                rgb(0x3b82f6)
            }
        });
        let badge_muted_bg = g.map(|x| x.surface2).unwrap_or_else(|| {
            if is_dark {
                rgb(0x374151)
            } else {
                rgb(0xf1f5f9)
            }
        });
        let badge_muted_fg = g.map(|x| x.dim).unwrap_or_else(|| {
            if is_dark {
                rgb(0x9ca3af)
            } else {
                rgb(0x64748b)
            }
        });

        Self {
            canvas,
            surface,
            surface_elevated,
            border,
            radius_md,
            ui_text,
            ui_subtext,
            content_title,
            content_meta,
            content_body,
            accent,
            on_accent,
            chrome_pill_bg,
            chrome_pill_fg,
            chrome_pill_hover_bg,
            panel_bg,
            panel_border,
            row_bg,
            row_border,
            toggle_knob_off,
            danger,
            badge_info_bg,
            badge_info_fg,
            badge_muted_bg,
            badge_muted_fg,
        }
    }
}

// Neutral chrome pill fallbacks — kept here so [`super::chrome`] can delegate without importing
// [`ThemePalette`] from chrome (dependency direction: chrome → palette only).

pub(crate) fn fallback_chrome_pill_bg(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.122,
            g: 0.161,
            b: 0.216,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.953,
            g: 0.957,
            b: 0.961,
            a: 1.0,
        }
    }
}

pub(crate) fn fallback_chrome_pill_fg(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.820,
            g: 0.847,
            b: 0.859,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.294,
            g: 0.337,
            b: 0.388,
            a: 1.0,
        }
    }
}

pub(crate) fn fallback_chrome_pill_hover_bg(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.165,
            g: 0.212,
            b: 0.278,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.922,
            g: 0.929,
            b: 0.941,
            a: 1.0,
        }
    }
}
