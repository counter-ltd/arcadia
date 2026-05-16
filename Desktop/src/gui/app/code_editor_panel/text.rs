use openframe::{px, Bounds, Pixels};

/// Expand `\t` to 4 spaces for visual rendering.
pub fn expand_tabs(s: &str) -> String {
    if !s.contains('\t') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        if c == '\t' {
            out.push_str("    ");
        } else {
            out.push(c);
        }
    }
    out
}

/// Display width of a char in grid cells: tab = 4, wide (CJK/emoji) = 2, else 1.
/// Single source of truth for the editor's monospace grid.
pub fn char_cols(c: char) -> usize {
    if c == '\t' {
        return 4;
    }
    let cp = c as u32;
    let wide = (0x1100..=0x115F).contains(&cp)
        || (0x2E80..=0x303E).contains(&cp)
        || (0x3041..=0x33FF).contains(&cp)
        || (0x3400..=0x4DBF).contains(&cp)
        || (0x4E00..=0x9FFF).contains(&cp)
        || (0xA000..=0xA4CF).contains(&cp)
        || (0xAC00..=0xD7A3).contains(&cp)
        || (0xF900..=0xFAFF).contains(&cp)
        || (0xFE30..=0xFE4F).contains(&cp)
        || (0xFF00..=0xFF60).contains(&cp)
        || (0xFFE0..=0xFFE6).contains(&cp)
        || (0x1F300..=0x1FAFF).contains(&cp)
        || (0x20000..=0x3FFFD).contains(&cp);
    if wide {
        2
    } else {
        1
    }
}

/// Map a visual column (tabs = 4, wide chars = 2) back to a byte offset in the original line.
pub fn vis_col_to_orig_byte(line: &str, vis_col: usize) -> usize {
    let mut col = 0;
    for (i, c) in line.char_indices() {
        if col >= vis_col {
            return i;
        }
        col += char_cols(c);
    }
    line.len()
}

pub fn prev_char_boundary(s: &str, pos: usize) -> usize {
    let mut p = pos;
    while p > 0 {
        p -= 1;
        if s.is_char_boundary(p) {
            break;
        }
    }
    p
}

pub fn next_char_boundary(s: &str, pos: usize) -> usize {
    if pos >= s.len() {
        return pos;
    }
    let mut p = pos + 1;
    while p < s.len() && !s.is_char_boundary(p) {
        p += 1;
    }
    p
}

pub fn word_bounds(content: &str, pos: usize) -> (usize, usize) {
    fn is_word(c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }
    let chars_before: Vec<(usize, char)> = content[..pos].char_indices().collect();
    let mut start = pos;
    for &(i, c) in chars_before.iter().rev() {
        if is_word(c) {
            start = i;
        } else {
            break;
        }
    }
    let mut end = pos;
    for (i, c) in content[pos..].char_indices() {
        if is_word(c) {
            end = pos + i + c.len_utf8();
        } else {
            break;
        }
    }
    if start == end {
        let e = next_char_boundary(content, pos).min(content.len());
        (pos, e)
    } else {
        (start, end)
    }
}

pub fn delete_selection(
    content: &mut String,
    cursor: &mut usize,
    anchor: &mut Option<usize>,
) -> bool {
    if let Some(a) = *anchor {
        let (sel_start, sel_end) = if a <= *cursor {
            (a, *cursor)
        } else {
            (*cursor, a)
        };
        content.drain(sel_start..sel_end);
        *cursor = sel_start;
        *anchor = None;
        true
    } else {
        false
    }
}

pub fn pos_to_byte_offset(
    content: &str,
    line_bounds: &[Bounds<Pixels>],
    event_pos: openframe::Point<Pixels>,
    gutter_w: Pixels,
    char_width: f32,
) -> usize {
    if line_bounds.is_empty() {
        return content.len();
    }
    let line_idx = line_bounds
        .iter()
        .rposition(|b| event_pos.y >= b.origin.y)
        .unwrap_or(0)
        .min(line_bounds.len() - 1);

    let line_x = line_bounds[line_idx].origin.x;
    // Must match the content line's left padding (`base_div` `.px_1()` in view.rs).
    let content_x = line_x + gutter_w + px(4.0);
    let col_x = f32::from(event_pos.x - content_x).max(0.0);
    let col_chars = (col_x / char_width).floor() as usize;

    let mut line_start = 0usize;
    let mut cur_line = 0usize;
    for (i, c) in content.char_indices() {
        if cur_line == line_idx {
            break;
        }
        if c == '\n' {
            cur_line += 1;
            line_start = i + 1;
        }
    }
    if cur_line < line_idx {
        return content.len();
    }
    let line_end = content[line_start..]
        .find('\n')
        .map(|i| line_start + i)
        .unwrap_or(content.len());
    let line_text = &content[line_start..line_end];
    let byte_in_line = vis_col_to_orig_byte(line_text, col_chars);
    line_start + byte_in_line
}

pub fn detect_language(filename: &str) -> Option<String> {
    let ext = filename.rsplit('.').next()?;
    let lang = match ext {
        "rs" => "rust",
        "py" => "python",
        "js" => "javascript",
        "ts" => "typescript",
        "toml" => "toml",
        "json" => "json",
        "yaml" | "yml" => "yaml",
        "md" => "markdown",
        _ => return None,
    };
    Some(lang.to_string())
}
