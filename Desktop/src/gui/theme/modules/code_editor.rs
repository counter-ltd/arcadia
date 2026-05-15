#![allow(clippy::approx_constant)]

use openframe::Rgba;

pub fn code_explorer_sidebar_bg(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.078,
            g: 0.098,
            b: 0.125,
            a: 1.0,
        } // 0x141920
    } else {
        Rgba {
            r: 0.953,
            g: 0.957,
            b: 0.965,
            a: 1.0,
        } // 0xf3f4f6
    }
}

pub fn code_explorer_border(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.165,
            g: 0.200,
            b: 0.251,
            a: 1.0,
        } // 0x2a3340
    } else {
        Rgba {
            r: 0.898,
            g: 0.906,
            b: 0.922,
            a: 1.0,
        } // 0xe5e7eb
    }
}

pub fn code_explorer_text(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.820,
            g: 0.835,
            b: 0.859,
            a: 1.0,
        } // 0xd1d5db
    } else {
        Rgba {
            r: 0.216,
            g: 0.255,
            b: 0.318,
            a: 1.0,
        } // 0x374151
    }
}

pub fn code_explorer_dim(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.420,
            g: 0.447,
            b: 0.502,
            a: 1.0,
        } // 0x6b7280
    } else {
        Rgba {
            r: 0.612,
            g: 0.639,
            b: 0.686,
            a: 1.0,
        } // 0x9ca3af
    }
}

pub fn code_explorer_hover_bg(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.118,
            g: 0.145,
            b: 0.188,
            a: 1.0,
        } // 0x1e2530
    } else {
        Rgba {
            r: 0.898,
            g: 0.906,
            b: 0.922,
            a: 1.0,
        } // 0xe5e7eb
    }
}

pub fn code_explorer_active_row_bg(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.102,
            g: 0.176,
            b: 0.271,
            a: 1.0,
        } // 0x1a2d45
    } else {
        Rgba {
            r: 0.859,
            g: 0.918,
            b: 0.996,
            a: 1.0,
        } // 0xdbeafe
    }
}

pub fn code_explorer_active_row_text(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.576,
            g: 0.773,
            b: 0.992,
            a: 1.0,
        } // 0x93c5fd
    } else {
        Rgba {
            r: 0.114,
            g: 0.306,
            b: 0.847,
            a: 1.0,
        } // 0x1d4ed8
    }
}

/// Opaque black used for overlay / modal mask layers.
pub const OVERLAY_MASK_COLOR: Rgba = Rgba {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};
