//! Canvas inspector bar — edits the selected block's source line and applies
//! structural operations (delete, reorder). Every operation re-parses the
//! buffer at action time, so it never relies on a stale block span.

use openframe::prelude::FluentBuilder as _;
use openframe::{
    div, px, AnyElement, Context, FontWeight, InteractiveElement, IntoElement, KeyDownEvent,
    MouseButton, ParentElement, Styled, Window,
};

use crate::gui::app::text_input_caret::text_with_trailing_caret;
use crate::gui::app::ArcadiaRoot;
use crate::gui::assets::MONO_FONT_FAMILY;
use crate::gui::theme;
use arcadia_core::modules::visual_editor::{codegen, parse};

fn prev_boundary(s: &str, mut i: usize) -> usize {
    while i > 0 {
        i -= 1;
        if s.is_char_boundary(i) {
            break;
        }
    }
    i
}

fn next_boundary(s: &str, mut i: usize) -> usize {
    let len = s.len();
    while i < len {
        i += 1;
        if s.is_char_boundary(i) {
            break;
        }
    }
    i
}

impl ArcadiaRoot {
    fn visual_active_idx(&self) -> Option<usize> {
        if self.visual_editor.tabs.is_empty() {
            None
        } else {
            Some(
                self.visual_editor.active_tab
                    .min(self.visual_editor.tabs.len() - 1),
            )
        }
    }

    /// Commit the inline draft — replace the selected block's source line.
    pub(super) fn visual_editor_commit_edit(&mut self) {
        let Some(path) = self.visual_editor.selected.clone() else {
            return;
        };
        let Some(idx) = self.visual_active_idx() else {
            return;
        };
        let content = self.visual_editor.tabs[idx].content.clone();
        let tree = parse::parse(&content);
        let Some(block) = tree.block_at(&path) else {
            return;
        };
        let start = block.span.start.min(content.len());
        let end = codegen::line_end(&content, start);
        let draft = self.visual_editor.edit_draft.clone();
        let updated = codegen::replace_region(&content, start, end, &draft);
        self.visual_editor.tabs[idx].content = updated;
        self.save_visual_editor_session();
    }

    /// Delete the selected block, removing its whole line(s).
    pub(super) fn visual_editor_delete_selected(&mut self) {
        let Some(path) = self.visual_editor.selected.clone() else {
            return;
        };
        let Some(idx) = self.visual_active_idx() else {
            return;
        };
        let content = self.visual_editor.tabs[idx].content.clone();
        let tree = parse::parse(&content);
        let Some(block) = tree.block_at(&path) else {
            return;
        };
        let updated = codegen::delete_block(&content, block.span.start, block.span.end);
        self.visual_editor.tabs[idx].content = updated;
        self.visual_editor.selected = None;
        self.save_visual_editor_session();
    }

    /// Move the selected block up or down among its siblings.
    pub(super) fn visual_editor_move_selected(&mut self, down: bool) {
        let Some(path) = self.visual_editor.selected.clone() else {
            return;
        };
        let Some(idx) = self.visual_active_idx() else {
            return;
        };
        let content = self.visual_editor.tabs[idx].content.clone();
        let tree = parse::parse(&content);
        let Some(siblings) = tree.siblings_at(&path) else {
            return;
        };
        let Some(&pos) = path.last() else {
            return;
        };
        let target = if down { pos + 1 } else { pos.wrapping_sub(1) };
        if down && target >= siblings.len() {
            return;
        }
        if !down && pos == 0 {
            return;
        }
        let (earlier, later) = if down {
            (&siblings[pos], &siblings[target])
        } else {
            (&siblings[target], &siblings[pos])
        };
        let updated = codegen::swap_blocks(
            &content,
            earlier.span.start,
            earlier.span.end,
            later.span.start,
            later.span.end,
        );
        self.visual_editor.tabs[idx].content = updated;
        let mut new_path = path.clone();
        if let Some(last) = new_path.last_mut() {
            *last = target;
        }
        self.visual_editor.selected = Some(new_path);
        self.save_visual_editor_session();
    }

    /// The inspector bar shown above the block canvas.
    pub(super) fn visual_editor_inspector(
        &mut self,
        sel_valid: bool,
        can_up: bool,
        can_down: bool,
        window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> AnyElement {
        let p = theme::theme_palette(cx, is_dark);

        if !sel_valid {
            return div()
                .text_xs()
                .text_color(p.content_meta)
                .child("Select a block to edit its line. Enter commits, Esc deselects.")
                .into_any_element();
        }

        let focused = self.visual_editor.input_focus.is_focused(window);
        let blink = self.text_caret_blink_visible;
        let caret = self
            .visual_editor.edit_caret
            .min(self.visual_editor.edit_draft.len());
        let before = &self.visual_editor.edit_draft[..caret];
        let after = &self.visual_editor.edit_draft[caret..];
        let shown = if focused {
            format!("{}{}", text_with_trailing_caret(before, focused, blink), after)
        } else {
            self.visual_editor.edit_draft.clone()
        };

        let input_bg = if is_dark { p.row_bg } else { p.panel_bg };
        let fh = self.visual_editor.input_focus.clone();

        let editor = div()
            .flex_1()
            .px_2()
            .py_1()
            .rounded(px(6.))
            .bg(input_bg)
            .border_1()
            .border_color(p.panel_border)
            .font_family(MONO_FONT_FAMILY)
            .text_xs()
            .text_color(p.content_title)
            .track_focus(&self.visual_editor.input_focus)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |_, _, window, _| {
                    fh.focus(window);
                }),
            )
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                let key = event.keystroke.key.as_str();
                let mods = event.keystroke.modifiers;
                let mut caret = this
                    .visual_editor.edit_caret
                    .min(this.visual_editor.edit_draft.len());
                match key {
                    "enter" => {
                        this.visual_editor_commit_edit();
                        cx.notify();
                        return;
                    }
                    "escape" => {
                        this.visual_editor.selected = None;
                        cx.notify();
                        return;
                    }
                    "backspace" => {
                        if caret > 0 {
                            let prev = prev_boundary(&this.visual_editor.edit_draft, caret);
                            this.visual_editor.edit_draft.replace_range(prev..caret, "");
                            caret = prev;
                        }
                    }
                    "delete" => {
                        if caret < this.visual_editor.edit_draft.len() {
                            let next = next_boundary(&this.visual_editor.edit_draft, caret);
                            this.visual_editor.edit_draft.replace_range(caret..next, "");
                        }
                    }
                    "left" => caret = prev_boundary(&this.visual_editor.edit_draft, caret),
                    "right" => caret = next_boundary(&this.visual_editor.edit_draft, caret),
                    "home" => caret = 0,
                    "end" => caret = this.visual_editor.edit_draft.len(),
                    "space" => {
                        this.visual_editor.edit_draft.insert(caret, ' ');
                        caret += 1;
                    }
                    _ => {
                        if !mods.control && !mods.alt && !mods.platform && !mods.function {
                            if let Some(ch) = &event.keystroke.key_char {
                                this.visual_editor.edit_draft.insert_str(caret, ch);
                                caret += ch.len();
                            }
                        }
                    }
                }
                this.visual_editor.edit_caret = caret;
                cx.notify();
            }))
            .child(shown);

        let pill = |label: &'static str, enabled: bool| {
            let mut d = div()
                .px_2()
                .py_1()
                .rounded(px(6.))
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .bg(if is_dark { p.row_bg } else { p.panel_bg })
                .border_1()
                .border_color(p.panel_border);
            if enabled {
                d = d.cursor_pointer().text_color(p.content_title);
            } else {
                d = d.text_color(p.content_meta);
            }
            d.child(label)
        };

        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .child(editor)
            .child(pill("↑", can_up).when(can_up, |d| {
                d.on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, _, cx| {
                        this.visual_editor_move_selected(false);
                        cx.notify();
                    }),
                )
            }))
            .child(pill("↓", can_down).when(can_down, |d| {
                d.on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, _, cx| {
                        this.visual_editor_move_selected(true);
                        cx.notify();
                    }),
                )
            }))
            .child(pill("Delete", true).on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.visual_editor_delete_selected();
                    cx.notify();
                }),
            ))
            .into_any_element()
    }
}
