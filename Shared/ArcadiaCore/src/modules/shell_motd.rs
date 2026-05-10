//! Arcadia MOTD — ethereal gateway scene (gradient sky, parabolic arch, sun, dunes) + system info.

use crate::config::appearance::AppearanceConfig;
use crate::config::extension_tokens;
use crate::config::ConfigFile;
use crate::modules::python_registry::{list_styles, GlyphParams};
use crate::modules::{ExecutionContext, ModuleCommand};
use std::fmt::Write as _;

#[derive(Clone, Copy)]
struct MotdAnsiPalette {
    accent: (u8, u8, u8),
    body: (u8, u8, u8),
    dim: (u8, u8, u8),
    sep: (u8, u8, u8),
}

fn hex_to_rgb(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim().trim_start_matches('#');
    if hex.len() < 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some((r, g, b))
}

fn rgb_from_hex(opt: Option<&String>, fallback: (u8, u8, u8)) -> (u8, u8, u8) {
    opt.and_then(|s| hex_to_rgb(s.as_str())).unwrap_or(fallback)
}

fn merged_glyph_for_active_style(is_dark: bool) -> Option<GlyphParams> {
    let active = AppearanceConfig::load_or_create().unwrap_or_default().active_style;
    let style_row = list_styles().into_iter().find(|s| s.name == active)?;
    let mut g = if is_dark {
        style_row.glyph.clone()
    } else {
        style_row.glyph_light.clone().or_else(|| style_row.glyph.clone())
    }?;
    if let Some(module) = style_row.module_name.as_deref() {
        if let Ok(file) = extension_tokens::load_module_tokens(module) {
            extension_tokens::apply_file_tokens_to_glyph(&mut g, &file, is_dark);
        }
    }
    Some(g)
}

fn palette_from_glyph(g: &GlyphParams, is_dark: bool) -> MotdAnsiPalette {
    let defaults = if is_dark {
        (
            (16_u8, 185_u8, 129_u8),
            (229_u8, 231_u8, 235_u8),
            (156_u8, 163_u8, 175_u8),
            (55_u8, 65_u8, 81_u8),
        )
    } else {
        (
            (16_u8, 185_u8, 129_u8),
            (17_u8, 24_u8, 39_u8),
            (107_u8, 114_u8, 128_u8),
            (209_u8, 213_u8, 219_u8),
        )
    };
    MotdAnsiPalette {
        accent: rgb_from_hex(g.accent.as_ref(), defaults.0),
        body: rgb_from_hex(g.text.as_ref(), defaults.1),
        dim: rgb_from_hex(g.dim.as_ref(), defaults.2),
        sep: rgb_from_hex(g.border.as_ref(), defaults.3),
    }
}

fn fallback_palette(is_dark: bool) -> MotdAnsiPalette {
    if is_dark {
        MotdAnsiPalette {
            accent: (176, 162, 236),
            body: (235, 238, 248),
            dim: (158, 138, 220),
            sep: (118, 102, 198),
        }
    } else {
        MotdAnsiPalette {
            accent: (16, 185, 129),
            body: (17, 24, 39),
            dim: (107, 114, 128),
            sep: (5, 150, 105),
        }
    }
}

fn motd_palette(is_dark: bool) -> MotdAnsiPalette {
    merged_glyph_for_active_style(is_dark)
        .map(|g| palette_from_glyph(&g, is_dark))
        .unwrap_or_else(|| fallback_palette(is_dark))
}

/// Dark/light for CLI MOTD when no GUI passes a scheme (`ARCADIA_COLOR_SCHEME=light|dark`).
pub fn guess_terminal_scheme_dark() -> bool {
    match std::env::var("ARCADIA_COLOR_SCHEME") {
        Ok(s) => match s.to_ascii_lowercase().as_str() {
            "light" => return false,
            "dark" => return true,
            _ => {}
        },
        Err(_) => {}
    }
    true
}

pub const NAME: &str = "shell-motd";

// ── pixel / color helpers ─────────────────────────────────────────────────────

/// One monospace cell — full block + matching fg so GPUI/fonts align columns like real TUIs.
fn px(r: u8, g: u8, b: u8) -> String {
    format!("\x1b[38;2;{r};{g};{b}m\x1b[48;2;{r};{g};{b}m█\x1b[0m")
}

fn star_tile(fr: u8, fg: u8, fb: u8, br: u8, bg: u8, bb: u8) -> String {
    format!("\x1b[38;2;{fr};{fg};{fb}m\x1b[48;2;{br};{bg};{bb}m█\x1b[0m")
}

fn lerp(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).round() as u8
}

fn lerp3(a: (u8, u8, u8), b: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
    (lerp(a.0, b.0, t), lerp(a.1, b.1, t), lerp(a.2, b.2, t))
}

fn lighten(rgb: (u8, u8, u8), amt: u8) -> (u8, u8, u8) {
    (
        rgb.0.saturating_add(amt).min(255),
        rgb.1.saturating_add(amt).min(255),
        rgb.2.saturating_add((amt / 2).min(255)),
    )
}

/// Visible char count — strips ANSI CSI/OSC sequences.
fn vw(s: &str) -> usize {
    let mut n = 0usize;
    let mut it = s.chars().peekable();
    while let Some(ch) = it.next() {
        if ch == '\x1b' {
            match it.peek().copied() {
                Some('[') => {
                    it.next();
                    while let Some(c) = it.next() {
                        if ('\x40'..='\x7e').contains(&c) {
                            break;
                        }
                    }
                }
                Some(']') => {
                    it.next();
                    while let Some(c) = it.next() {
                        if c == '\x07' {
                            break;
                        }
                        if c == '\x1b' && it.peek() == Some(&'\\') {
                            it.next();
                            break;
                        }
                    }
                }
                _ => {}
            }
            continue;
        }
        n += 1;
    }
    n
}

// ── Arcadia arch art ──────────────────────────────────────────────────────────

fn arch_art_lines() -> Vec<String> {
    /// Inner scene width; outer row width adds symmetric gutters for flush vertical edges.
    const GUTTER: usize = 2;
    /// Drawable columns — row length is `IW + 2 * GUTTER` (fixed vertical gutters).
    const IW: usize = 40;
    const SKY_ROWS: usize = 8;
    const DUNE_ROWS: usize = 2;
    const BASE_W: f32 = 36.0;

    let cx_f = (IW as f32) / 2.0;
    let gs = (IW as f32) / BASE_W;
    let gutter_rgb = (22, 16, 56);

    // Outer sky: deep indigo → warm dusk (Image #2 vibe)
    let sky_out_t: (u8, u8, u8) = (42, 28, 92);
    let sky_out_m: (u8, u8, u8) = (88, 48, 138);
    let sky_out_b: (u8, u8, u8) = (168, 92, 118);
    // Inner sky (through gate): cooler, subtler horizon bloom
    let sky_in_t: (u8, u8, u8) = (38, 36, 108);
    let sky_in_m: (u8, u8, u8) = (72, 52, 142);
    let sky_in_b: (u8, u8, u8) = (122, 72, 132);

    // Arch — luminous lilac stack + deeper silhouettes on outer faces
    let arch_core: (u8, u8, u8) = (252, 248, 255);
    let arch_mid: (u8, u8, u8) = (224, 214, 248);
    let arch_mid2: (u8, u8, u8) = (202, 188, 236);
    let arch_edge: (u8, u8, u8) = (168, 150, 222);
    let arch_deep: (u8, u8, u8) = (132, 112, 188);

    let sun_c: (u8, u8, u8) = (255, 242, 188);
    let sun_i: (u8, u8, u8) = (253, 228, 158);
    let sun_g: (u8, u8, u8) = (246, 188, 102);
    let sun_o: (u8, u8, u8) = (218, 132, 122);
    let sun_r: (u8, u8, u8) = (158, 92, 138);

    // Layered dunes
    let dune_back: (u8, u8, u8) = (52, 36, 118);
    let dune_mid: (u8, u8, u8) = (44, 32, 98);
    let dune_front: (u8, u8, u8) = (36, 26, 82);

    let stars_raw: &[(usize, usize, char)] = &[
        (1, 5, '.'),
        (1, 18, '.'),
        (1, 31, '.'),
        (2, 8, '.'),
        (2, 22, '*'),
        (2, 33, '.'),
        (3, 6, '.'),
        (3, 15, '.'),
        (3, 28, '.'),
        (4, 11, '.'),
        (4, 24, '.'),
        (5, 5, '.'),
        (5, 13, '*'),
        (5, 20, '.'),
        (5, 30, '.'),
        (6, 9, '.'),
        (6, 26, '.'),
        (7, 7, '.'),
        (7, 18, '.'),
        (7, 32, '.'),
    ];
    let wf_inner = IW as f32;
    let stars: Vec<(usize, usize, char)> = stars_raw
        .iter()
        .copied()
        .map(|(r, c, ch)| {
            let nc = ((c as f32 / BASE_W) * wf_inner)
                .round()
                .clamp(0.0, wf_inner - 1.0) as usize;
            (r, nc, ch)
        })
        .collect();

    let mut out = Vec::with_capacity(SKY_ROWS + DUNE_ROWS);

    /// Sky color by region and vertical position (smooth 3-stop gradient).
    fn sky_at(
        outer: bool,
        y: f32,
        sky_out_t: (u8, u8, u8),
        sky_out_m: (u8, u8, u8),
        sky_out_b: (u8, u8, u8),
        sky_in_t: (u8, u8, u8),
        sky_in_m: (u8, u8, u8),
        sky_in_b: (u8, u8, u8),
    ) -> (u8, u8, u8) {
        let (t, m, b) = if outer {
            (sky_out_t, sky_out_m, sky_out_b)
        } else {
            (sky_in_t, sky_in_m, sky_in_b)
        };
        if y < 0.5 {
            let u = y * 2.0;
            lerp3(t, m, u)
        } else {
            let u = (y - 0.5) * 2.0;
            lerp3(m, b, u)
        }
    }

    for r in 0..SKY_ROWS {
        let t = r as f32 / (SKY_ROWS - 1).max(1) as f32;
        let mut line = String::new();
        for _ in 0..GUTTER {
            line.push_str(&px(gutter_rgb.0, gutter_rgb.1, gutter_rgb.2));
        }

        // Parabolic opening: narrow aloft, wide near horizon (∩ gateway).
        let half_open = (3.2 + 11.8 * t.powf(1.35)) * gs;
        let inner_l = (cx_f - half_open).floor() as isize;
        let inner_r = (cx_f + half_open).ceil() as isize;
        let pillar: isize = if r < 2 {
            (2.0 * gs).round().clamp(2.0, 4.0) as isize
        } else {
            (3.0 * gs).round().clamp(3.0, 5.0) as isize
        };

        let lp0 = inner_l - pillar;
        let lp1 = inner_l;
        let rp0 = inner_r;
        let rp1 = inner_r + pillar;

        // Keystone / curved lintel (rows 0–1): stone bridge above gap.
        let lintel_half = half_open + pillar as f32 + 1.2 * gs;
        let cap_row = r <= 1;

        for ic in 0..IW {
            let c = ic as isize;
            let cf = ic as f32;

            let dist_top = ((cf - cx_f).powi(2) + ((r as f32) - 0.8).powi(2)).sqrt();
            let on_lintel =
                cap_row && dist_top <= lintel_half + 1.8 * gs && dist_top >= lintel_half - 2.6 * gs;

            let in_open = c >= lp1 && c < rp0;
            let left_pillar = c >= lp0 && c < lp1;
            let right_pillar = c >= rp0 && c < rp1;
            let outer_left = c < lp0;
            let outer_right = c >= rp1;

            let region_open = in_open && !on_lintel;

            let dx = (cf - cx_f).abs();
            let sr = r as f32;
            let sun_layer = if region_open && sr >= 3.0 {
                let spread = gs * (5.2 + (sr - 3.0) * 0.75);
                if dx <= spread {
                    if sr >= 6.0 && dx <= 1.05 * gs {
                        Some(sun_c)
                    } else if sr >= 5.5 && dx <= 1.85 * gs {
                        Some(sun_i)
                    } else if sr >= 5.0 && dx <= 3.2 * gs {
                        Some(sun_g)
                    } else if sr >= 4.0 && dx <= 4.9 * gs {
                        Some(sun_o)
                    } else if dx <= spread {
                        Some(sun_r)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            let arch_px = if on_lintel {
                let rim = (dist_top - lintel_half).abs();
                Some(if rim < 0.9 {
                    arch_core
                } else if rim < 1.7 {
                    arch_mid
                } else {
                    arch_edge
                })
            } else if left_pillar || right_pillar {
                let toward_inner = if left_pillar {
                    (lp1 - 1 - c).max(0)
                } else {
                    (c - rp0).max(0)
                };
                Some(match toward_inner {
                    0 => arch_core,
                    1 => arch_mid,
                    2 => arch_mid2,
                    3 => arch_edge,
                    _ => arch_deep,
                })
            } else {
                None
            };

            if let Some(col) = arch_px {
                line.push_str(&px(col.0, col.1, col.2));
            } else if let Some(col) = sun_layer {
                line.push_str(&px(col.0, col.1, col.2));
            } else {
                let outer = outer_left || outer_right;
                let bg = sky_at(
                    outer, t, sky_out_t, sky_out_m, sky_out_b, sky_in_t, sky_in_m, sky_in_b,
                );
                let star = stars.iter().find(|s| s.0 == r && s.1 == ic);
                if let Some(&(_, _, ch)) = star {
                    let (fr, fg, fb) = if ch == '*' {
                        (255, 254, 245)
                    } else {
                        (216, 210, 238)
                    };
                    line.push_str(&star_tile(fr, fg, fb, bg.0, bg.1, bg.2));
                } else {
                    line.push_str(&px(bg.0, bg.1, bg.2));
                }
            }
        }

        for _ in 0..GUTTER {
            line.push_str(&px(gutter_rgb.0, gutter_rgb.1, gutter_rgb.2));
        }
        out.push(line);
    }

    // Rolling dunes + faint crest highlights (same width as sky incl. gutters).
    for hill_r in 0..DUNE_ROWS {
        let mut line = String::new();
        for _ in 0..GUTTER {
            line.push_str(&px(gutter_rgb.0, gutter_rgb.1, gutter_rgb.2));
        }
        for ic in 0..IW {
            let x = ic as f32 / (IW - 1).max(1) as f32;
            let wave_a = (x * 6.25).sin();
            let wave_b = (x * 4.05 + 1.05).sin();
            let h0 = (wave_a * 0.35 + 0.5) > (hill_r as f32 * 0.12 + 0.38);
            let h1 = (wave_b * 0.28 + 0.52) > (hill_r as f32 * 0.1 + 0.42);
            let mut col = if hill_r == 0 {
                if h0 {
                    dune_back
                } else {
                    dune_mid
                }
            } else if h1 {
                dune_mid
            } else {
                dune_front
            };
            let crest = if hill_r == 0 {
                wave_a.abs()
            } else {
                wave_b.abs()
            };
            if crest > 0.92 {
                col = lighten(col, 14);
            }
            line.push_str(&px(col.0, col.1, col.2));
        }
        for _ in 0..GUTTER {
            line.push_str(&px(gutter_rgb.0, gutter_rgb.1, gutter_rgb.2));
        }
        out.push(line);
    }

    out
}

// ── system info ───────────────────────────────────────────────────────────────

fn run_cmd(program: &str, args: &[&str]) -> Option<String> {
    std::process::Command::new(program)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn username() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "user".into())
}

fn machine_model() -> String {
    #[cfg(target_os = "linux")]
    {
        if let Ok(s) = std::fs::read_to_string("/sys/devices/virtual/dmi/id/product_name") {
            let s = s.trim();
            if !s.is_empty() && s != "Default string" {
                return s.to_string();
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(m) = run_cmd("sysctl", &["-n", "hw.model"]) {
            return m;
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Some(o) = run_cmd("wmic", &["computersystem", "get", "model"]) {
            let line = o.lines().nth(1).unwrap_or("").trim();
            if !line.is_empty() && !line.eq_ignore_ascii_case("model") {
                return line.to_string();
            }
        }
    }
    hostname_str()
}

fn hostname_str() -> String {
    #[cfg(target_os = "windows")]
    {
        return std::env::var("COMPUTERNAME").unwrap_or_else(|_| "windows".into());
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(h) = std::fs::read_to_string("/etc/hostname") {
            let h = h.lines().next().unwrap_or("").trim();
            if !h.is_empty() {
                return h.to_string();
            }
        }
        run_cmd("hostname", &[]).unwrap_or_else(|| "localhost".into())
    }
}

fn os_pretty() -> String {
    #[cfg(target_os = "linux")]
    {
        if let Ok(txt) = std::fs::read_to_string("/etc/os-release") {
            for line in txt.lines() {
                if let Some(rest) = line.strip_prefix("PRETTY_NAME=") {
                    let v = rest.trim().trim_matches('"').trim_matches('\'');
                    if !v.is_empty() {
                        return v.to_string();
                    }
                }
            }
        }
        return "Linux".into();
    }
    #[cfg(target_os = "macos")]
    {
        let name = run_cmd("sw_vers", &["-productName"]).unwrap_or_else(|| "macOS".into());
        let ver = run_cmd("sw_vers", &["-productVersion"]).unwrap_or_default();
        if ver.is_empty() {
            name
        } else {
            format!("{name} {ver}")
        }
    }
    #[cfg(target_os = "windows")]
    {
        run_cmd("cmd", &["/C", "ver"]).unwrap_or_else(|| "Windows".into())
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        "Arcadia".into()
    }
}

fn kernel_line() -> String {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let sys = run_cmd("uname", &["-s"]).unwrap_or_else(|| "Unix".into());
        let rel = run_cmd("uname", &["-r"]).unwrap_or_default();
        return if rel.is_empty() {
            sys
        } else {
            format!("{sys} {rel}")
        };
    }
    #[cfg(target_os = "windows")]
    {
        return run_cmd("cmd", &["/C", "ver"]).unwrap_or_else(|| "Windows NT".into());
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        "unknown".into()
    }
}

fn uptime_line() -> String {
    #[cfg(target_os = "linux")]
    {
        if let Some(u) = run_cmd("uptime", &["-p"]) {
            return u;
        }
    }
    if let Some(u) = run_cmd("uptime", &[]) {
        return u;
    }
    "n/a".into()
}

fn shell_env() -> String {
    std::env::var("SHELL")
        .or_else(|_| std::env::var("COMSPEC"))
        .unwrap_or_else(|_| "n/a".into())
}

fn desktop_env() -> String {
    std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("DESKTOP_SESSION"))
        .unwrap_or_else(|_| "n/a".into())
}

fn cpu_model() -> String {
    #[cfg(target_os = "linux")]
    {
        if let Ok(cpuinfo) = std::fs::read_to_string("/proc/cpuinfo") {
            for line in cpuinfo.lines() {
                if let Some(m) = line
                    .strip_prefix("model name\t: ")
                    .or_else(|| line.strip_prefix("model name : "))
                {
                    return m.trim().to_string();
                }
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(m) = run_cmd("sysctl", &["-n", "machdep.cpu.brand_string"]) {
            return m;
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Some(o) = run_cmd("wmic", &["cpu", "get", "name"]) {
            let line = o.lines().nth(1).unwrap_or("").trim();
            if !line.is_empty() {
                return line.to_string();
            }
        }
    }
    "CPU".into()
}

fn memory_line() -> String {
    #[cfg(target_os = "linux")]
    {
        if let Ok(txt) = std::fs::read_to_string("/proc/meminfo") {
            let mut total_kb = 0u64;
            let mut avail_kb = None::<u64>;
            for line in txt.lines() {
                if let Some(n) = line.strip_prefix("MemTotal:") {
                    total_kb = n
                        .split_whitespace()
                        .next()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                }
                if let Some(n) = line.strip_prefix("MemAvailable:") {
                    avail_kb = n.split_whitespace().next().and_then(|s| s.parse().ok());
                }
            }
            if total_kb > 0 {
                let avail = avail_kb.unwrap_or(0);
                let used = total_kb.saturating_sub(avail);
                let pct = 100.0 * used as f64 / total_kb as f64;
                return format!("{} MiB / {} MiB ({pct:.0}%)", used / 1024, total_kb / 1024);
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(bytes) = run_cmd("sysctl", &["-n", "hw.memsize"]) {
            if let Ok(b) = bytes.parse::<u64>() {
                return format!("{} GiB total", b / (1024 * 1024 * 1024));
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Some(o) = run_cmd("wmic", &["computersystem", "get", "TotalPhysicalMemory"]) {
            if let Some(line) = o.lines().nth(1) {
                if let Ok(bytes) = line.trim().parse::<u64>() {
                    return format!("{} GiB total", bytes / (1024 * 1024 * 1024));
                }
            }
        }
    }
    "n/a".into()
}

/// Left column labels for stats block (fixed width reads cleaner beside wide art).
fn lbl_col(key: &str, width: usize, accent: (u8, u8, u8)) -> String {
    format!(
        "\x1b[38;2;{};{};{}m{key:<width$}\x1b[0m",
        accent.0,
        accent.1,
        accent.2,
        key = key,
        width = width
    )
}

fn stat_row(key: &str, value: &str, label_w: usize, pal: &MotdAnsiPalette) -> String {
    format!(
        "{}  \x1b[38;2;{};{};{}m{value}\x1b[0m",
        lbl_col(key, label_w, pal.accent),
        pal.body.0,
        pal.body.1,
        pal.body.2,
    )
}

fn palette_footer(pal: &MotdAnsiPalette) -> String {
    let colors: [(u8, u8, u8); 8] = [
        pal.accent,
        (42, 28, 92),
        (88, 48, 138),
        (168, 92, 118),
        (255, 238, 168),
        (52, 36, 118),
        (36, 26, 82),
        (255, 252, 255),
    ];
    let mut s = String::new();
    let _ = write!(
        s,
        "\x1b[38;2;{};{};{}m.\x1b[0m ",
        pal.dim.0,
        pal.dim.1,
        pal.dim.2
    );
    for &(r, g, b) in &colors {
        let _ = write!(s, "\x1b[38;2;{r};{g};{b}mo\x1b[0m  ");
    }
    s.trim_end().to_string()
}

fn gather_right_column(pal: &MotdAnsiPalette) -> Vec<String> {
    const KW: usize = 10;
    let host = hostname_str();
    let user = username();
    let head = format!(
        "\x1b[1m\x1b[38;2;{};{};{}m{user}\x1b[0m\x1b[38;2;{};{};{}m@\x1b[0m\x1b[1m\x1b[38;2;{};{};{}m{host}\x1b[0m",
        pal.accent.0,
        pal.accent.1,
        pal.accent.2,
        pal.dim.0,
        pal.dim.1,
        pal.dim.2,
        pal.body.0,
        pal.body.1,
        pal.body.2,
    );
    let sep_n = format!("{user}@{host}").chars().count().clamp(28, 44);
    let sep: String = std::iter::repeat('─').take(sep_n).collect();
    let sep_line = format!(
        "\x1b[38;2;{};{};{}m{sep}\x1b[0m",
        pal.sep.0,
        pal.sep.1,
        pal.sep.2
    );

    let mut lines = vec![head, sep_line];
    let pairs: [(&str, String); 9] = [
        ("OS", os_pretty()),
        ("Host", machine_model()),
        ("Kernel", kernel_line()),
        ("Uptime", uptime_line()),
        ("Shell", shell_env()),
        ("DE", desktop_env()),
        ("Terminal", "Arcadia".into()),
        ("CPU", cpu_model()),
        ("Memory", memory_line()),
    ];
    for (k, v) in pairs {
        lines.push(stat_row(k, &v, KW, pal));
    }
    lines.push(palette_footer(pal));
    lines
}

fn merge_ansi(left: &[String], right: &[String]) -> Vec<String> {
    let lh = left.len();
    let rh = right.len();
    let max_h = lh.max(rh);
    let pad_l_top = (max_h - lh) / 2;
    let pad_l_bot = max_h.saturating_sub(pad_l_top + lh);
    let pad_r_top = (max_h - rh) / 2;
    let pad_r_bot = max_h.saturating_sub(pad_r_top + rh);

    let mut lpadded = vec![String::new(); pad_l_top];
    lpadded.extend(left.iter().cloned());
    lpadded.extend(std::iter::repeat_with(|| String::new()).take(pad_l_bot));

    let mut rpadded = vec![String::new(); pad_r_top];
    rpadded.extend(right.iter().cloned());
    rpadded.extend(std::iter::repeat_with(|| String::new()).take(pad_r_bot));

    let max_vw = lpadded.iter().map(|s| vw(s)).max().unwrap_or(0);
    let gap = " ".repeat(6);
    lpadded
        .into_iter()
        .zip(rpadded.into_iter())
        .map(|(l, r)| {
            let pad = " ".repeat(max_vw.saturating_sub(vw(&l)));
            format!("{l}{pad}{gap}{r}")
        })
        .collect()
}

// ── public API ────────────────────────────────────────────────────────────────

fn show(_args: &[&str], _ctx: &ExecutionContext) -> String {
    motd_string()
}

pub fn commands() -> &'static [ModuleCommand] {
    &[ModuleCommand {
        name: "show",
        description: "print Arcadia-style system banner (MOTD)",
        run: show,
    }]
}

pub fn motd_string() -> String {
    motd_lines().join("\n")
}

/// Shell transcript / CLI MOTD using inferred scheme ([`guess_terminal_scheme_dark`]).
pub fn motd_lines() -> Vec<String> {
    motd_lines_for_scheme(guess_terminal_scheme_dark())
}

pub fn motd_lines_for_scheme(is_dark: bool) -> Vec<String> {
    let pal = motd_palette(is_dark);
    merge_ansi(&arch_art_lines(), &gather_right_column(&pal))
}
