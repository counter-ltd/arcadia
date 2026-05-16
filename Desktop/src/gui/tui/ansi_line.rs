//! Parse ANSI SGR sequences in plain strings for shell transcript rendering.

use openframe::{div, px, Div, FontWeight, ParentElement, Rgba, Styled};

const TRANSCRIPT_ROW_H: f32 = 18.0;

use super::colors::{self};
use crate::gui::assets::MONO_FONT_FAMILY;

#[derive(Clone, Copy)]
struct StyleState {
    fg: Rgba,
    bg: Option<Rgba>,
    bold: bool,
}

fn rgba_u8(r: u32, g: u32, b: u32) -> Rgba {
    Rgba {
        r: (r.min(255)) as f32 / 255.0,
        g: (g.min(255)) as f32 / 255.0,
        b: (b.min(255)) as f32 / 255.0,
        a: 1.0,
    }
}

fn sgr_fg_basic(code: u32, is_dark: bool) -> Option<Rgba> {
    match code {
        30..=37 => Some(colors::ansi_indexed((code - 30) as u8, is_dark)),
        90..=97 => Some(colors::ansi_indexed((code - 90 + 8) as u8, is_dark)),
        _ => None,
    }
}

fn sgr_bg_basic(code: u32, is_dark: bool) -> Option<Rgba> {
    match code {
        40..=47 => Some(colors::ansi_indexed((code - 40) as u8, is_dark)),
        100..=107 => Some(colors::ansi_indexed((code - 100 + 8) as u8, is_dark)),
        _ => None,
    }
}

fn parse_sgr_params(raw: &str) -> Vec<u32> {
    if raw.is_empty() {
        return vec![0];
    }
    raw.split(';')
        .map(|s| {
            if s.is_empty() {
                0
            } else {
                s.parse::<u32>().unwrap_or(0)
            }
        })
        .collect()
}

fn apply_sgr(codes: &[u32], st: &mut StyleState, default_fg: Rgba, is_dark: bool) {
    let mut i = 0usize;
    while i < codes.len() {
        match codes[i] {
            0 => {
                st.fg = default_fg;
                st.bg = None;
                st.bold = false;
            }
            1 => st.bold = true,
            22 => st.bold = false,
            39 => st.fg = default_fg,
            49 => st.bg = None,
            n @ 30..=37 | n @ 90..=97 => {
                if let Some(c) = sgr_fg_basic(n, is_dark) {
                    st.fg = c;
                }
            }
            n @ 40..=47 | n @ 100..=107 => {
                if let Some(c) = sgr_bg_basic(n, is_dark) {
                    st.bg = Some(c);
                }
            }
            38 => {
                if codes.get(i + 1) == Some(&5) && i + 2 < codes.len() {
                    st.fg = colors::ansi_indexed(codes[i + 2].min(255) as u8, is_dark);
                    i += 3;
                    continue;
                }
                // Params: 38;2;r;g;b → indices i..i+4 inclusive (len ≥ i+5).
                if codes.get(i + 1) == Some(&2) && i + 4 < codes.len() {
                    st.fg = rgba_u8(codes[i + 2], codes[i + 3], codes[i + 4]);
                    i += 5;
                    continue;
                }
            }
            48 => {
                if codes.get(i + 1) == Some(&5) && i + 2 < codes.len() {
                    st.bg = Some(colors::ansi_indexed(codes[i + 2].min(255) as u8, is_dark));
                    i += 3;
                    continue;
                }
                if codes.get(i + 1) == Some(&2) && i + 4 < codes.len() {
                    st.bg = Some(rgba_u8(codes[i + 2], codes[i + 3], codes[i + 4]));
                    i += 5;
                    continue;
                }
            }
            _ => {}
        }
        i += 1;
    }
}

fn skip_osc(it: &mut std::iter::Peekable<std::str::Chars>) {
    while let Some(c) = it.next() {
        if c == '\x07' {
            break;
        }
        if c == '\x1b' && it.peek() == Some(&'\\') {
            let _ = it.next();
            break;
        }
    }
}

#[derive(Clone)]
struct Run {
    text: String,
    fg: Rgba,
    bg: Option<Rgba>,
    bold: bool,
}

fn rgba_style_eq(a: Rgba, b: Rgba) -> bool {
    (a.r - b.r).abs() < f32::EPSILON
        && (a.g - b.g).abs() < f32::EPSILON
        && (a.b - b.b).abs() < f32::EPSILON
        && (a.a - b.a).abs() < f32::EPSILON
}

fn flush_run(buf: &mut String, runs: &mut Vec<Run>, st: StyleState) {
    if buf.is_empty() {
        return;
    }
    let text = std::mem::take(buf);
    if let Some(prev) = runs.last_mut() {
        let bg_match = match (prev.bg, st.bg) {
            (Some(a), Some(b)) => rgba_style_eq(a, b),
            (None, None) => true,
            _ => false,
        };
        if rgba_style_eq(prev.fg, st.fg) && bg_match && prev.bold == st.bold {
            prev.text.push_str(&text);
            return;
        }
    }
    runs.push(Run {
        text,
        fg: st.fg,
        bg: st.bg,
        bold: st.bold,
    });
}

fn parse_ansi_runs(line: &str, default_fg: Rgba, is_dark: bool) -> Vec<Run> {
    let mut runs = Vec::new();
    let mut buf = String::new();
    let mut st = StyleState {
        fg: default_fg,
        bg: None,
        bold: false,
    };
    let mut it = line.chars().peekable();

    while let Some(ch) = it.next() {
        if ch == '\r' {
            continue;
        }
        if ch == '\x1b' {
            flush_run(&mut buf, &mut runs, st);
            match it.peek().copied() {
                Some('[') => {
                    it.next();
                    let mut raw = String::new();
                    let mut closed = false;
                    while let Some(c) = it.next() {
                        if ('\x40'..='\x7e').contains(&c) {
                            if c == 'm' {
                                apply_sgr(&parse_sgr_params(&raw), &mut st, default_fg, is_dark);
                            }
                            closed = true;
                            break;
                        }
                        raw.push(c);
                    }
                    if !closed {
                        break;
                    }
                }
                Some(']') => {
                    it.next();
                    skip_osc(&mut it);
                }
                Some('(') | Some(')') => {
                    let _ = it.next();
                    let _ = it.next();
                }
                _ => {}
            }
            continue;
        }
        buf.push(ch);
    }
    flush_run(&mut buf, &mut runs, st);
    runs
}

pub(crate) fn shell_history_line(line: &str, is_dark: bool, cell_w: f32) -> Div {
    let default_fg = colors::default_fg(is_dark);

    let runs = parse_ansi_runs(line, default_fg, is_dark);
    if runs.is_empty() {
        return div().w_full().h(px(0.)).flex_shrink_0().overflow_hidden();
    }

    div()
        .w_full()
        .flex_shrink_0()
        .h(px(TRANSCRIPT_ROW_H))
        .line_height(px(TRANSCRIPT_ROW_H))
        .flex()
        .flex_row()
        .items_center()
        .overflow_hidden()
        .font_family(MONO_FONT_FAMILY)
        .text_sm()
        .children(runs.into_iter().flat_map(move |run| {
            // One fixed-width cell per character. Per-run boxes accumulated
            // sub-pixel rounding across runs, leaving the right edge ragged on
            // the multi-coloured pixel-art rows; a per-char grid is exact.
            let weight = if run.bold {
                FontWeight::BOLD
            } else {
                FontWeight::NORMAL
            };
            let fg = run.fg;
            let bg = run.bg;
            run.text
                .chars()
                .collect::<Vec<char>>()
                .into_iter()
                .map(move |ch| {
                    let mut cell = div()
                        .flex_shrink_0()
                        .w(px(cell_w))
                        .overflow_hidden()
                        .text_color(fg)
                        .font_weight(weight)
                        .child(ch.to_string());
                    if let Some(b) = bg {
                        cell = cell.bg(b);
                    }
                    cell
                })
                .collect::<Vec<_>>()
        }))
}
