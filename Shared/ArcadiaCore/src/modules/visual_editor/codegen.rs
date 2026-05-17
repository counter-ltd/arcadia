//! Pure text-splice helpers for block → source editing.
//!
//! Every block carries the byte range of its CST node, so a structural edit is
//! expressed as a splice on the canonical source buffer. After any splice the
//! caller re-parses — there is no separately-mutated block tree to keep in sync.

/// Replace bytes `[start, end)` of `source` with `text`.
pub fn replace_region(source: &str, start: usize, end: usize, text: &str) -> String {
    let start = start.min(source.len());
    let end = end.clamp(start, source.len());
    // Snap to char boundaries so slicing never panics on multi-byte input.
    let start = floor_boundary(source, start);
    let end = floor_boundary(source, end);
    let mut out = String::with_capacity(source.len() + text.len());
    out.push_str(&source[..start]);
    out.push_str(text);
    out.push_str(&source[end..]);
    out
}

/// Expand `[start, end)` to whole lines: back to the byte after the previous
/// `\n`, forward through the `\n` that terminates the last line.
pub fn line_bounds(source: &str, start: usize, end: usize) -> (usize, usize) {
    let start = start.min(source.len());
    let end = end.clamp(start, source.len());
    let ls = source[..start].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let le = source[end..]
        .find('\n')
        .map(|i| end + i + 1)
        .unwrap_or(source.len());
    (ls, le)
}

/// Byte offset of the end of the line `pos` sits on (exclusive of the `\n`).
pub fn line_end(source: &str, pos: usize) -> usize {
    let pos = pos.min(source.len());
    source[pos..]
        .find('\n')
        .map(|i| pos + i)
        .unwrap_or(source.len())
}

/// Delete the block occupying `[start, end)`, removing its whole line(s).
pub fn delete_block(source: &str, start: usize, end: usize) -> String {
    let (ls, le) = line_bounds(source, start, end);
    replace_region(source, ls, le, "")
}

/// Swap two sibling regions, each expanded to whole lines. `a` must lie before
/// `b`; overlapping or mis-ordered ranges return `source` unchanged. Leading
/// indentation travels with each block, so sibling order is preserved visually.
pub fn swap_blocks(
    source: &str,
    a_start: usize,
    a_end: usize,
    b_start: usize,
    b_end: usize,
) -> String {
    let (als, ale) = line_bounds(source, a_start, a_end);
    let (bls, ble) = line_bounds(source, b_start, b_end);
    if ale > bls {
        return source.to_string();
    }
    let mut out = String::with_capacity(source.len());
    out.push_str(&source[..als]);
    out.push_str(&source[bls..ble]);
    out.push_str(&source[ale..bls]);
    out.push_str(&source[als..ale]);
    out.push_str(&source[ble..]);
    out
}

/// Leading whitespace (the indentation) of the line `pos` sits on.
pub fn indent_at(source: &str, pos: usize) -> String {
    let pos = pos.min(source.len());
    let ls = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
    source[ls..]
        .chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .collect()
}

/// Insert `snippet` as new line(s) after the line containing `anchor`. Every
/// snippet line is prefixed with `indent`; a leading newline is added when the
/// anchor line has no trailing one (end of a file without a final newline).
pub fn insert_block(source: &str, anchor: usize, indent: &str, snippet: &str) -> String {
    let (_, le) = line_bounds(source, anchor, anchor);
    let body: String = snippet
        .split('\n')
        .map(|line| {
            if line.is_empty() {
                String::new()
            } else {
                format!("{indent}{line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let needs_lead_nl = le > 0 && !source[..le].ends_with('\n');
    let mut ins = String::new();
    if needs_lead_nl {
        ins.push('\n');
    }
    ins.push_str(&body);
    ins.push('\n');
    replace_region(source, le, le, &ins)
}

fn floor_boundary(source: &str, mut idx: usize) -> usize {
    while idx > 0 && !source.is_char_boundary(idx) {
        idx -= 1;
    }
    idx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_swaps_region() {
        assert_eq!(replace_region("abcdef", 2, 4, "XY"), "abXYef");
        assert_eq!(replace_region("abc", 0, 3, ""), "");
    }

    #[test]
    fn line_bounds_covers_whole_lines() {
        let s = "aaa\nbbb\nccc\n";
        // span inside "bbb"
        assert_eq!(line_bounds(s, 5, 6), (4, 8));
    }

    #[test]
    fn delete_removes_line() {
        let s = "import os\nx = 1\nprint(x)\n";
        // delete the "x = 1" line (bytes 10..15)
        assert_eq!(delete_block(s, 10, 15), "import os\nprint(x)\n");
    }

    #[test]
    fn swap_reorders_siblings() {
        let s = "a = 1\nb = 2\n";
        // swap the two assignment lines
        assert_eq!(swap_blocks(s, 0, 5, 6, 11), "b = 2\na = 1\n");
    }

    #[test]
    fn swap_rejects_overlap() {
        let s = "a = 1\nb = 2\n";
        assert_eq!(swap_blocks(s, 0, 5, 2, 4), s);
    }

    #[test]
    fn line_end_stops_before_newline() {
        assert_eq!(line_end("if a:\n    pass\n", 0), 5);
    }

    #[test]
    fn indent_at_reads_leading_whitespace() {
        let s = "def f():\n    x = 1\n";
        assert_eq!(indent_at(s, 13), "    ");
        assert_eq!(indent_at(s, 0), "");
    }

    #[test]
    fn insert_block_indents_and_appends() {
        let s = "x = 1\n";
        assert_eq!(insert_block(s, 0, "", "y = 2"), "x = 1\ny = 2\n");
    }

    #[test]
    fn insert_block_indents_multiline_snippet() {
        let s = "def f():\n    pass\n";
        let out = insert_block(s, 17, "    ", "if a:\n    pass");
        assert_eq!(out, "def f():\n    pass\n    if a:\n        pass\n");
    }

    #[test]
    fn insert_block_adds_leading_newline_when_missing() {
        assert_eq!(insert_block("x = 1", 0, "", "y = 2"), "x = 1\ny = 2\n");
    }
}
