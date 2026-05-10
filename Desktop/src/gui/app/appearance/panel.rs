use arcadia_core::modules::python_registry::{list_style_tokens, StyleTokenKind};
use openframe::{
    AnyElement, Rgba, div, px, rgb, Context, FontWeight, InteractiveElement,
    IntoElement, KeyDownEvent, MouseButton, ParentElement, Styled,
};
use openframe::prelude::FluentBuilder as _;

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
    pub(crate) fn appearance_panel(
        &mut self,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> impl IntoElement {
        let (panel_bg, panel_stroke, panel_radius, surface2, glyph_accent, glyph_dim, glyph_text, is_glyph) = {
            let g = theme::active_glyph(cx);
            (
                g.as_ref().map(|g| g.surface).unwrap_or_else(|| theme::module_panel_bg(is_dark)),
                g.as_ref().map(|g| g.border).unwrap_or_else(|| theme::module_panel_stroke(is_dark)),
                g.as_ref().map(|g| g.border_radius).unwrap_or(8.0),
                g.as_ref().map(|g| g.surface2),
                g.as_ref().map(|g| g.accent),
                g.as_ref().map(|g| g.dim),
                g.as_ref().map(|g| g.text),
                g.is_some(),
            )
        };
        let header_color = glyph_text.unwrap_or_else(|| theme::module_title_text(is_dark));
        let subtext_color = glyph_dim.unwrap_or_else(|| theme::module_meta_text(is_dark));

        let ext_token_focus = self.extension_token_focus.clone();

        let mut styles = self.available_styles.clone();
        styles.sort_by(|a, b| {
            a.label
                .to_ascii_lowercase()
                .cmp(&b.label.to_ascii_lowercase())
                .then_with(|| a.name.cmp(&b.name))
        });
        let active = self.active_style.clone();

        let style_rows = styles.into_iter().map(|info| {
            let name = info.name.clone();
            let label = info.label.clone();
            let description = info.description.clone();
            let is_selected = name == active;

            let row_bg = if is_selected {
                surface2.unwrap_or_else(|| if is_dark { rgb(0x1e2433) } else { rgb(0xeef2ff) })
            } else {
                panel_bg
            };
            let row_stroke = if is_selected {
                glyph_accent.unwrap_or(rgb(0x6366f1))
            } else {
                panel_stroke
            };
            let label_color = header_color;
            let desc_color = subtext_color;
            let indicator_color = if is_selected {
                glyph_accent.unwrap_or(rgb(0x6366f1))
            } else {
                glyph_dim.unwrap_or_else(|| if is_dark { rgb(0x374151) } else { rgb(0xd1d5db) })
            };

            div()
                .w_full()
                .px_4()
                .py_3()
                .rounded(px(panel_radius))
                .bg(row_bg)
                .border_1()
                .border_color(row_stroke)
                .flex()
                .items_center()
                .gap_3()
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                    this.apply_style(name.clone(), this.current_color_scheme_dark(), cx);
                }))
                .child(
                    div()
                        .w(px(10.))
                        .h(px(10.))
                        .when(!is_glyph, |d| d.rounded_full())
                        .rounded(px(panel_radius))
                        .bg(indicator_color)
                        .flex_shrink_0(),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(2.))
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(label_color)
                                .child(label),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(desc_color)
                                .child(description),
                        ),
                )
        });

        let selected_style_module = self
            .available_styles
            .iter()
            .find(|s| s.name == self.active_style)
            .and_then(|s| s.module_name.clone());
        let token_specs = selected_style_module
            .as_ref()
            .and_then(|module_id| {
                list_style_tokens()
                    .into_iter()
                    .find(|(m, _)| m == module_id)
                    .map(|(_, specs)| specs)
            })
            .unwrap_or_default();
        let show_extension_tokens = selected_style_module.is_some() && !token_specs.is_empty();
        let input_bg = theme::glyph_snapshot(cx)
            .map(|g| g.surface)
            .unwrap_or_else(|| theme::ui_surface(cx, is_dark));
        let input_border = theme::glyph_snapshot(cx)
            .map(|g| g.border)
            .unwrap_or_else(|| theme::ui_border(cx, is_dark));

        let token_sections = selected_style_module.into_iter().map(|module_id| {
            let heading = module_id.clone();
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(subtext_color)
                        .child(format!("Extension: {heading}")),
                )
                .children(token_specs.clone().into_iter().map(|spec| {
                    let key = spec.key.clone();
                    let label = spec.label.clone();
                    let kind = spec.kind;
                    let pair = (module_id.clone(), key.clone());
                    let display_val = self
                        .extension_token_values
                        .get(&pair)
                        .cloned()
                        .unwrap_or_else(|| spec.default_value.clone());
                    let is_editing = self
                        .extension_token_editing
                        .as_ref()
                        .is_some_and(|(m, k)| m == &module_id && k == &key);

                    let row_module = module_id.clone();
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
                                            if let Some((ref em, ref ek)) =
                                                this.extension_token_editing.clone()
                                            {
                                                this.flush_extension_token_edit(
                                                    em.clone(),
                                                    ek.clone(),
                                                    cx,
                                                );
                                            }
                                            this.extension_token_editing = None;
                                            this.color_picker_modal = Some((
                                                m.clone(),
                                                k.clone(),
                                                current.clone(),
                                                d.clone(),
                                            ));
                                            cx.notify();
                                        }
                                    }))
                                    .into_any_element()
                            } else if is_editing {
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
                                    .child(div().child(display_val.clone()))
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
                                                let file =
                                                    arcadia_core::config::extension_tokens::load_module_tokens(&m)
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
                                            let entry = this
                                                .extension_token_values
                                                .entry(pair.clone())
                                                .or_insert_with(String::new);
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
                                            if let Some((ref em, ref ek)) =
                                                this.extension_token_editing.clone()
                                            {
                                                if em != &m || ek != &k {
                                                    this.flush_extension_token_edit(
                                                        em.clone(),
                                                        ek.clone(),
                                                        cx,
                                                    );
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
                }))
        });

        div()
            .w_full()
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
                            .text_color(header_color)
                            .child("Appearance"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(subtext_color)
                            .child("Choose a render style. Changes apply live."),
                    ),
            )
            .child(
                div()
                    .w_full()
                    .p_4()
                    .rounded(px(panel_radius))
                    .bg(panel_bg)
                    .border_1()
                    .border_color(panel_stroke)
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(header_color)
                            .child("Style"),
                    )
                    .children(style_rows),
            )
            .when(show_extension_tokens, |root| {
                root.child(
                    div()
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
                                        .child("Extension tokens"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(subtext_color)
                                        .child(
                                            "Overrides are saved per extension under ~/Arcadia/Configuration/extension_tokens/. Press Enter to save.",
                                        ),
                                ),
                        )
                        .children(token_sections),
                )
            })
    }
}
