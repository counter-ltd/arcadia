use std::collections::HashMap;

use crate::gui::app::{ArcadiaRoot, CodeEditorTab};

/// (content, cursor, selection_anchor) at a point in time.
type EditSnapshot = (String, usize, Option<usize>);

const UNDO_CAP: usize = 200;

/// Per-tab undo/redo history. Kept on `ArcadiaRoot` keyed by tab id so the
/// `CodeEditorTab` struct (and its many construction sites) stay untouched.
#[derive(Default)]
pub struct EditorUndo {
    undo: Vec<EditSnapshot>,
    redo: Vec<EditSnapshot>,
    /// While true, a run of plain typing folds into the latest undo entry
    /// instead of pushing one snapshot per character.
    pub coalescing: bool,
}

impl EditorUndo {
    /// Record a pre-edit snapshot. Clears the redo branch.
    pub fn push(&mut self, snap: EditSnapshot) {
        self.undo.push(snap);
        if self.undo.len() > UNDO_CAP {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    pub fn step_back(&mut self, current: EditSnapshot) -> Option<EditSnapshot> {
        let prev = self.undo.pop()?;
        self.redo.push(current);
        Some(prev)
    }

    pub fn step_forward(&mut self, current: EditSnapshot) -> Option<EditSnapshot> {
        let next = self.redo.pop()?;
        self.undo.push(current);
        Some(next)
    }

    /// One-line previews of past states, newest first (newest = one undo away).
    pub fn undo_previews(&self) -> Vec<String> {
        self.undo.iter().rev().map(|s| snapshot_preview(&s.0)).collect()
    }

    /// One-line previews of future states, next-redo last (closest to current).
    pub fn redo_previews(&self) -> Vec<String> {
        self.redo.iter().map(|s| snapshot_preview(&s.0)).collect()
    }
}

/// First non-blank line of a snapshot, trimmed and truncated for the history list.
fn snapshot_preview(content: &str) -> String {
    let line = content
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("");
    if line.is_empty() {
        return "(empty)".to_string();
    }
    let truncated: String = line.chars().take(40).collect();
    if truncated.chars().count() < line.chars().count() {
        format!("{truncated}…")
    } else {
        truncated
    }
}

// ── line geometry ───────────────────────────────────────────────────────────

fn line_start(s: &str, pos: usize) -> usize {
    s[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0)
}

fn line_end(s: &str, pos: usize) -> usize {
    s[pos..].find('\n').map(|i| pos + i).unwrap_or(s.len())
}

/// Ordered (start, end) byte range of the current selection, or (cursor, cursor).
fn sel_span(tab: &CodeEditorTab) -> (usize, usize) {
    match tab.selection_anchor {
        Some(a) if a <= tab.cursor => (a, tab.cursor),
        Some(a) => (tab.cursor, a),
        None => (tab.cursor, tab.cursor),
    }
}

/// First-byte offsets of every line touched by the current selection/cursor.
fn touched_line_starts(tab: &CodeEditorTab) -> Vec<usize> {
    let (ss, se) = sel_span(tab);
    let first = line_start(&tab.content, ss);
    let last = line_start(&tab.content, se);
    let mut starts = Vec::new();
    let mut ls = first;
    loop {
        starts.push(ls);
        if ls >= last {
            break;
        }
        match tab.content[ls..].find('\n') {
            Some(i) => ls = ls + i + 1,
            None => break,
        }
    }
    starts
}

fn comment_token(lang: Option<&str>) -> &'static str {
    match lang {
        Some("python" | "ruby" | "yaml" | "shell" | "sh" | "toml") => "#",
        _ => "//",
    }
}

// ── line operations ─────────────────────────────────────────────────────────

fn select_line(tab: &mut CodeEditorTab) {
    let s = line_start(&tab.content, tab.cursor);
    let e = line_end(&tab.content, tab.cursor);
    tab.selection_anchor = Some(s);
    tab.cursor = e;
}

fn duplicate_line(tab: &mut CodeEditorTab) -> bool {
    let s = line_start(&tab.content, tab.cursor);
    let e = line_end(&tab.content, tab.cursor);
    let text = tab.content[s..e].to_string();
    tab.content.insert_str(e, &format!("\n{text}"));
    tab.cursor += (e - s) + 1;
    tab.selection_anchor = None;
    true
}

fn delete_line(tab: &mut CodeEditorTab) -> bool {
    let s = line_start(&tab.content, tab.cursor);
    let e = line_end(&tab.content, tab.cursor);
    let len = tab.content.len();
    let new_cursor = if e < len {
        tab.content.drain(s..=e); // line + trailing newline
        s
    } else if s > 0 {
        tab.content.drain(s - 1..e); // last line + leading newline
        s - 1
    } else {
        tab.content.drain(s..e); // only line
        0
    };
    tab.cursor = new_cursor.min(tab.content.len());
    tab.selection_anchor = None;
    true
}

fn move_line(tab: &mut CodeEditorTab, up: bool) -> bool {
    let s = line_start(&tab.content, tab.cursor);
    let e = line_end(&tab.content, tab.cursor);
    let col = tab.cursor - s;
    if up {
        if s == 0 {
            return false;
        }
        let prev_nl = s - 1;
        let ps = line_start(&tab.content, prev_nl);
        let cur = tab.content[s..e].to_string();
        let prev = tab.content[ps..prev_nl].to_string();
        tab.content.replace_range(ps..e, &format!("{cur}\n{prev}"));
        tab.cursor = ps + col;
    } else {
        if e == tab.content.len() {
            return false;
        }
        let next_s = e + 1;
        let ne = line_end(&tab.content, next_s);
        let cur = tab.content[s..e].to_string();
        let next = tab.content[next_s..ne].to_string();
        tab.content.replace_range(s..ne, &format!("{next}\n{cur}"));
        tab.cursor = s + next.len() + 1 + col;
    }
    tab.selection_anchor = None;
    true
}

fn indent_lines(tab: &mut CodeEditorTab, indent: bool) -> bool {
    let starts = touched_line_starts(tab);
    let mut cur = tab.cursor as isize;
    let mut anc = tab.selection_anchor.map(|a| a as isize);
    let mut changed = false;
    for &ls in starts.iter().rev() {
        let d: isize = if indent {
            tab.content.insert_str(ls, "    ");
            4
        } else {
            let rest = &tab.content[ls..];
            let rm = if rest.starts_with('\t') {
                1
            } else {
                rest.chars().take_while(|&c| c == ' ').count().min(4)
            };
            if rm == 0 {
                0
            } else {
                tab.content.drain(ls..ls + rm);
                -(rm as isize)
            }
        };
        if d != 0 {
            changed = true;
            let lsi = ls as isize;
            if cur >= lsi {
                cur = (cur + d).max(lsi);
            }
            if let Some(a) = anc {
                if a >= lsi {
                    anc = Some((a + d).max(lsi));
                }
            }
        }
    }
    tab.cursor = (cur.max(0) as usize).min(tab.content.len());
    tab.selection_anchor = anc.map(|a| (a.max(0) as usize).min(tab.content.len()));
    changed
}

fn toggle_comment(tab: &mut CodeEditorTab, lang: Option<&str>) -> bool {
    let token = comment_token(lang);
    let starts = touched_line_starts(tab);

    // Comment unless every non-blank touched line is already commented.
    let all_commented = starts.iter().all(|&ls| {
        let line = &tab.content[ls..line_end(&tab.content, ls)];
        let t = line.trim_start();
        t.is_empty() || t.starts_with(token)
    });

    let mut cur = tab.cursor as isize;
    let mut anc = tab.selection_anchor.map(|a| a as isize);
    let mut changed = false;
    for &ls in starts.iter().rev() {
        let le = line_end(&tab.content, ls);
        let ws = tab.content[ls..le]
            .bytes()
            .take_while(|&b| b == b' ' || b == b'\t')
            .count();
        let at = ls + ws;
        let d: isize = if all_commented {
            let rest = &tab.content[at..le];
            if let Some(after) = rest.strip_prefix(token) {
                let extra = usize::from(after.starts_with(' '));
                let rm = token.len() + extra;
                tab.content.drain(at..at + rm);
                -(rm as isize)
            } else {
                0
            }
        } else if at == le {
            0 // skip blank lines
        } else {
            let ins = format!("{token} ");
            let n = ins.len();
            tab.content.insert_str(at, &ins);
            n as isize
        };
        if d != 0 {
            changed = true;
            let ati = at as isize;
            if cur >= ati {
                cur = (cur + d).max(ati);
            }
            if let Some(a) = anc {
                if a >= ati {
                    anc = Some((a + d).max(ati));
                }
            }
        }
    }
    tab.cursor = (cur.max(0) as usize).min(tab.content.len());
    tab.selection_anchor = anc.map(|a| (a.max(0) as usize).min(tab.content.len()));
    changed
}

// ── dispatch ────────────────────────────────────────────────────────────────

impl ArcadiaRoot {
    /// Run an editor command fired by the shortcut system (`editor.*`).
    pub(crate) fn editor_run_command(&mut self, control_id: &str) {
        if self.code_editor_tabs.is_empty() {
            return;
        }
        let idx = self
            .active_code_editor_tab
            .min(self.code_editor_tabs.len() - 1);
        let tab_id = self.code_editor_tabs[idx].id;

        match control_id {
            "editor.undo" | "editor.redo" => {
                if !self.code_editor_undo_enabled {
                    return;
                }
                let undo = self.code_editor_undo.entry(tab_id).or_default();
                let tab = &self.code_editor_tabs[idx];
                let current = (tab.content.clone(), tab.cursor, tab.selection_anchor);
                let restored = if control_id == "editor.undo" {
                    undo.step_back(current)
                } else {
                    undo.step_forward(current)
                };
                undo.coalescing = false;
                if let Some((content, cursor, anchor)) = restored {
                    let tab = &mut self.code_editor_tabs[idx];
                    tab.content = content;
                    tab.cursor = cursor.min(tab.content.len());
                    tab.selection_anchor = anchor;
                    tab.highlight_dirty = true;
                    self.save_editor_session();
                }
            }
            "editor.save" => {
                let tab = &mut self.code_editor_tabs[idx];
                if let Some(path) = tab.file_path.clone() {
                    if std::fs::write(&path, &tab.content).is_ok() {
                        tab.saved_content = tab.content.clone();
                    }
                }
                self.save_editor_session();
            }
            _ => {
                if !self.code_editor_line_commands {
                    return;
                }
                let lang = self.code_editor_tabs[idx].language.clone();
                let before = {
                    let t = &self.code_editor_tabs[idx];
                    (t.content.clone(), t.cursor, t.selection_anchor)
                };
                let tab = &mut self.code_editor_tabs[idx];
                let changed = match control_id {
                    "editor.select_line" => {
                        select_line(tab);
                        false
                    }
                    "editor.duplicate_line" => duplicate_line(tab),
                    "editor.delete_line" => delete_line(tab),
                    "editor.move_line_up" => move_line(tab, true),
                    "editor.move_line_down" => move_line(tab, false),
                    "editor.indent" => indent_lines(tab, true),
                    "editor.outdent" => indent_lines(tab, false),
                    "editor.toggle_comment" => toggle_comment(tab, lang.as_deref()),
                    _ => false,
                };
                if changed {
                    tab.highlight_dirty = true;
                    if self.code_editor_undo_enabled {
                        let u = self.code_editor_undo.entry(tab_id).or_default();
                        u.push(before);
                        u.coalescing = false;
                    }
                    self.save_editor_session();
                }
            }
        }
    }

    /// Apply `steps` undo (or redo when `redo` is true) operations at once —
    /// drives the undo-history popup so a click can jump multiple steps.
    pub(crate) fn editor_undo_jump(&mut self, redo: bool, steps: usize) {
        if self.code_editor_tabs.is_empty() || steps == 0 || !self.code_editor_undo_enabled {
            return;
        }
        let idx = self
            .active_code_editor_tab
            .min(self.code_editor_tabs.len() - 1);
        let tab_id = self.code_editor_tabs[idx].id;
        let undo = self.code_editor_undo.entry(tab_id).or_default();
        let tab = &self.code_editor_tabs[idx];
        let mut current = (tab.content.clone(), tab.cursor, tab.selection_anchor);
        let mut restored = None;
        for _ in 0..steps {
            let next = if redo {
                undo.step_forward(current.clone())
            } else {
                undo.step_back(current.clone())
            };
            match next {
                Some(s) => {
                    current = s.clone();
                    restored = Some(s);
                }
                None => break,
            }
        }
        undo.coalescing = false;
        if let Some((content, cursor, anchor)) = restored {
            let tab = &mut self.code_editor_tabs[idx];
            tab.content = content;
            tab.cursor = cursor.min(tab.content.len());
            tab.selection_anchor = anchor;
            tab.highlight_dirty = true;
            self.save_editor_session();
        }
    }
}

/// Convenience alias so `mod.rs` can name the field type without a deep path.
pub type EditorUndoMap = HashMap<usize, EditorUndo>;
