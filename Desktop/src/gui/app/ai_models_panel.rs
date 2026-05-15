use arcadia_core::config::modules::{
    AI_LLAMA_CPP_MODULE_NAME, AI_OLLAMA_MODULE_NAME, AI_OPENAI_MODULE_NAME, MODULE_REGISTRY,
};
use arcadia_core::modules::ai::{
    cli_binary_for_module, cli_display_name, enabled_ai_providers, is_cli_provider,
};
use openframe::prelude::FluentBuilder as _;
use openframe::{
    div, px, AnyElement, Context, FontWeight, InteractiveElement, IntoElement, KeyDownEvent,
    MouseButton, ParentElement, Styled, Window,
};

use crate::gui::app::text_input_caret::text_with_trailing_caret;
use crate::gui::app::ArcadiaRoot;
use crate::gui::app::LlamaCppModelCreateDraft;
use crate::gui::theme::{self, render_icon, GLYPH_PANEL_CONTENT_MAX_W_PX};

pub(crate) fn provider_accent(module_name: &str) -> &'static str {
    use arcadia_core::config::modules::*;
    match module_name {
        AI_EXEC_CLAUDE_MODULE_NAME => "orange",
        AI_EXEC_GEMINI_MODULE_NAME => "sky",
        AI_EXEC_CODEX_MODULE_NAME => "cyan",
        AI_OPENAI_MODULE_NAME => "emerald",
        AI_EXEC_AIDER_MODULE_NAME => "emerald",
        AI_LLAMA_CPP_MODULE_NAME => "amber",
        AI_OLLAMA_MODULE_NAME => "cyan",
        AI_APFEL_MODULE_NAME => "indigo",
        _ => "violet",
    }
}

fn provider_glyph(module_name: &str) -> &'static str {
    MODULE_REGISTRY
        .iter()
        .find(|m| m.name == module_name)
        .map(|m| m.glyph)
        .unwrap_or("ai-provider")
}

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
            if let Some(model) = self
                .llama_cpp_models
                .iter()
                .find(|m| m.id == model_id)
                .cloned()
            {
                // Edit mode — delegate to edit form renderer.
                if self.llama_cpp_edit_draft.is_some() {
                    return self.llama_cpp_edit_model_form(window, cx, is_dark);
                }

                let is_delete_confirm = self.llama_cpp_delete_confirm;
                let is_vision =
                    model.model_kind == arcadia_core::config::llama_cpp::LlamaCppModelKind::Vision;
                let model_for_edit = model.clone();
                let pal = theme::nav_accent_palette("violet", is_dark);
                let delete_bg = if is_delete_confirm {
                    p.danger
                } else {
                    p.surface_elevated
                };
                let delete_fg = if is_delete_confirm {
                    p.on_accent
                } else {
                    p.danger
                };
                let delete_border = if is_delete_confirm {
                    p.danger
                } else {
                    p.border
                };

                let mut detail = div()
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
                            .child(div().text_sm().text_color(p.content_title).child(
                                if model.path.is_empty() {
                                    "No path configured.".to_string()
                                } else {
                                    model.path.clone()
                                },
                            )),
                    );

                if is_vision {
                    detail = detail.child(
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
                                    .child("MMPROJ FILE"),
                            )
                            .child(
                                div().text_sm().text_color(p.content_title).child(
                                    model
                                        .mmproj_path
                                        .as_deref()
                                        .unwrap_or("No path configured.")
                                        .to_string(),
                                ),
                            ),
                    );
                }

                detail = detail.child(
                    div()
                        .flex()
                        .gap_2()
                        .child(
                            div()
                                .px_3()
                                .py_1p5()
                                .rounded(px(radius.min(8.0)))
                                .bg(p.surface_elevated)
                                .border_1()
                                .border_color(p.border)
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(p.content_title)
                                .cursor_pointer()
                                .hover(move |s| s.border_color(pal.row_hover))
                                .child("Edit")
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |this, _, _, cx| {
                                        this.llama_cpp_edit_draft =
                                            Some(LlamaCppModelCreateDraft {
                                                name: model_for_edit.name.clone(),
                                                path: model_for_edit.path.clone(),
                                                mmproj_path: model_for_edit
                                                    .mmproj_path
                                                    .clone()
                                                    .unwrap_or_default(),
                                                model_kind: model_for_edit.model_kind.clone(),
                                                error: None,
                                            });
                                        this.llama_cpp_delete_confirm = false;
                                        cx.notify();
                                    }),
                                ),
                        )
                        .child(
                            div()
                                .px_3()
                                .py_1p5()
                                .rounded(px(radius.min(8.0)))
                                .bg(delete_bg)
                                .border_1()
                                .border_color(delete_border)
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(delete_fg)
                                .cursor_pointer()
                                .child(if is_delete_confirm {
                                    "Confirm Delete"
                                } else {
                                    "Delete"
                                })
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|this, _, _, cx| {
                                        if this.llama_cpp_delete_confirm {
                                            this.llama_cpp_delete_model(cx);
                                        } else {
                                            this.llama_cpp_delete_confirm = true;
                                            cx.notify();
                                        }
                                    }),
                                ),
                        ),
                );

                return detail.into_any_element();
            }
        }

        // OpenAI: create form early-return (takes priority over detail view).
        if let Some(ref draft) = self.openai_create_draft.clone() {
            return self.openai_provider_create_form(window, cx, is_dark, draft.clone()).into_any_element();
        }

        // OpenAI: provider detail / edit view early-return.
        if let Some(provider_id) = self.active_openai_provider_id.clone() {
            if let Some(provider) = self.openai_providers.iter().find(|p| p.id == provider_id).cloned() {
                if self.openai_provider_edit_draft.is_some() {
                    return self.openai_provider_edit_form(window, cx, is_dark, provider.clone()).into_any_element();
                }
                return self.openai_provider_detail(window, cx, is_dark, provider).into_any_element();
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

        let cli_providers: Vec<&arcadia_core::modules::ai::AiProviderManifest> = providers
            .iter()
            .copied()
            .filter(|p| is_cli_provider(p.module_name))
            .collect();

        let mut cli_card_inserted = false;

        for provider in &providers {
            if is_cli_provider(provider.module_name) {
                continue;
            }

            // Insert CLI card before the first provider that sorts after it alphabetically.
            if !cli_card_inserted
                && !cli_providers.is_empty()
                && provider.display_name.to_lowercase().as_str() > "cli providers"
            {
                cli_card_inserted = true;
                root = root.child(self.build_cli_card_element(
                    &cli_providers,
                    &active_module,
                    &p,
                    radius,
                    is_dark,
                    cx,
                ));
            }

            let module_name = provider.module_name.to_string();
            let _is_active =
                active_module == provider.module_name && self.active_llama_cpp_model_id.is_none();
            let pal = theme::nav_accent_palette(provider_accent(provider.module_name), is_dark);
            let card_border = pal.icon_idle;
            let glyph_key = provider_glyph(provider.module_name);
            let title_col = pal.icon_active;

            let module_name_click = module_name.clone();
            let mut card = div()
                .rounded(px(radius.min(12.0)))
                .border_1()
                .border_color(card_border)
                .bg(p.panel_bg)
                .p_4()
                .cursor_pointer()
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
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(render_icon(glyph_key).size_5().text_color(title_col))
                                .child(
                                    div()
                                        .text_base()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(title_col)
                                        .child(provider.display_name),
                                ),
                        ),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(p.content_meta)
                        .child(provider.description),
                )
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, _, cx| {
                        this.active_ai_provider_module = module_name_click.clone();
                        this.active_llama_cpp_model_id = None;
                        this.llama_cpp_edit_draft = None;
                        this.llama_cpp_delete_confirm = false;
                        cx.notify();
                    }),
                );

            // llama.cpp: show model list + Create Model button
            if provider.module_name == AI_LLAMA_CPP_MODULE_NAME {
                if !self.llama_cpp_models.is_empty() {
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
                        let type_icon = model.model_kind.icon_key();
                        let is_model_active =
                            self.active_llama_cpp_model_id.as_deref() == Some(model.id.as_str());
                        let row_bg = if is_model_active {
                            pal.row_selected
                        } else {
                            p.panel_bg
                        };
                        let row_hover = pal.row_hover;
                        let name_col = if is_model_active {
                            pal.icon_active
                        } else {
                            p.content_title
                        };

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
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .child(render_icon(type_icon).size_4().text_color(name_col))
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(name_col)
                                                .child(name),
                                        ),
                                )
                                .child(
                                    div()
                                        .px_1p5()
                                        .py_0p5()
                                        .rounded(px(4.0))
                                        .bg(p.badge_muted_bg)
                                        .flex()
                                        .items_center()
                                        .gap_1()
                                        .child(
                                            render_icon(type_icon)
                                                .size_3()
                                                .text_color(p.badge_muted_fg),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(p.badge_muted_fg)
                                                .child(type_label),
                                        ),
                                )
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |this, _, _, cx| {
                                        cx.stop_propagation();
                                        this.active_llama_cpp_model_id = Some(model_id_hover.clone());
                                        this.active_ai_provider_module =
                                            AI_LLAMA_CPP_MODULE_NAME.to_string();
                                        this.llama_cpp_edit_draft = None;
                                        this.llama_cpp_delete_confirm = false;
                                        cx.notify();
                                    }),
                                ),
                        );
                    }

                    card = card.child(model_list);
                }

                let has_models = !self.llama_cpp_models.is_empty();
                let create_btn = div()
                    .px_3()
                    .py_1()
                    .rounded(px(radius.min(8.0)))
                    .bg(p.surface_elevated)
                    .border_1()
                    .border_color(p.border)
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(p.content_title)
                    .cursor_pointer()
                    .child("Create Model")
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.llama_cpp_create_draft =
                                Some(crate::gui::app::LlamaCppModelCreateDraft {
                                    name: String::new(),
                                    path: String::new(),
                                    mmproj_path: String::new(),
                                    model_kind: arcadia_core::config::llama_cpp::LlamaCppModelKind::TextGeneration,
                                    error: None,
                                });
                            cx.notify();
                        }),
                    );

                card = card.child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .pt_2()
                        .border_t_1()
                        .border_color(p.panel_border)
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(p.ui_subtext)
                                .child(if has_models {
                                    format!("{} model(s)", self.llama_cpp_models.len())
                                } else {
                                    "No models configured".to_string()
                                }),
                        )
                        .child(create_btn),
                );
            }

            // Ollama: show discovered model list + Discover button
            if provider.module_name == AI_OLLAMA_MODULE_NAME {
                let discovering = self.ollama_discovering;
                let has_models = !self.ollama_models.is_empty();
                let ollama_models = self.ollama_models.clone();

                let discover_label = if discovering {
                    "Discovering…"
                } else {
                    "Discover Models"
                };
                let discover_btn = div()
                    .px_3()
                    .py_1()
                    .rounded(px(radius.min(8.0)))
                    .bg(p.surface_elevated)
                    .border_1()
                    .border_color(p.border)
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(if discovering {
                        p.ui_subtext
                    } else {
                        p.content_title
                    })
                    .cursor_pointer()
                    .child(discover_label)
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, window, cx| {
                            this.discover_ollama_models(window, cx);
                        }),
                    );

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
                        let kind_icon = model.model_kind.icon_key();
                        model_list = model_list.child(
                            div()
                                .px_2()
                                .py_1p5()
                                .rounded(px(radius.min(6.0)))
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(div().text_sm().text_color(p.content_title).child(name))
                                .child(
                                    div()
                                        .px_1p5()
                                        .py_0p5()
                                        .rounded(px(4.0))
                                        .bg(p.badge_muted_bg)
                                        .flex()
                                        .items_center()
                                        .gap_1()
                                        .child(
                                            render_icon(kind_icon)
                                                .size_3()
                                                .text_color(p.badge_muted_fg),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(p.badge_muted_fg)
                                                .child(kind_label),
                                        ),
                                ),
                        );
                    }
                    section = section.child(model_list);
                }

                card = card.child(section);
            }

            // OpenAI: show provider list + Add Provider button
            if provider.module_name == AI_OPENAI_MODULE_NAME {
                let openai_providers = self.openai_providers.clone();
                let has_providers = !openai_providers.is_empty();

                if has_providers {
                    let mut provider_list = div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .pt_2()
                        .border_t_1()
                        .border_color(p.panel_border);

                    for prov in &openai_providers {
                        let pid = prov.id.clone();
                        let pname = prov.name.clone();
                        let key_hint = if prov.api_key.is_empty() {
                            "No key".to_string()
                        } else {
                            let k = &prov.api_key;
                            if k.len() > 4 {
                                format!("{}…{}", "•".repeat(4), &k[k.len() - 4..])
                            } else {
                                "•".repeat(k.len())
                            }
                        };
                        let base = prov.base_url.clone();
                        let model_count = prov.models.len();
                        provider_list = provider_list.child(
                            div()
                                .px_2()
                                .py_1p5()
                                .rounded(px(radius.min(6.0)))
                                .flex()
                                .items_center()
                                .justify_between()
                                .cursor_pointer()
                                .hover(move |s| s.bg(p.surface_elevated))
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
                                                .child(pname),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(p.content_meta)
                                                .child(format!("{base}  ·  {key_hint}")),
                                        ),
                                )
                                .child(
                                    div()
                                        .px_1p5()
                                        .py_0p5()
                                        .rounded(px(4.0))
                                        .bg(p.badge_muted_bg)
                                        .text_xs()
                                        .text_color(p.badge_muted_fg)
                                        .child(format!("{model_count} model(s)")),
                                )
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |this, _, _, cx| {
                                        this.active_openai_provider_id = Some(pid.clone());
                                        this.openai_provider_edit_draft = None;
                                        this.openai_provider_delete_confirm = false;
                                        cx.notify();
                                    }),
                                ),
                        );
                    }

                    card = card.child(provider_list);
                }

                let add_btn = div()
                    .px_3()
                    .py_1()
                    .rounded(px(radius.min(8.0)))
                    .bg(p.surface_elevated)
                    .border_1()
                    .border_color(p.border)
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(p.content_title)
                    .cursor_pointer()
                    .child("Add Provider")
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.openai_create_draft = Some(crate::gui::app::OpenAiProviderDraft {
                                name: String::new(),
                                api_key: String::new(),
                                base_url: "https://api.openai.com".to_string(),
                                error: None,
                            });
                            this.active_openai_provider_id = None;
                            cx.notify();
                        }),
                    );

                card = card.child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .pt_2()
                        .border_t_1()
                        .border_color(p.panel_border)
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(p.ui_subtext)
                                .child(if has_providers {
                                    format!("{} provider(s)", openai_providers.len())
                                } else {
                                    "No providers configured".to_string()
                                }),
                        )
                        .child(add_btn),
                );
            }

            root = root.child(card);
        }

        // Insert CLI card in alphabetical position among provider cards.
        if !cli_providers.is_empty() && !cli_card_inserted {
            root = root.child(self.build_cli_card_element(
                &cli_providers,
                &active_module,
                &p,
                radius,
                is_dark,
                cx,
            ));
        }

        root.into_any_element()
    }

    fn build_cli_card_element(
        &mut self,
        cli_providers: &[&arcadia_core::modules::ai::AiProviderManifest],
        active_module: &str,
        p: &crate::gui::theme::palette::ThemePalette,
        radius: f32,
        is_dark: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let any_cli_active =
            is_cli_provider(active_module) && self.active_llama_cpp_model_id.is_none();
        let cli_card_pal = if any_cli_active {
            theme::nav_accent_palette(provider_accent(active_module), is_dark)
        } else {
            theme::nav_accent_palette("violet", is_dark)
        };
        let card_border = cli_card_pal.icon_idle;

        let mut cli_rows = div()
            .flex()
            .flex_col()
            .gap_1()
            .pt_2()
            .border_t_1()
            .border_color(p.panel_border);

        for cli_provider in cli_providers {
            let mn = cli_provider.module_name.to_string();
            let binary = cli_binary_for_module(cli_provider.module_name).unwrap_or("");
            let name = cli_display_name(cli_provider.module_name);
            let detected = self
                .detected_cli_providers
                .iter()
                .find(|c| c.binary == binary)
                .cloned();
            let is_row_active = active_module == cli_provider.module_name;
            let row_pal =
                theme::nav_accent_palette(provider_accent(cli_provider.module_name), is_dark);
            let row_bg = if is_row_active {
                row_pal.row_selected
            } else {
                p.panel_bg
            };
            let row_hover = row_pal.row_hover;
            let name_col = if is_row_active {
                row_pal.icon_active
            } else {
                p.content_title
            };
            let icon_col = row_pal.icon_active;
            let glyph = provider_glyph(cli_provider.module_name);
            let mn_click = mn.clone();

            let row = if let Some(ref cli) = detected {
                let version = cli.version.clone();
                div()
                    .px_2()
                    .py_1p5()
                    .rounded(px(radius.min(6.0)))
                    .bg(row_bg)
                    .hover(move |s| s.bg(row_hover))
                    .flex()
                    .items_center()
                    .justify_between()
                    .cursor_pointer()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.active_ai_provider_module = mn_click.clone();
                            this.ai_chat_model_id = None;
                            cx.notify();
                        }),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(render_icon(glyph).size_4().text_color(icon_col))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(name_col)
                                    .child(name),
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
                            .child(version),
                    )
                    .into_any_element()
            } else {
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
                            .items_center()
                            .gap_2()
                            .child(render_icon(glyph).size_4().text_color(p.ui_subtext))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(p.ui_subtext)
                                    .child(name),
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
                            .child("Not installed"),
                    )
                    .into_any_element()
            };

            cli_rows = cli_rows.child(row);
        }

        div()
            .rounded(px(radius.min(12.0)))
            .border_1()
            .border_color(card_border)
            .bg(p.panel_bg)
            .p_4()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .text_base()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(p.content_title)
                    .child("CLI Providers"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(p.content_meta)
                    .child("Subscription-based CLI tools. No API key required."),
            )
            .child(cli_rows)
            .into_any_element()
    }

    fn openai_provider_detail(
        &mut self,
        _window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
        provider: arcadia_core::config::openai::OpenAiProvider,
    ) -> impl IntoElement {
        let p = theme::theme_palette(cx, is_dark);
        let g_snap = theme::glyph_snapshot(cx);
        let is_glyph = g_snap.is_some();
        let radius = g_snap.map(|g| g.border_radius).unwrap_or(p.radius_md);
        let pal = theme::nav_accent_palette("emerald", is_dark);

        let is_delete_confirm = self.openai_provider_delete_confirm;
        let delete_bg = if is_delete_confirm { p.danger } else { p.surface_elevated };
        let delete_fg = if is_delete_confirm { p.on_accent } else { p.danger };
        let delete_border = if is_delete_confirm { p.danger } else { p.border };

        let key_display = if provider.api_key.is_empty() {
            "Not set".to_string()
        } else {
            let k = &provider.api_key;
            if k.len() > 4 {
                format!("{}…{}", "•".repeat(8), &k[k.len() - 4..])
            } else {
                "•".repeat(k.len())
            }
        };

        let prov_for_edit = provider.clone();
        let mut detail = div()
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
                            .child(provider.name.clone()),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(p.content_meta)
                            .child("OpenAI-compatible provider"),
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
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_0p5()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(p.ui_subtext)
                                    .child("BASE URL"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(p.content_title)
                                    .child(if provider.base_url.is_empty() {
                                        "https://api.openai.com".to_string()
                                    } else {
                                        provider.base_url.clone()
                                    }),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_0p5()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(p.ui_subtext)
                                    .child("API KEY"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(if provider.api_key.is_empty() {
                                        p.ui_subtext
                                    } else {
                                        p.content_title
                                    })
                                    .child(key_display),
                            ),
                    ),
            );

        if !provider.models.is_empty() {
            let mut model_list = div()
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
                        .child(format!("MODELS ({})", provider.models.len())),
                );
            for model in &provider.models {
                let name = model.name.clone();
                let model_id_label = model.model_id.clone();
                let kind_label = model.model_kind.label();
                let kind_icon = model.model_kind.icon_key();
                model_list = model_list.child(
                    div()
                        .px_1()
                        .py_1()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_0p5()
                                .child(div().text_sm().text_color(p.content_title).child(name))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(p.content_meta)
                                        .child(model_id_label),
                                ),
                        )
                        .child(
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded(px(4.0))
                                .bg(p.badge_muted_bg)
                                .flex()
                                .items_center()
                                .gap_1()
                                .child(
                                    render_icon(kind_icon)
                                        .size_3()
                                        .text_color(p.badge_muted_fg),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(p.badge_muted_fg)
                                        .child(kind_label),
                                ),
                        ),
                );
            }
            detail = detail.child(model_list);
        }

        detail = detail.child(
            div()
                .flex()
                .gap_2()
                .child(
                    div()
                        .px_3()
                        .py_1p5()
                        .rounded(px(radius.min(8.0)))
                        .bg(p.surface_elevated)
                        .border_1()
                        .border_color(p.border)
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(p.content_title)
                        .cursor_pointer()
                        .hover(move |s| s.border_color(pal.row_hover))
                        .child("Edit")
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, _, cx| {
                                this.openai_provider_edit_draft =
                                    Some(crate::gui::app::OpenAiProviderDraft {
                                        name: prov_for_edit.name.clone(),
                                        api_key: prov_for_edit.api_key.clone(),
                                        base_url: prov_for_edit.base_url.clone(),
                                        error: None,
                                    });
                                this.openai_provider_delete_confirm = false;
                                cx.notify();
                            }),
                        ),
                )
                .child(
                    div()
                        .px_3()
                        .py_1p5()
                        .rounded(px(radius.min(8.0)))
                        .bg(delete_bg)
                        .border_1()
                        .border_color(delete_border)
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(delete_fg)
                        .cursor_pointer()
                        .child(if is_delete_confirm { "Confirm Delete" } else { "Delete" })
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                if this.openai_provider_delete_confirm {
                                    this.openai_provider_delete(cx);
                                } else {
                                    this.openai_provider_delete_confirm = true;
                                    cx.notify();
                                }
                            }),
                        ),
                ),
        );

        detail
    }

    fn openai_provider_edit_form(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
        provider: arcadia_core::config::openai::OpenAiProvider,
    ) -> impl IntoElement {
        let _ = provider;
        self.openai_provider_form_inner(window, cx, is_dark, false)
    }

    fn openai_provider_create_form(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
        _draft: crate::gui::app::OpenAiProviderDraft,
    ) -> impl IntoElement {
        self.openai_provider_form_inner(window, cx, is_dark, true)
    }

    fn openai_provider_form_inner(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
        is_create: bool,
    ) -> impl IntoElement {
        let p = theme::theme_palette(cx, is_dark);
        let g_snap = theme::glyph_snapshot(cx);
        let is_glyph = g_snap.is_some();
        let radius = g_snap.map(|g| g.border_radius).unwrap_or(p.radius_md);
        let blink = self.text_caret_blink_visible;

        let (draft_ref, name_fh, api_key_fh, base_url_fh) = if is_create {
            (
                self.openai_create_draft.as_ref(),
                self.openai_create_name_focus.clone(),
                self.openai_create_api_key_focus.clone(),
                self.openai_create_base_url_focus.clone(),
            )
        } else {
            (
                self.openai_provider_edit_draft.as_ref(),
                self.openai_edit_name_focus.clone(),
                self.openai_edit_api_key_focus.clone(),
                self.openai_edit_base_url_focus.clone(),
            )
        };

        let name_val = draft_ref.map(|d| d.name.clone()).unwrap_or_default();
        let api_key_val = draft_ref.map(|d| d.api_key.clone()).unwrap_or_default();
        let base_url_val = draft_ref
            .map(|d| d.base_url.clone())
            .unwrap_or_else(|| "https://api.openai.com".to_string());
        let error_msg = draft_ref.and_then(|d| d.error.clone());

        let name_focused = name_fh.is_focused(window);
        let api_key_focused = api_key_fh.is_focused(window);
        let base_url_focused = base_url_fh.is_focused(window);

        let api_key_display = if api_key_val.is_empty() {
            text_with_trailing_caret("", api_key_focused, blink)
        } else {
            text_with_trailing_caret(&api_key_val, api_key_focused, blink)
        };

        let title = if is_create { "Add Provider" } else { "Edit Provider" };
        let save_label = if is_create { "Create" } else { "Save" };

        let name_fh2 = name_fh.clone();
        let api_key_fh2 = api_key_fh.clone();
        let base_url_fh2 = base_url_fh.clone();

        let mut form = div()
            .w_full()
            .when(is_glyph, |d| d.max_w(px(GLYPH_PANEL_CONTENT_MAX_W_PX)))
            .flex()
            .flex_col()
            .gap_6()
            .child(
                div()
                    .text_2xl()
                    .font_weight(FontWeight::BOLD)
                    .text_color(p.content_title)
                    .child(title),
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
                    .gap_3()
                    // Name field
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
                                    .child("NAME"),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .px_3()
                                    .py_2()
                                    .rounded(px(radius.min(8.0)))
                                    .bg(p.surface_elevated)
                                    .border_1()
                                    .border_color(if name_focused { p.accent } else { p.border })
                                    .text_sm()
                                    .text_color(p.content_title)
                                    .track_focus(&name_fh2)
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |this, _, window, cx| {
                                            if is_create {
                                                if let Some(ref mut d) = this.openai_create_draft {
                                                    let _ = d;
                                                }
                                            } else if let Some(ref mut d) =
                                                this.openai_provider_edit_draft
                                            {
                                                let _ = d;
                                            }
                                            name_fh.focus(window);
                                            cx.notify();
                                        }),
                                    )
                                    .on_key_down(cx.listener(move |this, event: &KeyDownEvent, _, cx| {
                                        let draft = if is_create {
                                            this.openai_create_draft.as_mut()
                                        } else {
                                            this.openai_provider_edit_draft.as_mut()
                                        };
                                        let Some(d) = draft else { return };
                                        let key = event.keystroke.key.as_str();
                                        let mods = event.keystroke.modifiers;
                                        if key == "escape" {
                                            if is_create { this.openai_create_draft = None; }
                                            else { this.openai_provider_edit_draft = None; }
                                        } else if key == "backspace" {
                                            d.name.pop();
                                        } else if !mods.control && !mods.alt && !mods.platform && !mods.function {
                                            if let Some(kc) = &event.keystroke.key_char {
                                                d.name.push_str(kc);
                                            }
                                        }
                                        cx.notify();
                                    }))
                                    .child(text_with_trailing_caret(&name_val, name_focused, blink)),
                            ),
                    )
                    // API Key field
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
                                    .flex_1()
                                    .px_3()
                                    .py_2()
                                    .rounded(px(radius.min(8.0)))
                                    .bg(p.surface_elevated)
                                    .border_1()
                                    .border_color(if api_key_focused { p.accent } else { p.border })
                                    .text_sm()
                                    .text_color(p.content_title)
                                    .track_focus(&api_key_fh2)
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |_, _, window, cx| {
                                            api_key_fh.focus(window);
                                            cx.notify();
                                        }),
                                    )
                                    .on_key_down(cx.listener(move |this, event: &KeyDownEvent, _, cx| {
                                        let draft = if is_create {
                                            this.openai_create_draft.as_mut()
                                        } else {
                                            this.openai_provider_edit_draft.as_mut()
                                        };
                                        let Some(d) = draft else { return };
                                        let key = event.keystroke.key.as_str();
                                        let mods = event.keystroke.modifiers;
                                        if key == "escape" {
                                            if is_create { this.openai_create_draft = None; }
                                            else { this.openai_provider_edit_draft = None; }
                                        } else if key == "backspace" {
                                            d.api_key.pop();
                                        } else if !mods.control && !mods.alt && !mods.platform && !mods.function {
                                            if let Some(kc) = &event.keystroke.key_char {
                                                d.api_key.push_str(kc);
                                            }
                                        }
                                        cx.notify();
                                    }))
                                    .child(api_key_display),
                            ),
                    )
                    // Base URL field
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
                                    .flex_1()
                                    .px_3()
                                    .py_2()
                                    .rounded(px(radius.min(8.0)))
                                    .bg(p.surface_elevated)
                                    .border_1()
                                    .border_color(if base_url_focused { p.accent } else { p.border })
                                    .text_sm()
                                    .text_color(p.content_title)
                                    .track_focus(&base_url_fh2)
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |_, _, window, cx| {
                                            base_url_fh.focus(window);
                                            cx.notify();
                                        }),
                                    )
                                    .on_key_down(cx.listener(move |this, event: &KeyDownEvent, _, cx| {
                                        let draft = if is_create {
                                            this.openai_create_draft.as_mut()
                                        } else {
                                            this.openai_provider_edit_draft.as_mut()
                                        };
                                        let Some(d) = draft else { return };
                                        let key = event.keystroke.key.as_str();
                                        let mods = event.keystroke.modifiers;
                                        if key == "escape" {
                                            if is_create { this.openai_create_draft = None; }
                                            else { this.openai_provider_edit_draft = None; }
                                        } else if key == "enter" {
                                            if is_create { this.openai_provider_save_create(cx); }
                                            else { this.openai_provider_save_edit(cx); }
                                            return;
                                        } else if key == "backspace" {
                                            d.base_url.pop();
                                        } else if !mods.control && !mods.alt && !mods.platform && !mods.function {
                                            if let Some(kc) = &event.keystroke.key_char {
                                                d.base_url.push_str(kc);
                                            }
                                        }
                                        cx.notify();
                                    }))
                                    .child(text_with_trailing_caret(
                                        &base_url_val,
                                        base_url_focused,
                                        blink,
                                    )),
                            ),
                    ),
            );

        if let Some(err) = error_msg {
            form = form.child(
                div()
                    .text_sm()
                    .text_color(p.danger)
                    .child(err),
            );
        }

        form = form.child(
            div()
                .flex()
                .gap_2()
                .child(
                    div()
                        .px_3()
                        .py_1p5()
                        .rounded(px(radius.min(8.0)))
                        .bg(p.accent)
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(p.on_accent)
                        .cursor_pointer()
                        .child(save_label)
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, _, cx| {
                                if is_create {
                                    this.openai_provider_save_create(cx);
                                } else {
                                    this.openai_provider_save_edit(cx);
                                }
                            }),
                        ),
                )
                .child(
                    div()
                        .px_3()
                        .py_1p5()
                        .rounded(px(radius.min(8.0)))
                        .bg(p.surface_elevated)
                        .border_1()
                        .border_color(p.border)
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(p.content_title)
                        .cursor_pointer()
                        .child("Cancel")
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, _, cx| {
                                if is_create {
                                    this.openai_create_draft = None;
                                } else {
                                    this.openai_provider_edit_draft = None;
                                }
                                cx.notify();
                            }),
                        ),
                ),
        );

        form
    }
}
