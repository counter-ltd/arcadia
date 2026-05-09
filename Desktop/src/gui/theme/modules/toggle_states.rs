use openframe::Rgba;

pub fn module_state_enabled_bg(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.070,
            g: 0.350,
            b: 0.255,
            a: 0.38,
        }
    } else {
        Rgba {
            r: 0.820,
            g: 0.970,
            b: 0.895,
            a: 1.0,
        }
    }
}

pub fn module_state_enabled_text(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.635,
            g: 0.955,
            b: 0.825,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.060,
            g: 0.460,
            b: 0.340,
            a: 1.0,
        }
    }
}

pub fn module_state_disabled_bg(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.420,
            g: 0.180,
            b: 0.190,
            a: 0.35,
        }
    } else {
        Rgba {
            r: 0.995,
            g: 0.880,
            b: 0.895,
            a: 1.0,
        }
    }
}

pub fn module_state_disabled_text(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.985,
            g: 0.760,
            b: 0.785,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.700,
            g: 0.180,
            b: 0.250,
            a: 1.0,
        }
    }
}
