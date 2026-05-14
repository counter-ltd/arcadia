use openframe::{
    AnyElement, Bounds, Context, FontWeight, InteractiveElement, IntoElement, KeyDownEvent,
    MouseButton, MouseDownEvent, MouseMoveEvent, ParentElement, Pixels, StatefulInteractiveElement,
    Styled, Window, div, font, px, rgb, rgba,
};

use arcadia_core::config::ConfigFile;
use arcadia_core::config::workspace::WorkspacesConfig;
use arcadia_core::modules::python_registry::{self, HighlightSpan};
use crate::gui::app::{ArcadiaRoot, CodeEditorTab};
use crate::gui::theme;

/// Expand `\t` to 4 spaces for visual rendering.
fn expand_tabs(s: &str) -> String {
    if !s.contains('\t') { return s.to_string(); }
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        if c == '\t' { out.push_str("    "); } else { out.push(c); }
    }
    out
}

/// Map a visual column (with tabs = 4 spaces) back to a byte offset in the original line.
fn vis_col_to_orig_byte(line: &str, vis_col: usize) -> usize {
    let mut col = 0;
    for (i, c) in line.char_indices() {
        if col >= vis_col { return i; }
        col += if c == '\t' { 4 } else { 1 };
    }
    line.len()
}

fn prev_char_boundary(s: &str, pos: usize) -> usize {
    let mut p = pos;
    while p > 0 {
        p -= 1;
        if s.is_char_boundary(p) {
            break;
        }
    }
    p
}

fn next_char_boundary(s: &str, pos: usize) -> usize {
    if pos >= s.len() {
        return pos;
    }
    let mut p = pos + 1;
    while p < s.len() && !s.is_char_boundary(p) {
        p += 1;
    }
    p
}

fn word_bounds(content: &str, pos: usize) -> (usize, usize) {
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

fn delete_selection(content: &mut String, cursor: &mut usize, anchor: &mut Option<usize>) -> bool {
    if let Some(a) = *anchor {
        let (sel_start, sel_end) = if a <= *cursor { (a, *cursor) } else { (*cursor, a) };
        content.drain(sel_start..sel_end);
        *cursor = sel_start;
        *anchor = None;
        true
    } else {
        false
    }
}

fn pos_to_byte_offset(
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
    let content_x = line_x + gutter_w + px(12.0);
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

fn parse_highlight_color(token: &str) -> Option<u32> {
    let hex = token.trim_start_matches('#');
    if hex.len() == 6 {
        u32::from_str_radix(hex, 16).ok()
    } else {
        None
    }
}

#[derive(Clone, Copy, PartialEq)]
enum SegKind {
    Normal,
    Selected,
    Cursor,
}

fn line_segments(
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
            if ls < le { Some((ls, le, color)) } else { None }
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
        let is_cursor_seg = !eol_cursor
            && cursor_byte.map_or(false, |c| {
                s >= c && e <= cursor_end.unwrap_or(c)
            });
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

impl ArcadiaRoot {
    pub fn code_editor_panel(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> AnyElement {
        let p = theme::theme_palette(cx, is_dark);

        if self.code_editor_tabs.is_empty() || self.code_editor_show_dashboard {
            let pal = theme::nav_accent_palette("emerald", is_dark);
            let r = p.radius_md.min(12.0);

            let workspace_cards: Vec<AnyElement> =
                WorkspacesConfig::load_or_create()
                    .map(|cfg| cfg.workspaces)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|ws| {
                        let ws_path_clone = ws.path.clone();
                        div()
                            .flex_1()
                            .min_w(px(220.))
                            .cursor_pointer()
                            .p_4()
                            .rounded(px(r))
                            .bg(p.panel_bg)
                            .border_1()
                            .border_color(p.panel_border)
                            .hover(move |s| s.bg(p.row_bg).border_color(pal.row_hover))
                            .flex()
                            .flex_col()
                            .gap_3()
                            .child(
                                div()
                                    .flex()
                                    .gap_3()
                                    .items_center()
                                    .child(
                                        theme::render_icon("folder-open")
                                            .size_6()
                                            .text_color(pal.icon_active),
                                    )
                                    .child(
                                        div()
                                            .text_base()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(p.content_title)
                                            .flex_1()
                                            .child(ws.label.clone()),
                                    ),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(p.content_meta)
                                    .max_w(px(248.))
                                    .child(ws.path.clone()),
                            )
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, window, cx| {
                                    let id = this.code_editor_next_id;
                                    this.code_editor_tabs.push(CodeEditorTab {
                                        id,
                                        title: format!("untitled-{id}"),
                                        content: String::new(),
                                        cursor: 0,
                                        selection_anchor: None,
                                        language: None,
                                        hl_spans: vec![],
                                        decorations: vec![],
                                        highlight_dirty: true,
                                        workspace_path: Some(ws_path_clone.clone()),
                                        file_path: None,
                                        saved_content: String::new(),
                                        cached_lines: vec![],
                                        cached_line_byte_starts: vec![],
                                    });
                                    this.active_code_editor_tab =
                                        this.code_editor_tabs.len() - 1;
                                    this.code_editor_next_id += 1;
                                    this.active_page_id = "editor.main".to_string();
                                    this.code_editor_show_dashboard = false;
                                    this.code_editor_focus.focus(window);
                                    cx.notify();
                                }),
                            )
                            .into_any_element()
                    })
                    .collect();

            let editor_bg_preview = if is_dark { rgb(0x141820) } else { rgb(0xf0f2f5) };
            let editor_cards: Vec<AnyElement> = self
                .code_editor_tabs
                .iter()
                .enumerate()
                .map(|(tab_idx, tab)| {
                    let title = tab.title.clone();
                    let subtitle = tab
                        .file_path
                        .as_deref()
                        .or(tab.workspace_path.as_deref())
                        .unwrap_or("unsaved")
                        .to_string();
                    let is_dirty = tab.content != tab.saved_content;
                    let preview_lines: Vec<String> = tab
                        .content
                        .lines()
                        .take(9)
                        .map(|l| {
                            let s: String = l.chars().take(48).collect();
                            s
                        })
                        .collect();
                    let fh_card = self.code_editor_focus.clone();
                    div()
                        .flex_1()
                        .min_w(px(220.))
                        .cursor_pointer()
                        .rounded(px(r))
                        .bg(p.panel_bg)
                        .border_1()
                        .border_color(p.panel_border)
                        .hover(move |s| s.bg(p.row_bg).border_color(pal.row_hover))
                        .flex()
                        .flex_col()
                        .overflow_hidden()
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, window, cx| {
                                this.active_code_editor_tab = tab_idx;
                                this.code_editor_show_dashboard = false;
                                this.active_page_id = "editor.main".to_string();
                                fh_card.focus(window);
                                cx.notify();
                            }),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_h(px(100.))
                                .bg(editor_bg_preview)
                                .p_2()
                                .overflow_hidden()
                                .flex()
                                .flex_col()
                                .gap_0()
                                .children(preview_lines.into_iter().map(|line| {
                                    div()
                                        .text_xs()
                                        .font_family("monospace")
                                        .text_color(p.content_meta)
                                        .flex_shrink_0()
                                        .child(if line.is_empty() { " ".to_string() } else { line })
                                        .into_any_element()
                                })),
                        )
                        .child(
                            div()
                                .p_3()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .border_t_1()
                                .border_color(p.panel_border)
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .gap_1()
                                        .items_center()
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(p.content_title)
                                                .flex_1()
                                                .child(title),
                                        )
                                        .child(if is_dirty {
                                            div()
                                                .w(px(6.))
                                                .h(px(6.))
                                                .rounded_full()
                                                .bg(pal.icon_active)
                                                .into_any_element()
                                        } else {
                                            div().into_any_element()
                                        }),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(p.content_meta)
                                        .child(subtitle),
                                ),
                        )
                        .into_any_element()
                })
                .collect();

            let editors_body: AnyElement = if editor_cards.is_empty() {
                div()
                    .text_sm()
                    .text_color(p.content_meta)
                    .child("No open editors.")
                    .into_any_element()
            } else {
                let phantoms: Vec<_> = (0..5)
                    .map(|_| div().flex_1().min_w(px(220.)).into_any_element())
                    .collect();
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap_4()
                    .children(editor_cards)
                    .children(phantoms)
                    .into_any_element()
            };

            let workspaces_body: AnyElement = if workspace_cards.is_empty() {
                div()
                    .text_sm()
                    .text_color(p.content_meta)
                    .child("No workspaces registered. Add one in Settings → Workspaces.")
                    .into_any_element()
            } else {
                let phantoms: Vec<_> = (0..5)
                    .map(|_| div().flex_1().min_w(px(220.)).into_any_element())
                    .collect();
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap_4()
                    .children(workspace_cards)
                    .children(phantoms)
                    .into_any_element()
            };

            return div()
                .id("editor-empty-state")
                .w_full()
                .h_full()
                .overflow_y_scroll()
                .bg(if is_dark { rgb(0x1a1f29) } else { rgb(0xfafafa) })
                .p_8()
                .flex()
                .flex_col()
                .gap_8()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_4()
                        .child(
                            div()
                                .text_xl()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(p.content_title)
                                .child("Workspaces"),
                        )
                        .child(workspaces_body),
                )
                .child(div().w_full().h(px(1.)).bg(p.panel_border))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_4()
                        .child(
                            div()
                                .text_xl()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(p.content_title)
                                .child("Open Editors"),
                        )
                        .child(editors_body),
                )
                .into_any_element();
        }

        let idx = self
            .active_code_editor_tab
            .min(self.code_editor_tabs.len().saturating_sub(1));
        let cursor = self.code_editor_tabs[idx].cursor;
        let selection_anchor = self.code_editor_tabs[idx].selection_anchor;
        let tab_title = self.code_editor_tabs[idx].title.clone();
        let language = self.code_editor_tabs[idx]
            .language
            .clone()
            .or_else(|| detect_language(&tab_title));
        let focused = self.code_editor_focus.is_focused(window);
        let blink = self.text_caret_blink_visible;

        // Rebuild highlight + decoration caches (and line caches) only when content changed.
        if self.code_editor_tabs[idx].highlight_dirty {
            let content = &self.code_editor_tabs[idx].content;
            let mut new_lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
            if new_lines.is_empty() || content.ends_with('\n') || content.is_empty() {
                new_lines.push(String::new());
            }
            let mut new_byte_starts: Vec<usize> = Vec::with_capacity(new_lines.len());
            {
                let mut pos = 0usize;
                for line in &new_lines {
                    new_byte_starts.push(pos);
                    pos += line.len() + 1;
                }
            }
            let hl = language
                .as_deref()
                .map(|lang| python_registry::call_highlight_provider(lang, content))
                .unwrap_or_default();
            let deco: Vec<Vec<_>> = new_lines
                .iter()
                .enumerate()
                .map(|(i, line)| {
                    let expanded = expand_tabs(line);
                    python_registry::call_decoration_providers(&expanded, i)
                })
                .collect();
            let tab = &mut self.code_editor_tabs[idx];
            tab.hl_spans = hl;
            tab.decorations = deco;
            tab.highlight_dirty = false;
            tab.cached_lines = new_lines;
            tab.cached_line_byte_starts = new_byte_starts;
        }

        // (line_idx, col_byte_in_line) for cursor — O(log N) via binary search.
        let cursor_line_col: Option<(usize, usize)> = if focused && blink {
            let starts = &self.code_editor_tabs[idx].cached_line_byte_starts;
            let cursor_clamped = cursor.min(self.code_editor_tabs[idx].content.len());
            let line_idx = starts.partition_point(|&s| s <= cursor_clamped).saturating_sub(1);
            let line_start = starts.get(line_idx).copied().unwrap_or(0);
            Some((line_idx, cursor_clamped - line_start))
        } else {
            None
        };

        // Selection range as (start, end) byte offsets, or None
        let sel_range: Option<(usize, usize)> = selection_anchor.map(|a| {
            if a <= cursor { (a, cursor) } else { (cursor, a) }
        });

        let line_count = self.code_editor_tabs[idx].cached_lines.len();
        let gutter_w = if line_count >= 1000 {
            px(56.)
        } else if line_count >= 100 {
            px(48.)
        } else {
            px(40.)
        };

        let editor_bg = if is_dark { rgb(0x1a1f29) } else { rgb(0xfafafa) };
        let gutter_bg = if is_dark { rgb(0x141820) } else { rgb(0xf0f0f2) };
        let gutter_fg = if is_dark { rgb(0x4a5568) } else { rgb(0xadb5bd) };
        let line_fg = p.content_title;
        let cursor_bg = if is_dark { rgb(0xe2e8f0) } else { rgb(0x1a202c) };
        let cursor_fg = if is_dark { rgb(0x1a1f29) } else { rgb(0xfafafa) };
        let sel_bg = if is_dark { rgb(0x2d4a7a) } else { rgb(0xbfdbfe) };
        let show_marks = self.code_editor_show_indentation_marks;
        let indent_guide_color = if is_dark { rgb(0x2d3748) } else { rgb(0xd1d5db) };

        let char_width = self.code_editor_char_width_override.unwrap_or_else(|| {
            let font_size = window.rem_size() * 0.875;
            let ts = window.text_system();
            let fid = ts.resolve_font(&font("monospace"));
            ts.ch_advance(fid, font_size).map(f32::from).unwrap_or(8.4)
        });
        let fh_click = self.code_editor_focus.clone();
        let fh_click2 = self.code_editor_focus.clone();
        let fh = self.code_editor_focus.clone();
        let bounds_cell = self.code_editor_line_bounds.clone();

        // Borrow cached data by reference — no per-frame heap allocation.
        let line_byte_starts = &self.code_editor_tabs[idx].cached_line_byte_starts;
        let hl_spans = &self.code_editor_tabs[idx].hl_spans;
        let cached_decorations = &self.code_editor_tabs[idx].decorations;
        let lines = &self.code_editor_tabs[idx].cached_lines;

        let line_els: Vec<AnyElement> = lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let line = line.as_str();
                let line_start_byte = *line_byte_starts.get(i).unwrap_or(&0);
                let line_end_byte = line_start_byte + line.len();

                // Cursor byte within this line, if cursor is on this line
                let cursor_in_line: Option<usize> = cursor_line_col.and_then(|(cl, col)| {
                    if cl == i {
                        Some(col.min(line.len()))
                    } else {
                        None
                    }
                });

                // Selection range clamped to this line
                let sel_in_line: Option<(usize, usize)> = sel_range.and_then(|(ss, se)| {
                    if se <= line_start_byte || ss > line_end_byte {
                        None
                    } else {
                        let s = ss.saturating_sub(line_start_byte).min(line.len());
                        let e = se.saturating_sub(line_start_byte).min(line.len());
                        if s < e { Some((s, e)) } else { None }
                    }
                });

                let display_line = expand_tabs(&line);
                let segs = line_segments(&line, line_start_byte, cursor_in_line, sel_in_line, &hl_spans);

                let decorations = cached_decorations.get(i).cloned().unwrap_or_default();

                let indent_levels = if show_marks {
                    display_line.chars().take_while(|&c| c == ' ').count() / 4
                } else {
                    0
                };

                // Content area left padding (px_3 = 12px); guides position at each 4-space column.
                let base_div = div()
                    .flex_1()
                    .px_3()
                    .py_0p5()
                    .text_sm()
                    .font_family("monospace")
                    .text_color(line_fg)
                    .relative()
                    .flex()
                    .flex_row()
                    .items_start();
                // Indent guides (thin vertical lines).
                let base_div = (0..indent_levels).fold(base_div, |d, level| {
                    let x = 12.0_f32 + (level as f32) * 4.0 * char_width;
                    d.child(
                        div()
                            .absolute()
                            .left(px(x))
                            .top(px(0.))
                            .bottom(px(0.))
                            .w(px(1.))
                            .bg(indent_guide_color),
                    )
                });
                // Rainbow-indent and other decoration boxes from extension providers.
                let base_div = decorations.into_iter().fold(base_div, |d, rect| {
                    let x = 12.0_f32 + rect.col_start as f32 * char_width;
                    let w = rect.col_width as f32 * char_width;
                    let color_u32 = ((rect.r as u32) << 24)
                        | ((rect.g as u32) << 16)
                        | ((rect.b as u32) << 8)
                        | rect.a as u32;
                    d.child(
                        div()
                            .absolute()
                            .left(px(x))
                            .top(px(0.))
                            .bottom(px(0.))
                            .w(px(w))
                            .bg(rgba(color_u32)),
                    )
                });
                let content_div = base_div.children(segs.into_iter().map(|(text, kind, hl_color)| {
                    let text_color = match (kind, hl_color) {
                        (SegKind::Normal, Some(c)) => rgb(c),
                        _ => line_fg,
                    };
                    let seg = div()
                        .text_sm()
                        .font_family("monospace")
                        .text_color(text_color)
                        .child(expand_tabs(&text));
                    match kind {
                        SegKind::Normal => seg.into_any_element(),
                        SegKind::Selected => seg.bg(sel_bg).into_any_element(),
                        SegKind::Cursor => seg
                            .bg(cursor_bg)
                            .text_color(cursor_fg)
                            .into_any_element(),
                    }
                }));

                div()
                    .flex()
                    .flex_row()
                    .flex_shrink_0()
                    .child(
                        div()
                            .w(gutter_w)
                            .flex_shrink_0()
                            .px_2()
                            .py_0p5()
                            .text_right()
                            .text_xs()
                            .font_family("monospace")
                            .text_color(gutter_fg)
                            .bg(gutter_bg)
                            .child((i + 1).to_string()),
                    )
                    .child(content_div)
                    .into_any_element()
            })
            .collect();

        div()
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .bg(editor_bg)
            .track_focus(&fh)
            .on_mouse_down(MouseButton::Left, cx.listener(move |_, _, window, _| {
                fh_click.focus(window);
            }))
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                if this.code_editor_tabs.is_empty() {
                    return;
                }
                let idx = this
                    .active_code_editor_tab
                    .min(this.code_editor_tabs.len().saturating_sub(1));
                let key = event.keystroke.key.as_str();
                let mods = event.keystroke.modifiers;
                let content_len_before = this.code_editor_tabs[idx].content.len();
                let tab = &mut this.code_editor_tabs[idx];

                // Ctrl/Cmd+A — select all
                if (mods.control || mods.platform) && key == "a" {
                    tab.selection_anchor = Some(0);
                    tab.cursor = tab.content.len();
                    cx.notify();
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
                            let prev_line_start =
                                prev_before.rfind('\n').map(|i| i + 1).unwrap_or(0);
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
                            let next_line = &next_content
                                [..next_content.find('\n').unwrap_or(next_content.len())];
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
                            if let Some(kc) = &event.keystroke.key_char {
                                delete_selection(
                                    &mut tab.content,
                                    &mut tab.cursor,
                                    &mut tab.selection_anchor,
                                );
                                tab.content.insert_str(tab.cursor, kc);
                                tab.cursor += kc.len();
                            }
                        }
                    }
                }
                if this.code_editor_tabs[idx].content.len() != content_len_before {
                    this.code_editor_tabs[idx].highlight_dirty = true;
                }
                cx.notify();
            }))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .id("code-editor-content")
                    .overflow_y_scroll()
                    .child(
                div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .py_2()
                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, event: &MouseDownEvent, window, cx| {
                        let idx = this.active_code_editor_tab
                            .min(this.code_editor_tabs.len().saturating_sub(1));
                        let bounds = this.code_editor_line_bounds.borrow();
                        let byte_pos = pos_to_byte_offset(
                            &this.code_editor_tabs[idx].content,
                            &bounds,
                            event.position,
                            gutter_w,
                            char_width,
                        );
                        drop(bounds);

                        if event.click_count == 2 {
                            let (ws, we) = word_bounds(&this.code_editor_tabs[idx].content, byte_pos);
                            this.code_editor_tabs[idx].selection_anchor = Some(ws);
                            this.code_editor_tabs[idx].cursor = we;
                        } else if event.modifiers.shift {
                            if this.code_editor_tabs[idx].selection_anchor.is_none() {
                                this.code_editor_tabs[idx].selection_anchor =
                                    Some(this.code_editor_tabs[idx].cursor);
                            }
                            this.code_editor_tabs[idx].cursor = byte_pos;
                        } else {
                            this.code_editor_tabs[idx].cursor = byte_pos;
                            this.code_editor_tabs[idx].selection_anchor = None;
                            this.code_editor_is_dragging = true;
                        }
                        fh_click2.focus(window);
                        cx.notify();
                    }))
                    .on_mouse_move(cx.listener(move |this, event: &MouseMoveEvent, _, cx| {
                        if !this.code_editor_is_dragging || !event.dragging() {
                            return;
                        }
                        let idx = this.active_code_editor_tab
                            .min(this.code_editor_tabs.len().saturating_sub(1));
                        if this.code_editor_tabs[idx].selection_anchor.is_none() {
                            this.code_editor_tabs[idx].selection_anchor =
                                Some(this.code_editor_tabs[idx].cursor);
                        }
                        let bounds = this.code_editor_line_bounds.borrow();
                        let byte_pos = pos_to_byte_offset(
                            &this.code_editor_tabs[idx].content,
                            &bounds,
                            event.position,
                            gutter_w,
                            char_width,
                        );
                        drop(bounds);
                        this.code_editor_tabs[idx].cursor = byte_pos;
                        cx.notify();
                    }))
                    .on_mouse_up(MouseButton::Left, cx.listener(|this, _, _, _| {
                        this.code_editor_is_dragging = false;
                    }))
                    .on_children_prepainted({
                        let bounds_cell = bounds_cell.clone();
                        move |child_bounds, _window, _cx| {
                            *bounds_cell.borrow_mut() = child_bounds;
                        }
                    })
                    .children(line_els),
                    ),
            )
            .into_any_element()
    }
}
