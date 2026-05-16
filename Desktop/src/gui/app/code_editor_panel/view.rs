use openframe::{
    div, font, px, rgb, rgba, AnyElement, Context, InteractiveElement, IntoElement, KeyDownEvent,
    MouseButton, MouseDownEvent, MouseMoveEvent, ParentElement, StatefulInteractiveElement, Styled,
    Window,
};

use super::segments::{line_segments, SegKind};
use super::text::{char_cols, detect_language, expand_tabs, pos_to_byte_offset, word_bounds};
use crate::gui::app::ArcadiaRoot;
use crate::gui::theme;
use arcadia_core::modules::python_registry;

impl ArcadiaRoot {
    pub(super) fn code_editor_view(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> AnyElement {
        let p = theme::theme_palette(cx, is_dark);

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
            let line_idx = starts
                .partition_point(|&s| s <= cursor_clamped)
                .saturating_sub(1);
            let line_start = starts.get(line_idx).copied().unwrap_or(0);
            Some((line_idx, cursor_clamped - line_start))
        } else {
            None
        };

        // Selection range as (start, end) byte offsets, or None
        let sel_range: Option<(usize, usize)> = selection_anchor.map(|a| {
            if a <= cursor {
                (a, cursor)
            } else {
                (cursor, a)
            }
        });

        let line_count = self.code_editor_tabs[idx].cached_lines.len();
        let gutter_w = if line_count >= 1000 {
            px(56.)
        } else if line_count >= 100 {
            px(48.)
        } else {
            px(40.)
        };

        let editor_bg = if is_dark {
            rgb(0x1a1f29)
        } else {
            rgb(0xfafafa)
        };
        let gutter_bg = if is_dark {
            rgb(0x141820)
        } else {
            rgb(0xf0f0f2)
        };
        let gutter_fg = if is_dark {
            rgb(0x4a5568)
        } else {
            rgb(0xadb5bd)
        };
        let line_fg = p.content_title;
        let cursor_bg = if is_dark {
            rgb(0xe2e8f0)
        } else {
            rgb(0x1a202c)
        };
        let cursor_fg = if is_dark {
            rgb(0x1a1f29)
        } else {
            rgb(0xfafafa)
        };
        let sel_bg = if is_dark {
            rgb(0x2d4a7a)
        } else {
            rgb(0xbfdbfe)
        };
        let show_marks = self.code_editor_show_indentation_marks;
        let indent_guide_color = if is_dark {
            rgb(0x2d3748)
        } else {
            rgb(0xd1d5db)
        };

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
                        if s < e {
                            Some((s, e))
                        } else {
                            None
                        }
                    }
                });

                let display_line = expand_tabs(&line);
                let segs = line_segments(
                    &line,
                    line_start_byte,
                    cursor_in_line,
                    sel_in_line,
                    &hl_spans,
                );

                let decorations = cached_decorations.get(i).cloned().unwrap_or_default();

                let indent_levels = if show_marks {
                    display_line.chars().take_while(|&c| c == ' ').count() / 4
                } else {
                    0
                };

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
                // Decoration backgrounds: absolute rects on the char grid,
                // rendered behind text + indent guides. left()/w() are in grid
                // units (col * char_width) — same coordinate system as the text.
                let base_div = decorations.iter().fold(base_div, |d, r| {
                    let color = ((r.r as u32) << 24)
                        | ((r.g as u32) << 16)
                        | ((r.b as u32) << 8)
                        | r.a as u32;
                    d.child(
                        div()
                            .absolute()
                            .left(px(r.col_start as f32 * char_width))
                            .top(px(0.))
                            .bottom(px(0.))
                            .w(px(r.col_width as f32 * char_width))
                            .bg(rgba(color)),
                    )
                });
                // Indent guides: 1px absolute lines at each 4-col boundary.
                // left() is from content-box origin (after px_3 padding), so no 12px offset.
                let base_div = (0..indent_levels).fold(base_div, |d, level| {
                    let x = (level as f32) * 4.0 * char_width;
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
                // Each character renders in its own fixed-width cell
                // (char_width, wide chars = 2x). A per-char cell keeps every
                // column on the grid regardless of the font's real glyph
                // advances — "monospace" can resolve to a proportional fallback,
                // so a per-segment width would let caps overflow and narrow
                // glyphs underflow. Text, indent guides, decorations and click
                // hit-testing all share the same char_width coordinate system.
                let content_div =
                    base_div.children(segs.into_iter().flat_map(move |(text, kind, hl_color)| {
                        let text_color = match (kind, hl_color) {
                            (SegKind::Normal, Some(c)) => rgb(c),
                            _ => line_fg,
                        };
                        expand_tabs(&text)
                            .chars()
                            .collect::<Vec<char>>()
                            .into_iter()
                            .map(move |ch| {
                                let cell = div()
                                    .flex_shrink_0()
                                    .w(px(char_cols(ch) as f32 * char_width))
                                    .overflow_hidden()
                                    .text_sm()
                                    .font_family("monospace")
                                    .text_color(text_color)
                                    .child(ch.to_string());

                                match kind {
                                    SegKind::Normal => cell.into_any_element(),
                                    SegKind::Selected => cell.bg(sel_bg).into_any_element(),
                                    SegKind::Cursor => {
                                        cell.bg(cursor_bg).text_color(cursor_fg).into_any_element()
                                    }
                                }
                            })
                            .collect::<Vec<_>>()
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
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |_, _, window, _| {
                    fh_click.focus(window);
                }),
            )
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                if this.code_editor_tabs.is_empty() {
                    return;
                }
                let idx = this
                    .active_code_editor_tab
                    .min(this.code_editor_tabs.len().saturating_sub(1));
                let len_before = this.code_editor_tabs[idx].content.len();
                super::edit::apply_key(&mut this.code_editor_tabs[idx], &event.keystroke);
                if this.code_editor_tabs[idx].content.len() != len_before {
                    this.code_editor_tabs[idx].highlight_dirty = true;
                    this.save_editor_session();
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
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, event: &MouseDownEvent, window, cx| {
                                    let idx = this
                                        .active_code_editor_tab
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
                                        let (ws, we) = word_bounds(
                                            &this.code_editor_tabs[idx].content,
                                            byte_pos,
                                        );
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
                                }),
                            )
                            .on_mouse_move(cx.listener(
                                move |this, event: &MouseMoveEvent, _, cx| {
                                    if !this.code_editor_is_dragging || !event.dragging() {
                                        return;
                                    }
                                    let idx = this
                                        .active_code_editor_tab
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
                                },
                            ))
                            .on_mouse_up(
                                MouseButton::Left,
                                cx.listener(|this, _, _, _| {
                                    this.code_editor_is_dragging = false;
                                }),
                            )
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
