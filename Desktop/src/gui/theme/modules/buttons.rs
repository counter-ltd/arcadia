use openframe::Rgba;

pub fn button_positive_bg(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.120,
            g: 0.465,
            b: 0.335,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.130,
            g: 0.610,
            b: 0.435,
            a: 1.0,
        }
    }
}

pub fn button_positive_hover_bg(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.145,
            g: 0.520,
            b: 0.380,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.150,
            g: 0.640,
            b: 0.470,
            a: 1.0,
        }
    }
}

pub fn button_positive_text(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.905,
            g: 1.000,
            b: 0.960,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.950,
            g: 1.000,
            b: 0.975,
            a: 1.0,
        }
    }
}

pub fn button_negative_bg(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.620,
            g: 0.250,
            b: 0.285,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.870,
            g: 0.285,
            b: 0.365,
            a: 1.0,
        }
    }
}

pub fn button_negative_text(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 1.000,
            g: 0.925,
            b: 0.940,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 1.000,
            g: 0.955,
            b: 0.965,
            a: 1.0,
        }
    }
}
