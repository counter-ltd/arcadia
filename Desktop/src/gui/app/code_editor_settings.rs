use arcadia_core::config::code_editor::CodeEditorConfig;
use arcadia_core::config::ConfigFile;
use arcadia_core::modules::python_registry;
use openframe::{
    div, px, Context, FontWeight, InteractiveElement, IntoElement, KeyDownEvent, MouseButton,
    ParentElement, Styled, Window,
};

use openframe::prelude::FluentBuilder as _;

use crate::gui::app::text_input_caret::text_with_trailing_caret;
use crate::gui::app::ArcadiaRoot;
use crate::gui::theme::{self, GLYPH_PANEL_CONTENT_MAX_W_PX};

impl ArcadiaRoot {
    pub(crate) fn code_editor_settings_panel(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> impl IntoElement {
        let p = theme::theme_palette(cx, is_dark);
        let g_snap = theme::glyph_snapshot(cx);
        let panel_radius = g_snap.map(|g| g.border_radius).unwrap_or(p.radius_md);
        let show_marks = self.code_editor_show_indentation_marks;

        let editor_token_modules = python_registry::editor_scoped_token_modules();

        let mut root = div()
            .w_full()
            .when(g_snap.is_some(), |d| {
                d.max_w(px(GLYPH_PANEL_CONTENT_MAX_W_PX))
            })
            .flex()
            .flex_col()
            .gap_6()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_2xl()
                            .font_weight(FontWeight::BOLD)
                            .text_color(p.content_title)
                            .child("Editor"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(p.content_meta)
                            .child("Code editor display and formatting preferences."),
                    ),
            )
            .child(
                div()
                    .rounded(px(panel_radius.min(12.0)))
                    .border_1()
                    .border_color(p.panel_border)
                    .bg(p.panel_bg)
                    .overflow_hidden()
                    .child(self.indentation_marks_row(cx, is_dark, show_marks, panel_radius))
                    .child(div().w_full().h(px(1.)).bg(p.panel_border))
                    .child(self.char_width_row(window, cx, is_dark, panel_radius)),
            );

        for (module_id, specs) in editor_token_modules {
            let card = self.extension_tokens_settings_card(
                window,
                cx,
                is_dark,
                &module_id,
                &specs,
                module_id
                    .split('-')
                    .map(|w| {
                        let mut c = w.chars();
                        match c.next() {
                            None => String::new(),
                            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" "),
                "Saved under ~/Arcadia/Configuration/extension_tokens/. Press Enter to save."
                    .to_string(),
                None,
            );
            root = root.child(card);
        }

        root
    }

    fn indentation_marks_row(
        &self,
        cx: &mut Context<Self>,
        _is_dark: bool,
        enabled: bool,
        panel_radius: f32,
    ) -> impl IntoElement {
        let p = theme::theme_palette(cx, _is_dark);
        let _ = panel_radius;

        let toggle = if enabled {
            div()
                .w_10()
                .h_6()
                .px_0p5()
                .rounded_full()
                .border_1()
                .border_color(p.border)
                .bg(p.accent)
                .flex()
                .items_center()
                .justify_end()
                .child(div().w_4().h_4().rounded_full().bg(p.on_accent))
        } else {
            div()
                .w_10()
                .h_6()
                .px_0p5()
                .rounded_full()
                .border_1()
                .border_color(p.border)
                .bg(p.surface_elevated)
                .flex()
                .items_center()
                .justify_start()
                .child(div().w_4().h_4().rounded_full().bg(p.toggle_knob_off))
        };

        div()
            .w_full()
            .px_4()
            .py_3()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .gap_4()
            .cursor_pointer()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(p.content_title)
                            .child("Indentation Marks"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(p.content_meta)
                            .child("Show guides at each indentation level."),
                    ),
            )
            .child(toggle)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.code_editor_show_indentation_marks =
                        !this.code_editor_show_indentation_marks;
                    let mut cfg = CodeEditorConfig::load_or_create().unwrap_or_default();
                    cfg.show_indentation_marks = this.code_editor_show_indentation_marks;
                    let _ = cfg.save();
                    cx.notify();
                }),
            )
    }

    fn char_width_row(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
        panel_radius: f32,
    ) -> impl IntoElement {
        let p = theme::theme_palette(cx, is_dark);
        let input_radius = panel_radius.min(8.0);
        let editing = self.code_editor_char_width_editing;
        let focused = self.code_editor_char_width_focus.is_focused(window);
        let blink = self.text_caret_blink_visible;
        let draft = self.code_editor_char_width_draft.clone();
        let cw_focus = self.code_editor_char_width_focus.clone();

        let display = if editing {
            text_with_trailing_caret(&draft, focused, blink)
        } else {
            self.code_editor_char_width_override
                .map(|v| format!("{:.2}", v))
                .unwrap_or_else(|| "auto".to_string())
        };

        let input_color = if editing {
            p.content_title
        } else if self.code_editor_char_width_override.is_some() {
            p.content_title
        } else {
            p.content_meta
        };

        let input_el = div()
            .w(px(80.))
            .px_3()
            .py_1p5()
            .rounded(px(input_radius))
            .border_1()
            .border_color(if editing { p.accent } else { p.border })
            .bg(p.surface_elevated)
            .text_sm()
            .text_color(input_color)
            .cursor_pointer()
            .track_focus(&cw_focus)
            .child(display)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, window, cx| {
                    if !this.code_editor_char_width_editing {
                        this.code_editor_char_width_draft = this
                            .code_editor_char_width_override
                            .map(|v| format!("{:.2}", v))
                            .unwrap_or_default();
                        this.code_editor_char_width_editing = true;
                    }
                    this.code_editor_char_width_focus.focus(window);
                    cx.notify();
                }),
            )
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                if !this.code_editor_char_width_editing {
                    return;
                }
                let key = event.keystroke.key.as_str();
                let mods = event.keystroke.modifiers;
                match key {
                    "enter" | "return" => {
                        let parsed = if this.code_editor_char_width_draft.is_empty() {
                            None
                        } else {
                            this.code_editor_char_width_draft
                                .parse::<f32>()
                                .ok()
                                .map(|v| v.max(1.0))
                        };
                        this.code_editor_char_width_override = parsed;
                        this.code_editor_char_width_draft =
                            parsed.map(|v| format!("{:.2}", v)).unwrap_or_default();
                        this.code_editor_char_width_editing = false;
                        let mut cfg = CodeEditorConfig::load_or_create().unwrap_or_default();
                        cfg.char_width_override = parsed;
                        let _ = cfg.save();
                        cx.notify();
                    }
                    "escape" => {
                        this.code_editor_char_width_draft = this
                            .code_editor_char_width_override
                            .map(|v| format!("{:.2}", v))
                            .unwrap_or_default();
                        this.code_editor_char_width_editing = false;
                        cx.notify();
                    }
                    "backspace" => {
                        this.code_editor_char_width_draft.pop();
                        cx.notify();
                    }
                    _ => {
                        if !mods.control && !mods.alt && !mods.platform && !mods.function {
                            if let Some(kc) = &event.keystroke.key_char {
                                // Only allow digits and one decimal point
                                let c = kc.chars().next().unwrap_or('\0');
                                if c.is_ascii_digit()
                                    || (c == '.'
                                        && !this.code_editor_char_width_draft.contains('.'))
                                {
                                    this.code_editor_char_width_draft.push(c);
                                    cx.notify();
                                }
                            }
                        }
                    }
                }
            }));

        div()
            .w_full()
            .px_4()
            .py_3()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .gap_4()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(p.content_title)
                            .child("Character Width"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(p.content_meta)
                            .child("Monospace advance width in px. Empty = auto from font."),
                    ),
            )
            .child(input_el)
    }
}
