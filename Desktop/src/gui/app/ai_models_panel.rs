use arcadia_core::config::modules::AI_LLAMA_CPP_MODULE_NAME;
use arcadia_core::modules::ai::enabled_ai_providers;
use openframe::{
    div, px, Context, FontWeight, InteractiveElement, IntoElement, MouseButton, ParentElement,
    Styled, Window,
};

use crate::gui::app::ArcadiaRoot;
use crate::gui::theme::{self, GLYPH_PANEL_CONTENT_MAX_W_PX};

impl ArcadiaRoot {
    pub(crate) fn ai_models_panel(
        &mut self,
        _window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> impl IntoElement {
        let p = theme::theme_palette(cx, is_dark);
        let g_snap = theme::glyph_snapshot(cx);
        let radius = g_snap.map(|g| g.border_radius).unwrap_or(p.radius_md);

        // If a specific llama-cpp model is selected, show its detail view.
        if let Some(model_id) = self.active_llama_cpp_model_id.clone() {
            if let Some(model) = self.llama_cpp_models.iter().find(|m| m.id == model_id).cloned() {
                return div()
                    .w_full()
                    .max_w(px(GLYPH_PANEL_CONTENT_MAX_W_PX))
                    .flex()
                    .flex_col()
                    .gap_6()
                    // Header
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
                                    .child(model.name.clone()),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(p.content_meta)
                                    .child(format!("llama.cpp · {}", model.model_type.label())),
                            ),
                    )
                    // Path card
                    .child(
                        div()
                            .rounded(px(radius.min(12.0)))
                            .border_1()
                            .border_color(p.panel_border)
                            .bg(p.panel_bg)
                            .p_4()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(p.ui_subtext)
                                    .child("MODEL FILE"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(p.content_title)
                                    .child(if model.path.is_empty() {
                                        "No path configured.".to_string()
                                    } else {
                                        model.path.clone()
                                    }),
                            ),
                    )
                    .into_any_element();
            }
        }

        let providers = enabled_ai_providers(&self.module_rows);
        let active_module = self.active_ai_provider_module.clone();

        let mut root = div()
            .w_full()
            .max_w(px(GLYPH_PANEL_CONTENT_MAX_W_PX))
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
                            .child("Models"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(p.content_meta)
                            .child("Enabled AI provider modules and their configuration."),
                    ),
            );

        if providers.is_empty() {
            return root
                .child(
                    div()
                        .rounded(px(radius.min(12.0)))
                        .border_1()
                        .border_color(p.panel_border)
                        .bg(p.panel_bg)
                        .p_4()
                        .text_sm()
                        .text_color(p.content_meta)
                        .child("No provider modules enabled."),
                )
                .into_any_element();
        }

        for provider in &providers {
            let module_name = provider.module_name.to_string();
            let is_active = active_module == provider.module_name
                && self.active_llama_cpp_model_id.is_none();
            let pal = theme::nav_accent_palette("violet", is_dark);
            let card_border = if is_active { pal.row_hover } else { p.panel_border };

            let module_name_click = module_name.clone();
            let mut card = div()
                .rounded(px(radius.min(12.0)))
                .border_1()
                .border_color(card_border)
                .bg(p.panel_bg)
                .p_4()
                .cursor_pointer()
                .hover(move |s| s.border_color(pal.row_hover))
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .text_base()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(p.content_title)
                                .child(provider.display_name),
                        )
                        .child(
                            div()
                                .px_2()
                                .py_0p5()
                                .rounded(px(4.0))
                                .bg(p.badge_muted_bg)
                                .text_xs()
                                .text_color(p.badge_muted_fg)
                                .child("Not configured"),
                        ),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(p.content_meta)
                        .child(provider.description),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(p.content_meta)
                        .font_weight(FontWeight::MEDIUM)
                        .child(module_name.clone()),
                )
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, _, cx| {
                        this.active_ai_provider_module = module_name_click.clone();
                        this.active_llama_cpp_model_id = None;
                        cx.notify();
                    }),
                );

            // Show model list under llama.cpp card
            if provider.module_name == AI_LLAMA_CPP_MODULE_NAME && !self.llama_cpp_models.is_empty() {
                let mut model_list = div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .pt_2()
                    .border_t_1()
                    .border_color(p.panel_border);

                for model in &self.llama_cpp_models {
                    let model_id = model.id.clone();
                    let model_id_hover = model_id.clone();
                    let name = model.name.clone();
                    let type_label = model.model_type.label();
                    let is_model_active = self.active_llama_cpp_model_id.as_deref() == Some(model.id.as_str());
                    let row_bg = if is_model_active { pal.row_selected } else { p.panel_bg };
                    let row_hover = pal.row_hover;
                    let name_col = if is_model_active { pal.icon_active } else { p.content_title };

                    model_list = model_list.child(
                        div()
                            .px_2()
                            .py_1p5()
                            .rounded(px(radius.min(6.0)))
                            .bg(row_bg)
                            .hover(move |s| s.bg(row_hover))
                            .cursor_pointer()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(name_col)
                                    .child(name),
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py_0p5()
                                    .rounded(px(4.0))
                                    .bg(p.badge_muted_bg)
                                    .text_xs()
                                    .text_color(p.badge_muted_fg)
                                    .child(type_label),
                            )
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, _, cx| {
                                    this.active_llama_cpp_model_id = Some(model_id_hover.clone());
                                    this.active_ai_provider_module =
                                        AI_LLAMA_CPP_MODULE_NAME.to_string();
                                    cx.notify();
                                }),
                            ),
                    );
                }

                card = card.child(model_list);
            }

            root = root.child(card);
        }

        root.into_any_element()
    }
}
