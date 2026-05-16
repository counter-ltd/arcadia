use super::text::next_char_boundary;
use arcadia_core::modules::python_registry::HighlightSpan;

fn parse_highlight_color(token: &str) -> Option<u32> {
    let hex = token.trim_start_matches('#');
    if hex.len() == 6 {
        u32::from_str_radix(hex, 16).ok()
    } else {
        None
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum SegKind {
    Normal,
    Selected,
    Cursor,
}

pub fn line_segments(
    line: &str,
    line_byte_start: usize,
    cursor_byte: Option<usize>,
    sel: Option<(usize, usize)>,
    hl_spans: &[HighlightSpan],
) -> Vec<(String, SegKind, Option<u32>)> {
    // Convert doc-absolute highlight spans to line-relative.
    let line_end_doc = line_byte_start + line.len();
    let local_spans: Vec<(usize, usize, u32)> = hl_spans
        .iter()
        .filter_map(|s| {
            if s.end <= line_byte_start || s.start >= line_end_doc {
                return None;
            }
            let ls = s.start.saturating_sub(line_byte_start).min(line.len());
            let le = s.end.saturating_sub(line_byte_start).min(line.len());
            let color = parse_highlight_color(&s.token)?;
            if ls < le {
                Some((ls, le, color))
            } else {
                None
            }
        })
        .collect();

    let eol_cursor = cursor_byte.map_or(false, |c| c >= line.len());
    let cursor_end = cursor_byte.map(|c| next_char_boundary(line, c));

    let mut splits: Vec<usize> = vec![0, line.len()];
    if let Some((s, e)) = sel {
        splits.push(s.min(line.len()));
        splits.push(e.min(line.len()));
    }
    if let Some(c) = cursor_byte {
        let c = c.min(line.len());
        splits.push(c);
        if let Some(ce) = cursor_end {
            splits.push(ce.min(line.len()));
        }
    }
    for &(s, e, _) in &local_spans {
        splits.push(s);
        splits.push(e);
    }
    splits.sort_unstable();
    splits.dedup();

    let mut result: Vec<(String, SegKind, Option<u32>)> = Vec::new();
    for w in splits.windows(2) {
        let (s, e) = (w[0], w[1]);
        if s >= e {
            continue;
        }
        let is_cursor_seg =
            !eol_cursor && cursor_byte.map_or(false, |c| s >= c && e <= cursor_end.unwrap_or(c));
        let is_sel = sel.map_or(false, |(ss, se)| s >= ss && e <= se);
        let kind = if is_cursor_seg {
            SegKind::Cursor
        } else if is_sel {
            SegKind::Selected
        } else {
            SegKind::Normal
        };
        let hl_color = if kind == SegKind::Normal {
            local_spans
                .iter()
                .find(|(ls, le, _)| s >= *ls && e <= *le)
                .map(|(_, _, c)| *c)
        } else {
            None
        };
        if let Some(last) = result.last_mut() {
            if last.1 == kind && last.2 == hl_color {
                last.0.push_str(&line[s..e]);
                continue;
            }
        }
        result.push((line[s..e].to_string(), kind, hl_color));
    }
    if eol_cursor {
        result.push((" ".to_string(), SegKind::Cursor, None));
    }
    if result.is_empty() {
        result.push((" ".to_string(), SegKind::Normal, None));
    }
    result
}
