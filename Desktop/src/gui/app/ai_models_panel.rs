use arcadia_core::config::modules::{AI_LLAMA_CPP_MODULE_NAME, AI_OLLAMA_MODULE_NAME, AI_OPENAI_MODULE_NAME};
use arcadia_core::modules::ai::enabled_ai_providers;
use openframe::{
    div, px, Context, FontWeight, InteractiveElement, IntoElement, KeyDownEvent, MouseButton,
    ParentElement, Styled, Window,
};
use openframe::prelude::FluentBuilder as _;

use crate::gui::app::ArcadiaRoot;
use crate::gui::app::text_input_caret::text_with_trailing_caret;
use crate::gui::theme::{self, GLYPH_PANEL_CONTENT_MAX_W_PX};

impl ArcadiaRoot {
    pub(crate) fn ai_models_panel(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> impl IntoElement {
        let p = theme::theme_palette(cx, is_dark);
        let g_snap = theme::glyph_snapshot(cx);
        let is_glyph = g_snap.is_some();
        let radius = g_snap.map(|g| g.border_radius).unwrap_or(p.radius_md);

        // If a specific llama-cpp model is selected, show its detail view.
        if let Some(model_id) = self.active_llama_cpp_model_id.clone() {
            if let Some(model) = self.llama_cpp_models.iter().find(|m| m.id == model_id).cloned() {
                return div()
                    .w_full()
                    .when(is_glyph, |d| d.max_w(px(GLYPH_PANEL_CONTENT_MAX_W_PX)))
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
                                    .child(model.name.clone()),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(p.content_meta)
                                    .child(format!("llama.cpp · {}", model.model_kind.label())),
                            ),
                    )
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
            .when(is_glyph, |d| d.max_w(px(GLYPH_PANEL_CONTENT_MAX_W_PX)))
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

            // llama.cpp: show model list
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
                    let type_label = model.model_kind.label();
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

            // Ollama: show discovered model list + Discover button
            if provider.module_name == AI_OLLAMA_MODULE_NAME {
                let discovering = self.ollama_discovering;
                let has_models = !self.ollama_models.is_empty();
                let ollama_models = self.ollama_models.clone();

                let discover_label = if discovering { "Discovering…" } else { "Discover Models" };
                let discover_btn = div()
                    .px_3()
                    .py_1()
                    .rounded(px(radius.min(8.0)))
                    .bg(p.surface_elevated)
                    .border_1()
                    .border_color(p.border)
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(if discovering { p.ui_subtext } else { p.content_title })
                    .cursor_pointer()
                    .child(discover_label)
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, window, cx| {
                        this.discover_ollama_models(window, cx);
                    }));

                let mut section = div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .pt_2()
                    .border_t_1()
                    .border_color(p.panel_border)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(p.ui_subtext)
                                    .child(if has_models {
                                        format!("{} model(s)", ollama_models.len())
                                    } else {
                                        "No models loaded".to_string()
                                    }),
                            )
                            .child(discover_btn),
                    );

                if has_models {
                    let mut model_list = div().flex().flex_col().gap_1();
                    for model in &ollama_models {
                        let name = model.name.clone();
                        let kind_label = model.model_kind.label();
                        model_list = model_list.child(
                            div()
                                .px_2()
                                .py_1p5()
                                .rounded(px(radius.min(6.0)))
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(p.content_title)
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
                                        .child(kind_label),
                                ),
                        );
                    }
                    section = section.child(model_list);
                }

                card = card.child(section);
            }

            // OpenAI: show api_key + base_url settings + models list
            if provider.module_name == AI_OPENAI_MODULE_NAME {
                let api_key_focused = self.openai_api_key_focus.is_focused(window);
                let base_url_focused = self.openai_base_url_focus.is_focused(window);
                let blink = self.text_caret_blink_visible;

                let api_key_display = if let Some(ref draft) = self.openai_api_key_draft {
                    text_with_trailing_caret(draft, api_key_focused, blink)
                } else if self.openai_api_key.is_empty() {
                    "Not set".to_string()
                } else {
                    // Mask all but last 4 chars
                    let key = &self.openai_api_key;
                    if key.len() > 4 {
                        format!("{}…{}", "•".repeat(8), &key[key.len() - 4..])
                    } else {
                        "•".repeat(key.len())
                    }
                };

                let base_url_display = if let Some(ref draft) = self.openai_base_url_draft {
                    text_with_trailing_caret(draft, base_url_focused, blink)
                } else {
                    self.openai_base_url.clone()
                };

                let editing_api_key = self.openai_api_key_draft.is_some();
                let editing_base_url = self.openai_base_url_draft.is_some();
                let api_key_fh = self.openai_api_key_focus.clone();
                let base_url_fh = self.openai_base_url_focus.clone();

                let openai_settings = div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .pt_2()
                    .border_t_1()
                    .border_color(p.panel_border)
                    // API Key row
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(p.ui_subtext)
                                    .child("API KEY"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .flex_1()
                                            .px_3()
                                            .py_2()
                                            .rounded(px(radius.min(8.0)))
                                            .bg(p.surface_elevated)
                                            .border_1()
                                            .border_color(if api_key_focused { p.accent } else { p.border })
                                            .text_sm()
                                            .text_color(if self.openai_api_key_draft.is_none() && self.openai_api_key.is_empty() {
                                                p.ui_subtext
                                            } else {
                                                p.content_title
                                            })
                                            .track_focus(&api_key_fh)
                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, window, cx| {
                                                this.openai_api_key_draft.get_or_insert_with(|| this.openai_api_key.clone());
                                                this.openai_api_key_focus.focus(window);
                                                cx.notify();
                                            }))
                                            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                                                if this.openai_api_key_draft.is_none() { return; }
                                                let key = event.keystroke.key.as_str();
                                                let mods = event.keystroke.modifiers;
                                                if key == "escape" {
                                                    this.openai_api_key_draft = None;
                                                } else if key == "enter" {
                                                    this.openai_save_settings(cx);
                                                    return;
                                                } else if let Some(ref mut d) = this.openai_api_key_draft {
                                                    if key == "backspace" { d.pop(); }
                                                    else if !mods.control && !mods.alt && !mods.platform && !mods.function {
                                                        if let Some(kc) = &event.keystroke.key_char { d.push_str(kc); }
                                                    }
                                                }
                                                cx.notify();
                                            }))
                                            .child(api_key_display),
                                    )
                                    .when(editing_api_key, |d| d.child(
                                        div()
                                            .px_3()
                                            .py_2()
                                            .rounded(px(radius.min(8.0)))
                                            .bg(p.accent)
                                            .text_xs()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(p.on_accent)
                                            .cursor_pointer()
                                            .child("Save")
                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                this.openai_save_settings(cx);
                                            })),
                                    )),
                            ),
                    )
                    // Base URL row
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(p.ui_subtext)
                                    .child("BASE URL"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .flex_1()
                                            .px_3()
                                            .py_2()
                                            .rounded(px(radius.min(8.0)))
                                            .bg(p.surface_elevated)
                                            .border_1()
                                            .border_color(if base_url_focused { p.accent } else { p.border })
                                            .text_sm()
                                            .text_color(p.content_title)
                                            .track_focus(&base_url_fh)
                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, window, cx| {
                                                this.openai_base_url_draft.get_or_insert_with(|| this.openai_base_url.clone());
                                                this.openai_base_url_focus.focus(window);
                                                cx.notify();
                                            }))
                                            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                                                if this.openai_base_url_draft.is_none() { return; }
                                                let key = event.keystroke.key.as_str();
                                                let mods = event.keystroke.modifiers;
                                                if key == "escape" {
                                                    this.openai_base_url_draft = None;
                                                } else if key == "enter" {
                                                    this.openai_save_settings(cx);
                                                    return;
                                                } else if let Some(ref mut d) = this.openai_base_url_draft {
                                                    if key == "backspace" { d.pop(); }
                                                    else if !mods.control && !mods.alt && !mods.platform && !mods.function {
                                                        if let Some(kc) = &event.keystroke.key_char { d.push_str(kc); }
                                                    }
                                                }
                                                cx.notify();
                                            }))
                                            .child(base_url_display),
                                    )
                                    .when(editing_base_url, |d| d.child(
                                        div()
                                            .px_3()
                                            .py_2()
                                            .rounded(px(radius.min(8.0)))
                                            .bg(p.accent)
                                            .text_xs()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(p.on_accent)
                                            .cursor_pointer()
                                            .child("Save")
                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                this.openai_save_settings(cx);
                                            })),
                                    )),
                            ),
                    );

                card = card.child(openai_settings);

                // OpenAI model list
                if !self.openai_models.is_empty() {
                    let openai_models = self.openai_models.clone();
                    let mut model_list = div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .pt_2()
                        .border_t_1()
                        .border_color(p.panel_border);
                    for model in &openai_models {
                        let name = model.name.clone();
                        let kind_label = model.model_kind.label();
                        let model_id_label = model.model_id.clone();
                        model_list = model_list.child(
                            div()
                                .px_2()
                                .py_1p5()
                                .rounded(px(radius.min(6.0)))
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_0p5()
                                        .child(
                                            div()
                                                .text_sm()
                                                .text_color(p.content_title)
                                                .child(name),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(p.content_meta)
                                                .child(model_id_label),
                                        ),
                                )
                                .child(
                                    div()
                                        .px_2()
                                        .py_0p5()
                                        .rounded(px(4.0))
                                        .bg(p.badge_muted_bg)
                                        .text_xs()
                                        .text_color(p.badge_muted_fg)
                                        .child(kind_label),
                                ),
                        );
                    }
                    card = card.child(model_list);
                }
            }

            root = root.child(card);
        }

        root.into_any_element()
    }
}
