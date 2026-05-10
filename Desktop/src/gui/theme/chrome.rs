use openframe::{App, Rgba};

use super::palette::{
    fallback_chrome_pill_bg, fallback_chrome_pill_fg, fallback_chrome_pill_hover_bg, theme_palette,
};

/// Top-bar and inline action pills — [`ThemePalette::chrome_pill_*`].
pub fn action_pill_bg(cx: &App, is_dark: bool) -> Rgba {
    theme_palette(cx, is_dark).chrome_pill_bg
}

pub fn action_pill_text(cx: &App, is_dark: bool) -> Rgba {
    theme_palette(cx, is_dark).chrome_pill_fg
}

pub fn action_pill_hover_bg(cx: &App, is_dark: bool) -> Rgba {
    theme_palette(cx, is_dark).chrome_pill_hover_bg
}

/// Neutral compact pill in the main top bar — falls back to [`super::palette`] neutrals.
pub fn top_bar_pill_bg(is_dark: bool) -> Rgba {
    fallback_chrome_pill_bg(is_dark)
}

pub fn top_bar_pill_text(is_dark: bool) -> Rgba {
    fallback_chrome_pill_fg(is_dark)
}

/// Non-selected sidebar / top-bar nav labels (neutral). Icons use `NavAccentPalette::icon_idle`.
#[inline]
pub fn sidebar_nav_idle_foreground(is_dark: bool) -> Rgba {
    top_bar_pill_text(is_dark)
}

pub fn top_bar_pill_hover_bg(is_dark: bool) -> Rgba {
    fallback_chrome_pill_hover_bg(is_dark)
}

/// Selected top-bar nav pill (e.g. Logs when that page is active).
#[allow(dead_code)]
pub fn top_bar_pill_active_bg(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.122,
            g: 0.165,
            b: 0.243,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.882,
            g: 0.906,
            b: 1.000,
            a: 1.0,
        }
    }
}

#[allow(dead_code)]
pub fn top_bar_pill_active_text(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.576,
            g: 0.773,
            b: 0.992,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.114,
            g: 0.306,
            b: 0.847,
            a: 1.0,
        }
    }
}

#[allow(dead_code)]
pub fn top_bar_pill_active_hover_bg(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.145,
            g: 0.196,
            b: 0.282,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.855,
            g: 0.878,
            b: 0.992,
            a: 1.0,
        }
    }
}

/// Sidebar brand row: outlined “session” chip (matches panel surface).
pub fn sidebar_session_chip_bg(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.090,
            g: 0.106,
            b: 0.133,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }
}

pub fn sidebar_session_chip_border(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.231,
            g: 0.263,
            b: 0.318,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.820,
            g: 0.847,
            b: 0.878,
            a: 1.0,
        }
    }
}

pub fn sidebar_session_chip_text(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.176,
            g: 0.204,
            b: 0.235,
            a: 1.0,
        }
    }
}

pub fn sidebar_session_chip_hover_bg(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.125,
            g: 0.145,
            b: 0.180,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.965,
            g: 0.970,
            b: 0.980,
            a: 1.0,
        }
    }
}
