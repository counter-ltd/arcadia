use super::commands::EditorUndo;
use super::text::{delete_selection, next_char_boundary, prev_char_boundary};
use crate::gui::app::CodeEditorTab;
use openframe::Keystroke;

/// Quality-of-life behaviour flags for `apply_key`, sourced from `CodeEditorConfig`.
#[derive(Clone, Copy)]
pub struct EditCtx {
    pub auto_indent: bool,
    pub auto_close: bool,
    pub record_undo: bool,
}

fn line_start(s: &str, pos: usize) -> usize {
    s[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0)
}

fn snapshot(tab: &CodeEditorTab) -> (String, usize, Option<usize>) {
    (tab.content.clone(), tab.cursor, tab.selection_anchor)
}

/// Apply a keystroke to a tab's content/cursor/selection. The view layer owns
/// highlight invalidation and persistence; this owns undo recording.
pub fn apply_key(tab: &mut CodeEditorTab, undo: &mut EditorUndo, ctx: EditCtx, keystroke: &Keystroke) {
    let key = keystroke.key.as_str();
    let mods = keystroke.modifiers;

    if (mods.control || mods.platform) && key == "a" {
        tab.selection_anchor = Some(0);
        tab.cursor = tab.content.len();
        undo.coalescing = false;
        return;
    }

    match key {
        "backspace" => {
            if ctx.record_undo {
                undo.push(snapshot(tab));
                undo.coalescing = false;
            }
            if !delete_selection(&mut tab.content, &mut tab.cursor, &mut tab.selection_anchor) {
                if tab.cursor > 0 {
                    let ls = line_start(&tab.content, tab.cursor);
                    let before = &tab.content[ls..tab.cursor];
                    if !before.is_empty() && before.bytes().all(|b| b == b' ') {
                        // Inside leading indentation — delete back to the
                        // previous 4-column tab stop, not a single space.
                        let col = before.len();
                        let del = if col % 4 == 0 { 4 } else { col % 4 };
                        tab.content.drain(tab.cursor - del..tab.cursor);
                        tab.cursor -= del;
                    } else {
                        let prev = prev_char_boundary(&tab.content, tab.cursor);
                        tab.content.drain(prev..tab.cursor);
                        tab.cursor = prev;
                    }
                }
            }
        }
        "delete" => {
            if ctx.record_undo {
                undo.push(snapshot(tab));
                undo.coalescing = false;
            }
            if !delete_selection(&mut tab.content, &mut tab.cursor, &mut tab.selection_anchor) {
                if tab.cursor < tab.content.len() {
                    let next = next_char_boundary(&tab.content, tab.cursor);
                    tab.content.drain(tab.cursor..next);
                }
            }
        }
        "enter" => {
            if ctx.record_undo {
                undo.push(snapshot(tab));
                undo.coalescing = false;
            }
            delete_selection(&mut tab.content, &mut tab.cursor, &mut tab.selection_anchor);
            // Auto-indent: carry the leading whitespace of the current line.
            let lead: String = if ctx.auto_indent {
                let ls = line_start(&tab.content, tab.cursor);
                tab.content[ls..]
                    .chars()
                    .take_while(|&c| c == ' ' || c == '\t')
                    .collect()
            } else {
                String::new()
            };
            tab.content.insert(tab.cursor, '\n');
            tab.cursor += 1;
            if !lead.is_empty() {
                tab.content.insert_str(tab.cursor, &lead);
                tab.cursor += lead.len();
            }
        }
        "tab" => {
            if ctx.record_undo {
                undo.push(snapshot(tab));
                undo.coalescing = false;
            }
            delete_selection(&mut tab.content, &mut tab.cursor, &mut tab.selection_anchor);
            tab.content.insert_str(tab.cursor, "    ");
            tab.cursor += 4;
        }
        "left" => {
            undo.coalescing = false;
            if mods.shift {
                if tab.selection_anchor.is_none() {
                    tab.selection_anchor = Some(tab.cursor);
                }
            } else {
                tab.selection_anchor = None;
            }
            if tab.cursor > 0 {
                tab.cursor = prev_char_boundary(&tab.content, tab.cursor);
            }
        }
        "right" => {
            undo.coalescing = false;
            if mods.shift {
                if tab.selection_anchor.is_none() {
                    tab.selection_anchor = Some(tab.cursor);
                }
            } else {
                tab.selection_anchor = None;
            }
            if tab.cursor < tab.content.len() {
                tab.cursor = next_char_boundary(&tab.content, tab.cursor);
            }
        }
        "up" => {
            if mods.alt {
                return; // Alt+Up is the move-line-up command shortcut.
            }
            undo.coalescing = false;
            if mods.shift && tab.selection_anchor.is_none() {
                tab.selection_anchor = Some(tab.cursor);
            } else if !mods.shift {
                tab.selection_anchor = None;
            }
            let before = &tab.content[..tab.cursor];
            let ls = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
            let col = before[ls..].chars().count();
            if ls > 0 {
                let prev_nl_end = ls - 1;
                let prev_before = &tab.content[..prev_nl_end];
                let prev_line_start = prev_before.rfind('\n').map(|i| i + 1).unwrap_or(0);
                let prev_line = &tab.content[prev_line_start..prev_nl_end];
                let target = col.min(prev_line.chars().count());
                tab.cursor = prev_line_start
                    + prev_line
                        .char_indices()
                        .nth(target)
                        .map(|(i, _)| i)
                        .unwrap_or(prev_line.len());
            }
        }
        "down" => {
            if mods.alt {
                return; // Alt+Down is the move-line-down command shortcut.
            }
            undo.coalescing = false;
            if mods.shift && tab.selection_anchor.is_none() {
                tab.selection_anchor = Some(tab.cursor);
            } else if !mods.shift {
                tab.selection_anchor = None;
            }
            let before = &tab.content[..tab.cursor];
            let ls = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
            let col = before[ls..].chars().count();
            let after = &tab.content[tab.cursor..];
            if let Some(nl_offset) = after.find('\n') {
                let next_start = tab.cursor + nl_offset + 1;
                let next_content = &tab.content[next_start..];
                let next_line =
                    &next_content[..next_content.find('\n').unwrap_or(next_content.len())];
                let target = col.min(next_line.chars().count());
                tab.cursor = next_start
                    + next_line
                        .char_indices()
                        .nth(target)
                        .map(|(i, _)| i)
                        .unwrap_or(next_line.len());
            } else {
                tab.cursor = tab.content.len();
            }
        }
        "home" => {
            undo.coalescing = false;
            if mods.shift && tab.selection_anchor.is_none() {
                tab.selection_anchor = Some(tab.cursor);
            } else if !mods.shift {
                tab.selection_anchor = None;
            }
            let before = &tab.content[..tab.cursor];
            tab.cursor = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
        }
        "end" => {
            undo.coalescing = false;
            if mods.shift && tab.selection_anchor.is_none() {
                tab.selection_anchor = Some(tab.cursor);
            } else if !mods.shift {
                tab.selection_anchor = None;
            }
            let after = &tab.content[tab.cursor..];
            tab.cursor += after.find('\n').unwrap_or(after.len());
        }
        _ => {
            if mods.control || mods.alt || mods.platform || mods.function {
                return;
            }
            let Some(kc) = keystroke.key_char.as_deref() else {
                return;
            };

            // Skip-over: typing a closer/quote that already sits under the
            // cursor just steps past it (pairs nicely with auto-close).
            if ctx.auto_close
                && tab.selection_anchor.is_none()
                && matches!(kc, ")" | "]" | "}" | "\"" | "'")
                && tab.content[tab.cursor..].starts_with(kc)
            {
                undo.coalescing = false;
                tab.cursor += kc.len();
                return;
            }

            if ctx.record_undo && !undo.coalescing {
                undo.push(snapshot(tab));
                undo.coalescing = true;
            }
            delete_selection(&mut tab.content, &mut tab.cursor, &mut tab.selection_anchor);

            let closer = match kc {
                "(" => Some(")"),
                "[" => Some("]"),
                "{" => Some("}"),
                "\"" => Some("\""),
                "'" => Some("'"),
                _ => None,
            };
            tab.content.insert_str(tab.cursor, kc);
            tab.cursor += kc.len();
            if ctx.auto_close {
                if let Some(c) = closer {
                    // Insert the partner; leave the cursor between the pair.
                    tab.content.insert_str(tab.cursor, c);
                }
            }
        }
    }
}
