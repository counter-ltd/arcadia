use openframe::Rgba;
pub const SPLASH_BG_TOP: Rgba = Rgba {
    r: 0.060,
    g: 0.055,
    b: 0.580,
    a: 1.0,
};
pub const SPLASH_BG_MID: Rgba = Rgba {
    r: 0.205,
    g: 0.105,
    b: 0.760,
    a: 1.0,
};
pub const SPLASH_BG_HORIZON: Rgba = Rgba {
    r: 0.790,
    g: 0.240,
    b: 0.760,
    a: 1.0,
};
pub const SPLASH_BG_BOTTOM: Rgba = Rgba {
    r: 1.000,
    g: 0.480,
    b: 0.560,
    a: 1.0,
};

pub const SPLASH_HORIZON_PINK: Rgba = Rgba {
    r: 1.000,
    g: 0.250,
    b: 0.670,
    a: 1.0,
};
pub const SPLASH_HORIZON_GOLD: Rgba = Rgba {
    r: 1.000,
    g: 0.690,
    b: 0.250,
    a: 1.0,
};

pub const SPLASH_HILL_BACK: Rgba = Rgba {
    r: 0.430,
    g: 0.150,
    b: 0.900,
    a: 0.62,
};
pub const SPLASH_HILL_LEFT: Rgba = Rgba {
    r: 0.500,
    g: 0.180,
    b: 0.920,
    a: 0.78,
};
pub const SPLASH_HILL_RIGHT: Rgba = Rgba {
    r: 0.420,
    g: 0.135,
    b: 0.835,
    a: 0.76,
};
pub const SPLASH_HILL_FRONT: Rgba = Rgba {
    r: 0.045,
    g: 0.040,
    b: 0.430,
    a: 0.82,
};

pub const SPLASH_ARCH_CORE: Rgba = Rgba {
    r: 0.930,
    g: 0.860,
    b: 1.000,
    a: 1.0,
};
pub const SPLASH_ARCH_GLOW: Rgba = Rgba {
    r: 0.765,
    g: 0.610,
    b: 1.000,
    a: 1.0,
};

pub const SPLASH_SUN_LAYERS: [(f32, f32, Rgba); 5] = [
    (
        4.2,
        0.055,
        Rgba {
            r: 1.000,
            g: 0.300,
            b: 0.620,
            a: 1.0,
        },
    ),
    (
        3.2,
        0.115,
        Rgba {
            r: 1.000,
            g: 0.500,
            b: 0.280,
            a: 1.0,
        },
    ),
    (
        2.2,
        0.210,
        Rgba {
            r: 1.000,
            g: 0.690,
            b: 0.300,
            a: 1.0,
        },
    ),
    (
        1.45,
        0.440,
        Rgba {
            r: 1.000,
            g: 0.830,
            b: 0.520,
            a: 1.0,
        },
    ),
    (
        1.0,
        1.000,
        Rgba {
            r: 1.000,
            g: 0.950,
            b: 0.770,
            a: 1.0,
        },
    ),
];

pub const SPLASH_STAR: Rgba = Rgba {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};
