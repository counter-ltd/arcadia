use openframe::Rgba;

/// Neutral compact pill in the main top bar (matches cwd / small actions).
pub fn top_bar_pill_bg(is_dark: bool) -> Rgba {
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

pub fn top_bar_pill_text(is_dark: bool) -> Rgba {
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

/// Non-selected sidebar / top-bar nav labels (neutral). Icons use `NavAccentPalette::icon_idle`.
#[inline]
pub fn sidebar_nav_idle_foreground(is_dark: bool) -> Rgba {
    top_bar_pill_text(is_dark)
}

pub fn top_bar_pill_hover_bg(is_dark: bool) -> Rgba {
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
