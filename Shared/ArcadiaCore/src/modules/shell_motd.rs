//! Arcadia MOTD — app-icon scene (parabolic arch, rising sun, twin hills, stars) + system info.

use crate::config::appearance::AppearanceConfig;
use crate::config::extension_tokens;
use crate::config::ConfigFile;
use crate::modules::python_registry::{list_styles, GlyphParams};
use crate::modules::{ExecutionContext, ModuleCommand};

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
    let active = AppearanceConfig::load_or_create()
        .unwrap_or_default()
        .active_style;
    let style_row = list_styles().into_iter().find(|s| s.name == active)?;
    let mut g = if is_dark {
        style_row.glyph.clone()
    } else {
        style_row
            .glyph_light
            .clone()
            .or_else(|| style_row.glyph.clone())
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

fn lerp(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).round() as u8
}

fn lerp3(a: (u8, u8, u8), b: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
    (lerp(a.0, b.0, t), lerp(a.1, b.1, t), lerp(a.2, b.2, t))
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

/// Pixel scene matching the app icon: a single white parabolic arch over a night
/// sky, a sun rising between two hills, stars, on a rounded-square gradient field.
fn arch_art_lines() -> Vec<String> {
    use std::f32::consts::PI;

    const W: usize = 28;
    const H: usize = 13;
    let wf = W as f32;
    let hf = H as f32;
    let cx = wf / 2.0 - 0.5;
    // Terminal cells are ~8.4w × 18h — compress x so discs read as round.
    let ax = 8.4_f32 / 18.0_f32;

    // ── palette ──────────────────────────────────────────────────────────────
    let sky_top: (u8, u8, u8) = (94, 62, 182);
    let sky_mid: (u8, u8, u8) = (150, 92, 186);
    let sky_bot: (u8, u8, u8) = (236, 152, 178);
    let night_top: (u8, u8, u8) = (30, 24, 72);
    let night_bot: (u8, u8, u8) = (74, 48, 112);
    let arch_core: (u8, u8, u8) = (250, 248, 255);
    let arch_soft: (u8, u8, u8) = (214, 202, 242);
    let sun_core: (u8, u8, u8) = (255, 247, 208);
    let sun_mid: (u8, u8, u8) = (252, 223, 156);
    let sun_glow: (u8, u8, u8) = (244, 174, 128);
    let hill_back: (u8, u8, u8) = (112, 78, 170);
    let hill_front: (u8, u8, u8) = (72, 48, 124);
    let star_dim: (u8, u8, u8) = (224, 218, 246);
    let star_bright: (u8, u8, u8) = (255, 255, 250);

    // ── geometry ─────────────────────────────────────────────────────────────
    // Arch follows a parabola: peak high at centre, arms sweeping to the corners.
    let y_peak = 1.5_f32;
    let k = (hf - 0.6 - y_peak) / (cx * cx);
    let stroke = 1.15_f32; // half-thickness of the white arch band, in rows

    let hill_base = hf - 2.3;
    let sun_cx = cx;
    let sun_cy = hill_base - 0.2;
    let sun_r = 3.05_f32;

    // Smooth 0..1 hump centred at `center`, zero beyond `half`.
    let bump = |x: f32, center: f32, half: f32| -> f32 {
        let u = (x - center) / half;
        if u.abs() < 1.0 {
            0.5 * (1.0 + (PI * u).cos())
        } else {
            0.0
        }
    };

    // Stars — placed in the interior night sky, clear of the arch band and sun.
    let stars: &[(usize, usize)] = &[
        (3, 12),
        (3, 16),
        (4, 9),
        (4, 14),
        (4, 18),
        (5, 8),
        (5, 15),
        (5, 20),
        (6, 11),
        (6, 19),
    ];

    let mut out = Vec::with_capacity(H);
    for r in 0..H {
        let rf = r as f32;
        let mut line = String::new();
        for c in 0..W {
            let cf = c as f32;

            let para_y = y_peak + k * (cf - cx) * (cf - cx);
            let vt = (rf / (hf - 1.0)).clamp(0.0, 1.0);

            // Hills (foreground) — the sun rises in the dip between them.
            let back_top = hill_base - 1.5 * bump(cf, 0.33 * wf, 0.40 * wf);
            let front_top = hill_base + 0.5 - 1.8 * bump(cf, 0.68 * wf, 0.44 * wf);
            if rf >= front_top {
                line.push_str(&px(hill_front.0, hill_front.1, hill_front.2));
                continue;
            }
            if rf >= back_top {
                line.push_str(&px(hill_back.0, hill_back.1, hill_back.2));
                continue;
            }

            // White arch band straddling the parabola.
            if (rf - para_y).abs() <= stroke {
                let edge = (rf - para_y).abs() / stroke;
                let col = lerp3(arch_core, arch_soft, edge.powf(1.6));
                line.push_str(&px(col.0, col.1, col.2));
                continue;
            }

            if rf > para_y {
                // Interior: night sky, rising sun, stars.
                let dx = (cf - sun_cx) * ax;
                let dy = rf - sun_cy;
                let d = (dx * dx + dy * dy).sqrt();
                if d <= sun_r {
                    let col = if d <= sun_r * 0.46 {
                        sun_core
                    } else if d <= sun_r * 0.78 {
                        lerp3(sun_core, sun_mid, (d / sun_r - 0.46) / 0.32)
                    } else {
                        lerp3(sun_mid, sun_glow, (d / sun_r - 0.78) / 0.22)
                    };
                    line.push_str(&px(col.0, col.1, col.2));
                    continue;
                }
                let mut col = lerp3(night_top, night_bot, vt);
                let bloom = (1.0 - (d / (sun_r * 2.6)).min(1.0)).powf(2.0);
                col = lerp3(col, sun_glow, bloom * 0.5);
                if stars.iter().any(|&(sr, sc)| sr == r && sc == c) {
                    let s = if (r + c) % 3 == 0 {
                        star_bright
                    } else {
                        star_dim
                    };
                    line.push_str(&px(s.0, s.1, s.2));
                } else {
                    line.push_str(&px(col.0, col.1, col.2));
                }
                continue;
            }

            // Outer sky — indigo aloft warming to dusk pink at the horizon.
            let col = if vt < 0.5 {
                lerp3(sky_top, sky_mid, vt * 2.0)
            } else {
                lerp3(sky_mid, sky_bot, (vt - 0.5) * 2.0)
            };
            line.push_str(&px(col.0, col.1, col.2));
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

fn version_line(pal: &MotdAnsiPalette) -> String {
    format!(
        "\x1b[38;2;{};{};{}mArcadia\x1b[0m \x1b[38;2;{};{};{}m{}\x1b[0m",
        pal.accent.0,
        pal.accent.1,
        pal.accent.2,
        pal.dim.0,
        pal.dim.1,
        pal.dim.2,
        env!("CARGO_PKG_VERSION"),
    )
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
        pal.sep.0, pal.sep.1, pal.sep.2
    );

    let mut lines = vec![head, sep_line.clone()];
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
    lines.push(sep_line);
    lines.push(version_line(pal));
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
        required_permissions: &[],
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
