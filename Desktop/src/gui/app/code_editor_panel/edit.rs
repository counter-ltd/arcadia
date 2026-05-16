use super::text::{delete_selection, next_char_boundary, prev_char_boundary};
use crate::gui::app::CodeEditorTab;
use openframe::Keystroke;

/// Apply a keystroke to a tab's content/cursor/selection. The view layer owns
/// highlight invalidation and persistence.
pub fn apply_key(tab: &mut CodeEditorTab, keystroke: &Keystroke) {
    let key = keystroke.key.as_str();
    let mods = keystroke.modifiers;

    if (mods.control || mods.platform) && key == "a" {
        tab.selection_anchor = Some(0);
        tab.cursor = tab.content.len();
        return;
    }

    match key {
        "backspace" => {
            if !delete_selection(&mut tab.content, &mut tab.cursor, &mut tab.selection_anchor) {
                if tab.cursor > 0 {
                    let prev = prev_char_boundary(&tab.content, tab.cursor);
                    tab.content.drain(prev..tab.cursor);
                    tab.cursor = prev;
                }
            }
        }
        "delete" => {
            if !delete_selection(&mut tab.content, &mut tab.cursor, &mut tab.selection_anchor) {
                if tab.cursor < tab.content.len() {
                    let next = next_char_boundary(&tab.content, tab.cursor);
                    tab.content.drain(tab.cursor..next);
                }
            }
        }
        "enter" => {
            delete_selection(&mut tab.content, &mut tab.cursor, &mut tab.selection_anchor);
            tab.content.insert(tab.cursor, '\n');
            tab.cursor += 1;
        }
        "tab" => {
            delete_selection(&mut tab.content, &mut tab.cursor, &mut tab.selection_anchor);
            tab.content.insert_str(tab.cursor, "    ");
            tab.cursor += 4;
        }
        "left" => {
            if mods.shift {
                if tab.selection_anchor.is_none() {
                    tab.selection_anchor = Some(tab.cursor);
                }
                if tab.cursor > 0 {
                    tab.cursor = prev_char_boundary(&tab.content, tab.cursor);
                }
            } else {
                tab.selection_anchor = None;
                if tab.cursor > 0 {
                    tab.cursor = prev_char_boundary(&tab.content, tab.cursor);
                }
            }
        }
        "right" => {
            if mods.shift {
                if tab.selection_anchor.is_none() {
                    tab.selection_anchor = Some(tab.cursor);
                }
                if tab.cursor < tab.content.len() {
                    tab.cursor = next_char_boundary(&tab.content, tab.cursor);
                }
            } else {
                tab.selection_anchor = None;
                if tab.cursor < tab.content.len() {
                    tab.cursor = next_char_boundary(&tab.content, tab.cursor);
                }
            }
        }
        "up" => {
            if mods.shift && tab.selection_anchor.is_none() {
                tab.selection_anchor = Some(tab.cursor);
            } else if !mods.shift {
                tab.selection_anchor = None;
            }
            let before = &tab.content[..tab.cursor];
            let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
            let col = before[line_start..].chars().count();
            if line_start > 0 {
                let prev_nl_end = line_start - 1;
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
            if mods.shift && tab.selection_anchor.is_none() {
                tab.selection_anchor = Some(tab.cursor);
            } else if !mods.shift {
                tab.selection_anchor = None;
            }
            let before = &tab.content[..tab.cursor];
            let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
            let col = before[line_start..].chars().count();
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
            if mods.shift && tab.selection_anchor.is_none() {
                tab.selection_anchor = Some(tab.cursor);
            } else if !mods.shift {
                tab.selection_anchor = None;
            }
            let before = &tab.content[..tab.cursor];
            tab.cursor = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
        }
        "end" => {
            if mods.shift && tab.selection_anchor.is_none() {
                tab.selection_anchor = Some(tab.cursor);
            } else if !mods.shift {
                tab.selection_anchor = None;
            }
            let after = &tab.content[tab.cursor..];
            tab.cursor += after.find('\n').unwrap_or(after.len());
        }
        _ => {
            if !mods.control && !mods.alt && !mods.platform && !mods.function {
                if let Some(kc) = &keystroke.key_char {
                    delete_selection(&mut tab.content, &mut tab.cursor, &mut tab.selection_anchor);
                    tab.content.insert_str(tab.cursor, kc);
                    tab.cursor += kc.len();
                }
            }
        }
    }
}
