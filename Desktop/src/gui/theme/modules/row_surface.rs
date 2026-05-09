use openframe::Rgba;

pub fn module_row_bg(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.120,
            g: 0.150,
            b: 0.190,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 1.000,
            g: 1.000,
            b: 1.000,
            a: 0.98,
        }
    }
}

pub fn module_row_stroke(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.260,
            g: 0.315,
            b: 0.390,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.860,
            g: 0.905,
            b: 0.965,
            a: 1.0,
        }
    }
}
