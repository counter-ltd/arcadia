use arcadia_core::modules::python_registry::list_style_tokens;
use openframe::prelude::FluentBuilder as _;
use openframe::{
    div, px, rgb, Context, FontWeight, InteractiveElement, IntoElement, MouseButton, ParentElement,
    Styled, Window,
};

use crate::gui::app::ArcadiaRoot;
use crate::gui::theme;

impl ArcadiaRoot {
    pub(crate) fn appearance_panel(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> impl IntoElement {
        let (
            panel_bg,
            panel_stroke,
            panel_radius,
            surface2,
            glyph_accent,
            glyph_dim,
            glyph_text,
            is_glyph,
        ) = {
            let g = theme::active_glyph(cx);
            (
                g.as_ref()
                    .map(|g| g.surface)
                    .unwrap_or_else(|| theme::module_panel_bg(is_dark)),
                g.as_ref()
                    .map(|g| g.border)
                    .unwrap_or_else(|| theme::module_panel_stroke(is_dark)),
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
                surface2.unwrap_or_else(|| {
                    if is_dark {
                        rgb(0x1e2433)
                    } else {
                        rgb(0xeef2ff)
                    }
                })
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
                glyph_dim.unwrap_or_else(|| {
                    if is_dark {
                        rgb(0x374151)
                    } else {
                        rgb(0xd1d5db)
                    }
                })
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
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, _, cx| {
                        this.apply_style(name.clone(), this.current_color_scheme_dark(), cx);
                    }),
                )
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
                        .child(div().text_xs().text_color(desc_color).child(description)),
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
                let Some(ref mid) = selected_style_module else {
                    return root;
                };
                root.child(self.extension_tokens_settings_card(
                    window,
                    cx,
                    is_dark,
                    mid.as_str(),
                    &token_specs,
                    "Extension tokens".to_string(),
                    "Overrides are saved per extension under ~/Arcadia/Configuration/extension_tokens/. Press Enter to save.".to_string(),
                    Some(mid.clone()),
                ))
            })
    }
}
