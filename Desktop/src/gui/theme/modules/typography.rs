use openframe::Rgba;

pub fn module_title_text(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.940,
            g: 0.965,
            b: 1.000,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.090,
            g: 0.145,
            b: 0.230,
            a: 1.0,
        }
    }
}

pub fn module_description_text(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.700,
            g: 0.760,
            b: 0.840,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.320,
            g: 0.390,
            b: 0.500,
            a: 1.0,
        }
    }
}

pub fn module_meta_text(is_dark: bool) -> Rgba {
    if is_dark {
        Rgba {
            r: 0.775,
            g: 0.835,
            b: 0.930,
            a: 1.0,
        }
    } else {
        Rgba {
            r: 0.250,
            g: 0.390,
            b: 0.660,
            a: 1.0,
        }
    }
}
