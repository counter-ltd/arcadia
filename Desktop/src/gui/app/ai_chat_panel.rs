use arcadia_core::config::ai_rules::{all_rules, AiRulesConfig};
use arcadia_core::config::ai_skills::{all_skills, AiSkillsConfig};
use arcadia_core::config::ollama::OllamaConfig;
use arcadia_core::config::ConfigFile;
use arcadia_core::config::modules::{
    AI_APFEL_MODULE_NAME, AI_EXEC_AIDER_MODULE_NAME, AI_EXEC_CLAUDE_MODULE_NAME,
    AI_EXEC_CODEX_MODULE_NAME, AI_EXEC_GEMINI_MODULE_NAME, AI_LLAMA_CPP_MODULE_NAME,
    AI_OLLAMA_MODULE_NAME, AI_OPENAI_MODULE_NAME, WORKSPACE_MODULE_NAME,
};
use arcadia_core::modules::ai_exec_cli::cli_for_module;
use arcadia_core::modules::ai::is_ai_provider_available;
use arcadia_core::modules::ai_types::{AiWorkspaceContext, TextGenerationRequest};
use openframe::{
    div, px, Context, FontWeight, InteractiveElement, IntoElement, KeyDownEvent,
    MouseButton, ParentElement, StatefulInteractiveElement, Styled, Window,
};
use openframe::prelude::FluentBuilder as _;

use crate::gui::app::ai_runtime::{AiRuntimeHandle, AiRuntimeRequest, ProviderRouting};
use crate::gui::app::text_input_caret::text_with_trailing_caret;
use crate::gui::app::{AiChat, AiMessage, AiMessageRole, ArcadiaRoot};
use crate::gui::theme;

impl ArcadiaRoot {
    pub(crate) fn ai_chat_panel(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> impl IntoElement {
        if self.ai_chats.is_empty() {
            return self.ai_empty_state_panel(cx, is_dark).into_any_element();
        }

        if !is_ai_provider_available(&self.module_rows) {
            return self.ai_no_provider_panel(cx, is_dark).into_any_element();
        }

        self.ai_active_chat_panel(window, cx, is_dark).into_any_element()
    }

    fn ai_empty_state_panel(
        &self,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> impl IntoElement {
        let p = theme::theme_palette(cx, is_dark);
        div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_3()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(p.content_title)
                    .child("No chats"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(p.content_meta)
                    .child("Right-click Chat in the sidebar to start a new conversation."),
            )
    }

    fn ai_no_provider_panel(
        &self,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> impl IntoElement {
        let p = theme::theme_palette(cx, is_dark);
        let g_snap = theme::glyph_snapshot(cx);
        let radius = g_snap.map(|g| g.border_radius).unwrap_or(p.radius_md);
        div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_4()
            .child(
                div()
                    .w(px(320.))
                    .p_5()
                    .rounded(px(radius.min(12.0)))
                    .border_1()
                    .border_color(p.panel_border)
                    .bg(p.panel_bg)
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        div()
                            .text_base()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(p.content_title)
                            .child("No AI provider configured"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(p.content_meta)
                            .child("Enable an AI provider module to start chatting."),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1p5()
                            .rounded(px(radius.min(8.0)))
                            .bg(theme::ui_accent(cx))
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme::ui_accent_fg(cx))
                            .cursor_pointer()
                            .child("Open Modules")
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _, cx| {
                                    this.active_page_id = "global.modules".to_string();
                                    cx.notify();
                                }),
                            ),
                    ),
            )
    }

    fn ai_active_chat_panel(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> impl IntoElement {
        let p = theme::theme_palette(cx, is_dark);
        let g_snap = theme::glyph_snapshot(cx);
        let radius = g_snap.map(|g| g.border_radius).unwrap_or(p.radius_md);
        let blink = self.text_caret_blink_visible;
        let is_focused = self.ai_input_focus.is_focused(window);
        let active_id = self.active_ai_chat_id;

        let chat = self.ai_chats.iter().find(|c| c.id == active_id);
        let draft = chat.map(|c| c.input_draft.clone()).unwrap_or_default();
        let messages = chat.map(|c| c.messages.clone()).unwrap_or_default();
        let is_loading = chat.map(|c| c.is_loading).unwrap_or(false);

        let border_col = if is_focused {
            theme::ui_accent(cx)
        } else {
            p.panel_border
        };

        // Message list
        let mut msg_col = div()
            .flex()
            .flex_col()
            .gap_4()
            .w_full();

        if messages.is_empty() && !is_loading {
            msg_col = msg_col.child(
                div()
                    .text_sm()
                    .text_color(p.content_meta)
                    .child("Send a message to start the conversation."),
            );
        }

        for msg in &messages {
            let is_user = msg.role == AiMessageRole::User;
            let bubble_bg = if is_user {
                theme::ui_accent(cx)
            } else {
                p.panel_bg
            };
            let text_col = if is_user { theme::ui_accent_fg(cx) } else { p.content_body };
            let content = msg.content.clone();

            let bubble = div()
                .px_3()
                .py_2()
                .rounded(px(radius.min(12.0)))
                .bg(bubble_bg)
                .border_1()
                .border_color(if is_user { bubble_bg } else { p.panel_border })
                .text_sm()
                .text_color(text_col)
                .when(is_user, |d| d.max_w(px(540.)))
                .when(!is_user, |d| d.w_full())
                .child(content);

            msg_col = msg_col.child(
                div()
                    .w_full()
                    .flex()
                    .flex_row()
                    .when(is_user, |d| d.justify_end())
                    .child(bubble),
            );
        }

        // Loading indicator: distinguish model-load phase (empty assistant placeholder)
        // from token-streaming phase (assistant message has content).
        let waiting_for_first_token = is_loading && messages.last()
            .map(|m| m.role == AiMessageRole::Assistant && m.content.is_empty())
            .unwrap_or(false);
        if waiting_for_first_token {
            let dim = p.content_meta;
            msg_col = msg_col.child(
                div()
                    .flex()
                    .flex_row()
                    .child(
                        div()
                            .w_full()
                            .px_3()
                            .py_2()
                            .rounded(px(radius.min(12.0)))
                            .bg(p.panel_bg)
                            .border_1()
                            .border_color(p.panel_border)
                            .text_sm()
                            .text_color(dim)
                            .child("Loading model…"),
                    ),
            );
        } else if is_loading {
            let dim = p.content_meta;
            msg_col = msg_col.child(
                div()
                    .flex()
                    .flex_row()
                    .child(
                        div()
                            .w_full()
                            .px_3()
                            .py_2()
                            .rounded(px(radius.min(12.0)))
                            .bg(p.panel_bg)
                            .border_1()
                            .border_color(p.panel_border)
                            .text_sm()
                            .text_color(dim)
                            .child("…"),
                    ),
            );
        }

        // Input area
        let input_area = div()
            .flex_shrink_0()
            .w_full()
            .px_4()
            .py_3()
            .border_t_1()
            .border_color(p.panel_border)
            .child(
                div()
                    .w_full()
                    .min_h(px(36.))
                    .px_3()
                    .py_2()
                    .rounded(px(radius.min(8.0)))
                    .border_1()
                    .border_color(border_col)
                    .bg(p.panel_bg)
                    .text_sm()
                    .text_color(p.content_body)
                    .cursor_text()
                    .track_focus(&self.ai_input_focus)
                    .on_mouse_down(MouseButton::Left, cx.listener(|_, _, window, _| {
                        let _ = window;
                    }))
                    .on_key_down(cx.listener(move |this, ev: &KeyDownEvent, window, cx| {
                        let key = &ev.keystroke.key;
                        if key == "escape" {
                            return;
                        }
                        if key == "enter" && !ev.keystroke.modifiers.shift {
                            this.ai_send_message(window, cx);
                            return;
                        }
                        let chat = this.ai_chats.iter_mut().find(|c| c.id == active_id);
                        let Some(chat) = chat else { return };
                        if key == "backspace" {
                            chat.input_draft.pop();
                        } else if !ev.keystroke.modifiers.platform
                            && !ev.keystroke.modifiers.control
                            && !ev.keystroke.modifiers.alt
                            && !ev.keystroke.modifiers.function
                        {
                            if let Some(kc) = &ev.keystroke.key_char {
                                chat.input_draft.push_str(kc);
                            }
                        }
                        cx.notify();
                    }))
                    .child(if draft.is_empty() && !is_focused {
                        div()
                            .text_color(p.content_meta)
                            .child("Message…")
                    } else {
                        div().child(text_with_trailing_caret(&draft, is_focused, blink))
                    }),
            );

        self.ensure_text_caret_blink_task(window, cx);

        let strip = self.ai_rules_skills_strip(cx, is_dark);

        let main_col = div()
            .flex_1()
            .min_w_0()
            .flex()
            .flex_col()
            .child(
                div()
                    .id("ai-chat-messages")
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .overflow_y_scroll()
                    .flex()
                    .flex_col()
                    .p_4()
                    .child(msg_col),
            )
            .child(strip)
            .child(input_area);

        div()
            .size_full()
            .flex()
            .flex_row()
            .child(main_col)
    }

    fn ai_rules_skills_strip(
        &mut self,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> impl IntoElement {
        let p = theme::theme_palette(cx, is_dark);
        let g_snap = theme::glyph_snapshot(cx);
        let radius = g_snap.map(|g| g.border_radius).unwrap_or(p.radius_md);
        let accent = theme::ui_accent(cx);
        let accent_fg = theme::ui_accent_fg(cx);

        let active_rules = self.ai_active_rule_ids.clone();
        let active_skills = self.ai_active_skill_ids.clone();
        let rule_picker_open = self.ai_rule_picker_open;
        let skill_picker_open = self.ai_skill_picker_open;

        // Load available rules + skills (ignore errors — degrade to empty).
        let rules_cfg = AiRulesConfig::load_or_create().unwrap_or_default();
        let skills_cfg = AiSkillsConfig::load_or_create().unwrap_or_default();
        let all_r = all_rules(&rules_cfg);
        let all_s = all_skills(&skills_cfg);

        let mut row = div()
            .w_full()
            .flex()
            .flex_row()
            .flex_wrap()
            .gap_1p5()
            .px_4()
            .py_2()
            .border_t_1()
            .border_color(p.panel_border);

        // Active rule chips
        for id in &active_rules {
            if let Some(rule) = all_r.iter().find(|r| &r.id == id) {
                let rule_id = id.clone();
                let chip = div()
                    .px_2()
                    .py_0p5()
                    .rounded(px(radius.min(6.0)))
                    .bg(accent)
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(accent_fg)
                    .cursor_pointer()
                    .child(format!("R: {}", rule.name))
                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                        this.ai_active_rule_ids.retain(|r| r != &rule_id);
                        cx.notify();
                    }));
                row = row.child(chip);
            }
        }

        // Active skill chips
        for id in &active_skills {
            if let Some(skill) = all_s.iter().find(|s| &s.id == id) {
                let skill_id = id.clone();
                let chip = div()
                    .px_2()
                    .py_0p5()
                    .rounded(px(radius.min(6.0)))
                    .bg(p.panel_bg)
                    .border_1()
                    .border_color(accent)
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(accent)
                    .cursor_pointer()
                    .child(format!("S: {}", skill.name))
                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                        this.ai_active_skill_ids.retain(|s| s != &skill_id);
                        cx.notify();
                    }));
                row = row.child(chip);
            }
        }

        // "+ Rule" toggle button
        let rule_btn_bg = if rule_picker_open { accent } else { p.panel_bg };
        let rule_btn_fg = if rule_picker_open { accent_fg } else { p.content_meta };
        row = row.child(
            div()
                .px_2()
                .py_0p5()
                .rounded(px(radius.min(6.0)))
                .bg(rule_btn_bg)
                .border_1()
                .border_color(p.panel_border)
                .text_xs()
                .text_color(rule_btn_fg)
                .cursor_pointer()
                .child("+ Rule")
                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                    this.ai_rule_picker_open = !this.ai_rule_picker_open;
                    this.ai_skill_picker_open = false;
                    cx.notify();
                })),
        );

        // "+ Skill" toggle button
        let skill_btn_bg = if skill_picker_open { accent } else { p.panel_bg };
        let skill_btn_fg = if skill_picker_open { accent_fg } else { p.content_meta };
        row = row.child(
            div()
                .px_2()
                .py_0p5()
                .rounded(px(radius.min(6.0)))
                .bg(skill_btn_bg)
                .border_1()
                .border_color(p.panel_border)
                .text_xs()
                .text_color(skill_btn_fg)
                .cursor_pointer()
                .child("+ Skill")
                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                    this.ai_skill_picker_open = !this.ai_skill_picker_open;
                    this.ai_rule_picker_open = false;
                    cx.notify();
                })),
        );

        // "Stage Writes" toggle chip
        let stage_writes = self.ai_stage_writes;
        let (sw_bg, sw_fg) = if stage_writes {
            (accent, accent_fg)
        } else {
            (p.panel_bg, p.content_meta)
        };
        row = row.child(
            div()
                .px_2()
                .py_0p5()
                .rounded(px(radius.min(6.0)))
                .bg(sw_bg)
                .border_1()
                .border_color(if stage_writes { accent } else { p.panel_border })
                .text_xs()
                .text_color(sw_fg)
                .cursor_pointer()
                .child(if stage_writes { "Stage On" } else { "Stage Off" })
                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                    this.ai_stage_writes = !this.ai_stage_writes;
                    cx.notify();
                })),
        );

        // Picker panels (inline dropdown below the strip)
        let picker = if rule_picker_open {
            let mut picker_col = div()
                .w_full()
                .px_4()
                .pb_2()
                .flex()
                .flex_col()
                .gap_1();
            for rule in &all_r {
                let is_active = active_rules.contains(&rule.id);
                let rule_id = rule.id.clone();
                let item_bg = if is_active { accent } else { p.panel_bg };
                let item_fg = if is_active { accent_fg } else { p.content_body };
                picker_col = picker_col.child(
                    div()
                        .w_full()
                        .px_3()
                        .py_1p5()
                        .rounded(px(radius.min(6.0)))
                        .bg(item_bg)
                        .border_1()
                        .border_color(p.panel_border)
                        .flex()
                        .flex_col()
                        .gap_0p5()
                        .cursor_pointer()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(item_fg)
                                .child(rule.name.clone()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(if is_active { accent_fg } else { p.content_meta })
                                .child(rule.system_fragment.chars().take(80).collect::<String>()),
                        )
                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                            if this.ai_active_rule_ids.contains(&rule_id) {
                                this.ai_active_rule_ids.retain(|r| r != &rule_id);
                            } else {
                                this.ai_active_rule_ids.push(rule_id.clone());
                            }
                            cx.notify();
                        })),
                );
            }
            Some(picker_col)
        } else if skill_picker_open {
            let mut picker_col = div()
                .w_full()
                .px_4()
                .pb_2()
                .flex()
                .flex_col()
                .gap_1();
            for skill in &all_s {
                let is_active = active_skills.contains(&skill.id);
                let skill_id = skill.id.clone();
                let item_bg = if is_active { accent } else { p.panel_bg };
                let item_fg = if is_active { accent_fg } else { p.content_body };
                picker_col = picker_col.child(
                    div()
                        .w_full()
                        .px_3()
                        .py_1p5()
                        .rounded(px(radius.min(6.0)))
                        .bg(item_bg)
                        .border_1()
                        .border_color(p.panel_border)
                        .flex()
                        .flex_col()
                        .gap_0p5()
                        .cursor_pointer()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(item_fg)
                                .child(skill.name.clone()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(if is_active { accent_fg } else { p.content_meta })
                                .child(skill.system_fragment.chars().take(80).collect::<String>()),
                        )
                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                            if this.ai_active_skill_ids.contains(&skill_id) {
                                this.ai_active_skill_ids.retain(|s| s != &skill_id);
                            } else {
                                this.ai_active_skill_ids.push(skill_id.clone());
                            }
                            cx.notify();
                        })),
                );
            }
            Some(picker_col)
        } else {
            None
        };

        div()
            .w_full()
            .flex()
            .flex_col()
            .child(row)
            .when_some(picker, |d, p| d.child(p))
    }

    pub fn ai_send_message(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let active_id = self.active_ai_chat_id;

        // Collect input
        let input = self
            .ai_chats
            .iter()
            .find(|c| c.id == active_id)
            .map(|c| c.input_draft.trim().to_string())
            .unwrap_or_default();

        if input.is_empty() {
            return;
        }

        // If already inferring on this chat, ignore.
        if self.ai_stream_chat_id == Some(active_id) {
            return;
        }

        // Build provider routing from active provider + selected model.
        let provider = self.active_ai_provider_module.clone();
        let model_id = self.ai_chat_model_id.clone();
        let routing_result: Result<ProviderRouting, String> = match provider.as_str() {
            AI_LLAMA_CPP_MODULE_NAME => {
                match model_id.as_deref() {
                    None => Err("No model selected. Choose a model from the top bar.".to_string()),
                    Some(id) => self.llama_cpp_models.iter().find(|m| m.id == id)
                        .map(|m| ProviderRouting::LlamaCpp { model_path: m.path.clone() })
                        .ok_or_else(|| format!("Model '{id}' not found. It may have been moved or deleted.")),
                }
            }
            AI_OLLAMA_MODULE_NAME => {
                let cfg = OllamaConfig::load_or_create().unwrap_or_default();
                match model_id.as_deref() {
                    None => Err("No model selected. Choose a model from the top bar.".to_string()),
                    Some(id) => self.ollama_models.iter().find(|m| m.id == id)
                        .map(|m| ProviderRouting::Ollama { endpoint: cfg.endpoint.clone(), model_name: m.model_tag.clone() })
                        .ok_or_else(|| format!("Model '{id}' not found in Ollama model list.")),
                }
            }
            AI_OPENAI_MODULE_NAME => {
                match model_id.as_deref() {
                    None => Err("No model selected. Choose a model from the top bar.".to_string()),
                    Some(id) => self.openai_models.iter().find(|m| m.id == id)
                        .map(|m| ProviderRouting::OpenAi { model_id: m.model_id.clone() })
                        .ok_or_else(|| format!("Model '{id}' not found.")),
                }
            }
            AI_EXEC_CLAUDE_MODULE_NAME
            | AI_EXEC_CODEX_MODULE_NAME
            | AI_EXEC_GEMINI_MODULE_NAME
            | AI_EXEC_AIDER_MODULE_NAME => {
                match cli_for_module(&provider) {
                    Some((binary, model_flag)) => Ok(ProviderRouting::ExecCli {
                        binary: binary.to_string(),
                        model_flag: model_flag.map(str::to_string),
                        model_value: model_id.as_deref().unwrap_or("").to_string(),
                    }),
                    None => Err(format!("Unknown CLI provider module: {provider}")),
                }
            }
            AI_APFEL_MODULE_NAME => Ok(ProviderRouting::Apfel),
            _ => Err("No AI provider enabled. Enable one in Modules.".to_string()),
        };

        // Resolve workspace context from cached entries (avoids a disk read per message).
        let workspace_context: Option<AiWorkspaceContext> =
            if self.is_module_enabled(WORKSPACE_MODULE_NAME) {
                self.ai_chat_workspace_id.as_deref().and_then(|ws_id| {
                    self.workspace_entries.iter()
                        .find(|w| w.id == ws_id)
                        .map(AiWorkspaceContext::from_workspace_entry)
                })
            } else {
                None
            };

        // Push user message + clear draft. Track provider/model for session persistence.
        if let Some(chat) = self.ai_chats.iter_mut().find(|c| c.id == active_id) {
            chat.messages.push(AiMessage {
                role: AiMessageRole::User,
                content: input.clone(),
            });
            chat.input_draft.clear();
            chat.session_provider = provider.clone();
            if let Some(ref mid) = model_id {
                chat.session_model_id = mid.clone();
            }
        }

        let routing = match routing_result {
            Ok(r) => r,
            Err(msg) => {
                if let Some(chat) = self.ai_chats.iter_mut().find(|c| c.id == active_id) {
                    chat.messages.push(AiMessage {
                        role: AiMessageRole::Assistant,
                        content: msg,
                    });
                }
                cx.notify();
                return;
            }
        };

        // Build full message history.
        let system = self.ai_default_system_prompt.clone();
        let history: Vec<(String, String)> = self
            .ai_chats
            .iter()
            .find(|c| c.id == active_id)
            .map(|c| {
                c.messages.iter().map(|m| {
                    let role = match m.role {
                        AiMessageRole::User => "user".to_string(),
                        AiMessageRole::Assistant => "assistant".to_string(),
                    };
                    (role, m.content.clone())
                }).collect()
            })
            .unwrap_or_default();

        // Placeholder assistant message (tokens stream into it).
        if let Some(chat) = self.ai_chats.iter_mut().find(|c| c.id == active_id) {
            chat.messages.push(AiMessage { role: AiMessageRole::Assistant, content: String::new() });
            chat.is_loading = true;
        }

        self.ai_stream_chat_id = Some(active_id);

        let tools = if workspace_context.is_some() {
            arcadia_core::modules::ai_tools::WORKSPACE_TOOLS.to_vec()
        } else {
            vec![]
        };

        let runtime = self.ai_runtime.get_or_insert_with(AiRuntimeHandle::start);
        let _ = runtime.request_tx.try_send(AiRuntimeRequest::Generate {
            routing,
            request: TextGenerationRequest {
                system,
                messages: history,
                max_tokens: 512,
                workspace_context,
                tools,
                active_rule_ids: self.ai_active_rule_ids.clone(),
                active_skill_ids: self.ai_active_skill_ids.clone(),
                stage_writes: self.ai_stage_writes,
            },
        });

        self.ensure_ai_poll_task(window, cx);
        cx.notify();
    }
}

