//! Shared extension token editor (persisted under `extension_tokens/`).

use arcadia_core::modules::python_registry::{StyleTokenKind, StyleTokenSpec};
use openframe::{
    AnyElement, Context, FontWeight, InteractiveElement, IntoElement, KeyDownEvent, MouseButton,
    ParentElement, Rgba, Styled, Window, div, px, rgb,
};
use crate::gui::app::text_input_caret::text_with_trailing_caret;
use crate::gui::app::ArcadiaRoot;
use crate::gui::theme;

fn token_edit_rgba(display: &str, default_s: &str) -> Rgba {
    crate::gui::app::lifecycle::parse_hex_color(display.trim())
        .or_else(|| crate::gui::app::lifecycle::parse_hex_color(default_s.trim()))
        .unwrap_or_else(|| rgb(0x00cc88))
}

fn token_kind_label(kind: StyleTokenKind) -> &'static str {
    match kind {
        StyleTokenKind::Color => "color",
        StyleTokenKind::Float => "float",
        StyleTokenKind::String => "string",
        StyleTokenKind::Bool => "bool",
        StyleTokenKind::Int => "int",
    }
}

impl ArcadiaRoot {
    pub(crate) fn extension_tokens_settings_card(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
        module_id: &str,
        specs: &[StyleTokenSpec],
        title: String,
        help: String,
        extension_label: Option<String>,
    ) -> openframe::Div {
        let (panel_bg, panel_stroke, panel_radius, header_color, subtext_color, _is_glyph) = {
            let g = theme::active_glyph(cx);
            (
                g.as_ref().map(|g| g.surface).unwrap_or_else(|| theme::module_panel_bg(is_dark)),
                g.as_ref().map(|g| g.border).unwrap_or_else(|| theme::module_panel_stroke(is_dark)),
                g.as_ref().map(|g| g.border_radius).unwrap_or(8.0),
                g.as_ref()
                    .map(|g| g.text)
                    .unwrap_or_else(|| theme::module_title_text(is_dark)),
                g.as_ref()
                    .map(|g| g.dim)
                    .unwrap_or_else(|| theme::module_meta_text(is_dark)),
                g.is_some(),
            )
        };
        let ext_token_focus = self.extension_token_focus.clone();
        let input_bg = theme::glyph_snapshot(cx)
            .map(|g| g.surface)
            .unwrap_or_else(|| theme::ui_surface(cx, is_dark));
        let input_border = theme::glyph_snapshot(cx)
            .map(|g| g.border)
            .unwrap_or_else(|| theme::ui_border(cx, is_dark));

        let module_owned = module_id.to_string();
        let rows = specs.iter().map(|spec| {
            let key = spec.key.clone();
            let label = spec.label.clone();
            let kind = spec.kind;
            let pair = (module_owned.clone(), key.clone());
            let display_val = self
                .extension_token_values
                .get(&pair)
                .cloned()
                .unwrap_or_else(|| spec.default_value.clone());
            let is_editing = self
                .extension_token_editing
                .as_ref()
                .is_some_and(|(m, k)| m == &module_owned && k == &key);

            let row_module = module_owned.clone();
            let row_key = key.clone();
            let row_default = spec.default_value.clone();

            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(1.))
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(header_color)
                                        .child(label),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(subtext_color)
                                        .child(format!(
                                            "{} · {}",
                                            key,
                                            token_kind_label(kind)
                                        )),
                                ),
                        ),
                )
                .child({
                    let edit_cell: AnyElement = if kind == StyleTokenKind::Color {
                        let preview = token_edit_rgba(&display_val, &row_default);
                        div()
                            .px_3()
                            .py_2()
                            .rounded(px(panel_radius))
                            .bg(input_bg)
                            .border_1()
                            .border_color(input_border)
                            .text_sm()
                            .text_color(header_color)
                            .cursor_pointer()
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .w(px(14.))
                                            .h(px(14.))
                                            .rounded(px(3.))
                                            .border_1()
                                            .border_color(input_border)
                                            .bg(preview),
                                    )
                                    .child(div().child(display_val.clone())),
                            )
                            .on_mouse_down(MouseButton::Left, cx.listener({
                                let m = row_module.clone();
                                let k = row_key.clone();
                                let d = row_default.clone();
                                let current = display_val.clone();
                                move |this, _, _, cx| {
                                    if let Some((ref em, ref ek)) = this.extension_token_editing.clone() {
                                        this.flush_extension_token_edit(em.clone(), ek.clone(), cx);
                                    }
                                    this.extension_token_editing = None;
                                    this.color_picker_modal = Some((m.clone(), k.clone(), current.clone(), d.clone()));
                                    cx.notify();
                                }
                            }))
                            .into_any_element()
                    } else if is_editing {
                        let show_caret = ext_token_focus.is_focused(window) && is_editing;
                        let blink = self.text_caret_blink_visible;
                        div()
                            .px_3()
                            .py_2()
                            .rounded(px(panel_radius))
                            .bg(input_bg)
                            .border_1()
                            .border_color(input_border)
                            .text_sm()
                            .text_color(header_color)
                            .track_focus(&ext_token_focus)
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, window, _| {
                                    this.extension_token_focus.focus(window);
                                }),
                            )
                            .child(div().child(text_with_trailing_caret(
                                display_val.as_str(),
                                show_caret,
                                blink,
                            )))
                            .on_key_down(cx.listener({
                                let m = row_module.clone();
                                let k = row_key.clone();
                                let default_for_esc = row_default.clone();
                                move |this, event: &KeyDownEvent, _, cx| {
                                    let key_ev = event.keystroke.key.as_str();
                                    let mods = event.keystroke.modifiers;
                                    if key_ev == "enter" || key_ev == "return" {
                                        this.flush_extension_token_edit(m.clone(), k.clone(), cx);
                                        this.extension_token_editing = None;
                                        cx.notify();
                                        return;
                                    }
                                    if key_ev == "escape" {
                                        let file = arcadia_core::config::extension_tokens::load_module_tokens(&m)
                                            .unwrap_or_default();
                                        let v = arcadia_core::config::extension_tokens::merged_display_for_key(
                                            &k,
                                            &default_for_esc,
                                            &file,
                                        );
                                        this.extension_token_values.insert((m.clone(), k.clone()), v);
                                        this.extension_token_editing = None;
                                        cx.notify();
                                        return;
                                    }
                                    let pair = (m.clone(), k.clone());
                                    let entry = this.extension_token_values.entry(pair.clone()).or_insert_with(String::new);
                                    if key_ev == "backspace" {
                                        entry.pop();
                                        cx.notify();
                                    } else if key_ev == "space" {
                                        entry.push(' ');
                                        cx.notify();
                                    } else if !mods.control
                                        && !mods.alt
                                        && !mods.platform
                                        && !mods.function
                                    {
                                        if let Some(ch) = &event.keystroke.key_char {
                                            entry.push_str(ch);
                                            cx.notify();
                                        }
                                    }
                                }
                            }))
                            .into_any_element()
                    } else {
                        div()
                            .px_3()
                            .py_2()
                            .rounded(px(panel_radius))
                            .bg(input_bg)
                            .border_1()
                            .border_color(input_border)
                            .text_sm()
                            .text_color(header_color)
                            .cursor_pointer()
                            .child(div().child(display_val.clone()))
                            .on_mouse_down(MouseButton::Left, cx.listener({
                                let m = row_module.clone();
                                let k = row_key.clone();
                                move |this, _, window, cx| {
                                    if let Some((ref em, ref ek)) = this.extension_token_editing.clone() {
                                        if em != &m || ek != &k {
                                            this.flush_extension_token_edit(em.clone(), ek.clone(), cx);
                                        }
                                    }
                                    this.extension_token_editing = Some((m.clone(), k.clone()));
                                    this.extension_token_focus.focus(window);
                                    cx.notify();
                                }
                            }))
                            .into_any_element()
                    };
                    edit_cell
                })
        });

        let mut outer = div()
            .w_full()
            .p_4()
            .rounded(px(panel_radius))
            .bg(panel_bg)
            .border_1()
            .border_color(panel_stroke)
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(header_color)
                            .child(title.clone()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(subtext_color)
                            .child(help.clone()),
                    ),
            );
        if let Some(lbl) = extension_label {
            outer = outer.child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(subtext_color)
                    .child(format!("Extension: {lbl}")),
            );
        }
        outer.children(rows)
    }
}
