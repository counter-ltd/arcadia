//! Desktop theme tokens: splash palette, module UI, chrome, nav accents, icons.
//!
//! **Canonical resolution:** [`ThemePalette`] via [`theme_palette`] — all [`ui_*`](crate::gui::theme)
//! and [`content_*`] helpers delegate there for a single source of truth.

mod chrome;
mod icons;
mod modules;
mod nav_accents;
/// Canonical palette types and resolver ([`palette::ThemePalette`], [`palette::theme_palette`]).
pub mod palette;
mod splash_colors;

pub use chrome::*;
pub use icons::*;
pub use modules::*;
pub use nav_accents::*;
pub use palette::theme_palette;
pub use splash_colors::*;

use openframe::{App, Global, Rgba};

/// Full glyph rendering config provided by a Python extension style.
/// Colors are linear Rgba; `border_chars` are the 7 border glyphs (any symbols extensions choose).
#[derive(Copy, Clone)]
pub struct GlyphStyleConfig {
    pub bg: Rgba,
    pub surface: Rgba,
    pub surface2: Rgba,
    pub text: Rgba,
    pub dim: Rgba,
    pub border: Rgba,
    pub accent: Rgba,
    pub border_radius: f32,
    /// `[top_left, top, top_right, side, bottom_left, bottom, bottom_right]`
    pub border_chars: [char; 7],
}

/// Optional typography for [`openframe::GlyphBorder`] borders, registered separately so
/// [`GlyphStyleConfig`] stays `Copy` (color snapshot only).
#[derive(Clone)]
pub struct GlyphBorderTypography {
    pub font_family: Option<String>,
    pub font_size_rems: Option<f32>,
    pub side_rail_px: Option<f32>,
}

/// Repeating horizontal / vertical border sequences from [`GlyphParams`] (spaces = gaps).
#[derive(Clone)]
pub struct GlyphBorderPatterns {
    pub horizontal: Option<String>,
    pub vertical: Option<String>,
}

/// Desktop-level global that carries the active glyph style config.
/// `None` means the Default style is active.
pub struct ActiveGlyphStyle(pub Option<GlyphStyleConfig>);

impl Global for ActiveGlyphStyle {}

/// Active border typography from the Python style’s [`arcadia_core::modules::python_registry::GlyphParams`].
pub struct ActiveGlyphBorderTypography(pub Option<GlyphBorderTypography>);

impl Global for ActiveGlyphBorderTypography {}

/// Active dashed / alternating border patterns from [`GlyphParams`].
pub struct ActiveGlyphBorderPatterns(pub Option<GlyphBorderPatterns>);

impl Global for ActiveGlyphBorderPatterns {}

/// Returns a reference to the active glyph style config, or `None` when Default is active.
pub fn active_glyph(cx: &App) -> Option<&GlyphStyleConfig> {
    cx.try_global::<ActiveGlyphStyle>()
        .and_then(|s| s.0.as_ref())
}

/// Extracts glyph config as an owned value, ending the cx borrow immediately.
/// Use this when you need glyph colors alongside mutable cx (e.g. cx.listener calls).
pub fn glyph_snapshot(cx: &App) -> Option<GlyphStyleConfig> {
    active_glyph(cx).copied()
}

/// Applies extension-provided border typography and repeating glyph patterns when set.
pub fn apply_glyph_border_typography(cx: &App, mut b: openframe::GlyphBorder) -> openframe::GlyphBorder {
    if let Some(extra) = cx
        .try_global::<ActiveGlyphBorderTypography>()
        .and_then(|x| x.0.as_ref())
    {
        if let Some(ref f) = extra.font_family {
            b = b.border_font_family(f.as_str());
        }
        if let Some(r) = extra.font_size_rems {
            b = b.border_font_size_rems(r);
        }
        if let Some(w) = extra.side_rail_px {
            b = b.border_side_rail_px(w);
        }
    }
    if let Some(pat) = cx
        .try_global::<ActiveGlyphBorderPatterns>()
        .and_then(|x| x.0.as_ref())
    {
        if let Some(ref h) = pat.horizontal {
            if !h.is_empty() {
                b = b.border_horizontal_pattern(h.clone());
            }
        }
        if let Some(ref v) = pat.vertical {
            if !v.is_empty() {
                b = b.border_vertical_pattern(v.clone());
            }
        }
    }
    b
}

/// Returns true when a custom glyph style is active.
pub fn is_glyph_active(cx: &App) -> bool {
    active_glyph(cx).is_some()
}

// ---------------------------------------------------------------------------
// Global cx-aware tokens — read glyph style automatically, fall back to
// dark/light defaults. Safe to call in builder chains alongside cx.listener
// because glyph_snapshot returns an owned `Copy` snapshot (borrow ends immediately).
// ---------------------------------------------------------------------------

pub fn ui_bg(cx: &App, is_dark: bool) -> Rgba {
    theme_palette(cx, is_dark).canvas
}

pub fn ui_surface(cx: &App, is_dark: bool) -> Rgba {
    theme_palette(cx, is_dark).surface
}

pub fn ui_surface2(cx: &App, is_dark: bool) -> Rgba {
    theme_palette(cx, is_dark).surface_elevated
}

pub fn ui_text(cx: &App, is_dark: bool) -> Rgba {
    theme_palette(cx, is_dark).ui_text
}

pub fn ui_subtext(cx: &App, is_dark: bool) -> Rgba {
    theme_palette(cx, is_dark).ui_subtext
}

pub fn ui_border(cx: &App, is_dark: bool) -> Rgba {
    theme_palette(cx, is_dark).border
}

/// Accent does not vary with light/dark in the default palette; `is_dark` is ignored for these
/// fields when no glyph is active. Call sites without `is_dark` use `true` as a stable default.
pub fn ui_accent(cx: &App) -> Rgba {
    theme_palette(cx, true).accent
}

pub fn ui_accent_fg(cx: &App) -> Rgba {
    theme_palette(cx, true).on_accent
}

pub fn ui_radius(cx: &App) -> f32 {
    theme_palette(cx, true).radius_md
}

/// Destructive / error foreground (e.g. “Kill terminal”, inline errors).
pub fn ui_danger(cx: &App, is_dark: bool) -> Rgba {
    theme_palette(cx, is_dark).danger
}

/// Primary body copy on content panels — follows glyph text when a custom style is active.
pub fn content_emphasis_text(cx: &App, is_dark: bool) -> Rgba {
    theme_palette(cx, is_dark).content_title
}

/// Secondary labels — follows glyph dim when a custom style is active.
pub fn content_muted_text(cx: &App, is_dark: bool) -> Rgba {
    theme_palette(cx, is_dark).content_meta
}

/// Tertiary / placeholder copy — follows glyph dim when a custom style is active.
pub fn content_subdued_text(cx: &App, is_dark: bool) -> Rgba {
    theme_palette(cx, is_dark).content_body
}
