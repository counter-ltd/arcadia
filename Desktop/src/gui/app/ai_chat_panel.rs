use arcadia_core::config::ai_rules::{all_rules, AiRulesConfig};
use arcadia_core::config::ai_skills::{all_skills, AiSkillsConfig};
use arcadia_core::config::modules::{
    AI_APFEL_MODULE_NAME, AI_EXEC_AIDER_MODULE_NAME, AI_EXEC_CLAUDE_MODULE_NAME,
    AI_EXEC_CODEX_MODULE_NAME, AI_EXEC_GEMINI_MODULE_NAME, AI_IMAGE_PLAYGROUND_MODULE_NAME,
    AI_LLAMA_CPP_MODULE_NAME, AI_OLLAMA_MODULE_NAME, AI_OPENAI_MODULE_NAME, WORKSPACE_MODULE_NAME,
};
use arcadia_core::config::ollama::OllamaConfig;
use arcadia_core::config::ConfigFile;
use arcadia_core::modules::ai::is_ai_provider_available;
use arcadia_core::modules::ai_exec_cli::cli_for_module;
use arcadia_core::ai_types::{AiWorkspaceContext, ImageGenerationRequest, TextGenerationRequest};
use openframe::prelude::FluentBuilder as _;
use openframe::{
    div, img, px, rgb, text_input, AnyElement, Context, FontWeight, InteractiveElement, IntoElement,
    KeyDownEvent, MouseButton, ObjectFit, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, StyledImage, Window,
};

use crate::gui::app::ai_runtime::{
    AiRuntimeHandle, AiRuntimeRequest, ImageProviderRouting, ProviderRouting,
};
use crate::gui::app::{AiMessage, AiMessageRole, ArcadiaRoot};
use crate::gui::theme;

enum MsgSegment {
    Text(String),
    ToolCall { name: String, args: serde_json::Value },
    PendingTool,
}

fn parse_msg_segments(content: &str) -> Vec<MsgSegment> {
    let mut segments = Vec::new();
    let mut remaining = content;
    while let Some(block_start) = remaining.find("```json") {
        let before = remaining[..block_start].trim_end();
        if !before.is_empty() {
            segments.push(MsgSegment::Text(before.to_string()));
        }
        let after_marker = &remaining[block_start + 7..];
        if let Some(block_end) = after_marker.find("```") {
            let json_str = after_marker[..block_end].trim();
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
                if let Some(calls) = val["tool_calls"].as_array() {
                    for call in calls {
                        if let Some(name) = call["name"].as_str() {
                            segments.push(MsgSegment::ToolCall {
                                name: name.to_string(),
                                args: call["arguments"].clone(),
                            });
                        }
                    }
                }
            }
            remaining = &after_marker[block_end + 3..];
        } else {
            // Unclosed block — still streaming; suppress raw JSON, show pending pill.
            segments.push(MsgSegment::PendingTool);
            remaining = "";
            break;
        }
    }
    let tail = remaining.trim_start_matches('\n').trim_end();
    if !tail.is_empty() {
        segments.push(MsgSegment::Text(tail.to_string()));
    }
    if segments.is_empty() {
        segments.push(MsgSegment::Text(content.to_string()));
    }
    segments
}


impl ArcadiaRoot {
    pub(crate) fn ai_chat_panel(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> impl IntoElement {
        if self.ai.chats.is_empty() || self.ai.chat_show_dashboard {
            return self.ai_chat_dashboard(cx, is_dark).into_any_element();
        }

        if !is_ai_provider_available(&self.module_rows) {
            return self.ai_no_provider_panel(cx, is_dark).into_any_element();
        }

        self.ai_active_chat_panel(window, cx, is_dark)
            .into_any_element()
    }

    fn ai_chat_dashboard(&mut self, cx: &mut Context<Self>, is_dark: bool) -> impl IntoElement {
        let p = theme::theme_palette(cx, is_dark);
        let ws_pal = theme::nav_accent_palette("emerald", is_dark);
        let chat_pal = theme::nav_accent_palette("violet", is_dark);
        let r = p.radius_md.min(12.0);
        let msg_bg_preview = if is_dark {
            rgb(0x141820)
        } else {
            rgb(0xf0f2f5)
        };
        let divider_col = if is_dark {
            rgb(0x2a3340)
        } else {
            rgb(0xe6e8ef)
        };
        // Workspace cards — clicking opens a new chat in that workspace
        let workspace_cards: Vec<openframe::AnyElement> =
            arcadia_core::config::workspace::WorkspacesConfig::load_or_create()
                .map(|cfg| cfg.workspaces)
                .unwrap_or_default()
                .into_iter()
                .map(|ws| {
                    let ws_id = ws.id.clone();
                    let ws_label = if ws.label.is_empty() {
                        ws.path.rsplit('/').next().unwrap_or(&ws.path).to_string()
                    } else {
                        ws.label.clone()
                    };
                    let ws_path = ws.path.clone();
                    div()
                        .flex_1()
                        .min_w(px(220.))
                        .cursor_pointer()
                        .p_4()
                        .rounded(px(r))
                        .bg(p.panel_bg)
                        .border_1()
                        .border_color(p.panel_border)
                        .hover(move |s| s.bg(p.row_bg).border_color(ws_pal.row_hover))
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(
                            div()
                                .flex()
                                .gap_3()
                                .items_center()
                                .child(
                                    theme::render_icon("folder-open")
                                        .size_6()
                                        .text_color(ws_pal.icon_active),
                                )
                                .child(
                                    div()
                                        .text_base()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(p.content_title)
                                        .flex_1()
                                        .child(ws_label),
                                ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(p.content_meta)
                                .max_w(px(248.))
                                .child(ws_path),
                        )
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, _, cx| {
                                let id = this.ai.next_id;
                                this.ai.chats.push(crate::gui::app::AiChat {
                                    id,
                                    title: format!("Chat {id}"),
                                    messages: vec![],
                                    input_draft: String::new(),
                                    is_loading: false,
                                    session_id: None,
                                    session_provider: String::new(),
                                    session_model_id: String::new(),
                                    workspace_id: Some(ws_id.clone()),
                                });
                                this.ai.active_chat_id = id;
                                this.ai.next_id += 1;
                                this.ai.chat_workspace_id = Some(ws_id.clone());
                                this.ai.chat_show_dashboard = false;
                                this.active_page_id = "ai.chat".to_string();
                                this.sync_settings_hub_expanded_from_active_page();
                                cx.notify();
                            }),
                        )
                        .into_any_element()
                })
                .collect();

        // Open chat cards — rendered from ai_sessions so they persist across restarts.
        // If a session is currently open in ai_chats, use live messages for the preview.
        let chat_cards: Vec<openframe::AnyElement> = self
            .ai.sessions
            .iter()
            .map(|session| {
                let session_id = session.id.clone();
                let title = session.title.clone();
                let provider = session.provider.clone();
                let workspace_id_card = session.workspace_id.clone();
                let open_chat = self
                    .ai.chats
                    .iter()
                    .find(|c| c.session_id.as_deref() == Some(&session.id));
                let is_loading = open_chat.map(|c| c.is_loading).unwrap_or(false);
                let preview_msgs: Vec<(bool, String)> = if let Some(chat) = open_chat {
                    chat.messages
                        .iter()
                        .rev()
                        .take(4)
                        .map(|m| {
                            let is_user = matches!(m.role, crate::gui::app::AiMessageRole::User);
                            let text: String = m.content.chars().take(44).collect();
                            (is_user, text)
                        })
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .collect()
                } else {
                    session.last_messages.clone()
                };
                let provider_icon = if provider.is_empty() {
                    "ai-provider"
                } else {
                    arcadia_core::config::modules::MODULE_REGISTRY
                        .iter()
                        .find(|m| m.name == provider.as_str())
                        .map(|m| m.glyph)
                        .unwrap_or("ai-provider")
                };
                let workspace_label = workspace_id_card.as_deref().and_then(|ws_id| {
                    arcadia_core::config::workspace::WorkspacesConfig::load_or_create()
                        .ok()
                        .and_then(|cfg| cfg.workspaces.into_iter().find(|w| w.id == ws_id))
                        .map(|w| {
                            if w.label.is_empty() {
                                w.path.rsplit('/').next().unwrap_or(&w.path).to_string()
                            } else {
                                w.label.clone()
                            }
                        })
                });
                let provider_pal = theme::nav_accent_palette(
                    crate::gui::app::ai_models_panel::provider_accent(&provider),
                    is_dark,
                );
                let card_key = format!("chat-card-{}", session.id);
                let card_key_hover = card_key.clone();
                let ha = *self.item_hover_alphas.get(&card_key).unwrap_or(&0.0);
                let card_bg =
                    crate::gui::app::sidebar::nav_items::lerp_color(p.panel_bg, p.row_bg, ha);
                let card_border = crate::gui::app::sidebar::nav_items::lerp_color(
                    p.panel_border,
                    provider_pal.icon_idle,
                    ha,
                );
                let divider_col = crate::gui::app::sidebar::nav_items::lerp_color(
                    p.panel_border,
                    provider_pal.icon_idle,
                    ha,
                );
                div()
                    .id(SharedString::from(card_key.clone()))
                    .flex_1()
                    .min_w(px(220.))
                    .cursor_pointer()
                    .rounded(px(r))
                    .bg(card_bg)
                    .border_1()
                    .border_color(card_border)
                    .on_hover(cx.listener(move |this, hovered, _, cx| {
                        this.start_item_hover_anim(card_key_hover.clone(), *hovered);
                        cx.notify();
                    }))
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            // If already open, switch to it.
                            if let Some(chat) = this
                                .ai.chats
                                .iter()
                                .find(|c| c.session_id.as_deref() == Some(&session_id))
                            {
                                let chat_id = chat.id;
                                this.ai.active_chat_id = chat_id;
                                this.ai.chat_show_dashboard = false;
                                this.active_page_id = "ai.chat".to_string();
                                this.sync_settings_hub_expanded_from_active_page();
                                cx.notify();
                                return;
                            }
                            // Load from disk.
                            if let Ok(stored) =
                                arcadia_core::modules::ai_chat_store::load_session(&session_id)
                            {
                                let id = this.ai.next_id;
                                this.ai.next_id += 1;
                                let messages = stored
                                    .messages
                                    .iter()
                                    .map(|m| {
                                        let role = if m.role == "user" {
                                            crate::gui::app::AiMessageRole::User
                                        } else {
                                            crate::gui::app::AiMessageRole::Assistant
                                        };
                                        crate::gui::app::AiMessage {
                                            role,
                                            content: m.content.clone(),
                                            provider: stored.provider.clone(),
                                            images: Vec::new(),
                                        }
                                    })
                                    .collect();
                                this.ai.chats.push(crate::gui::app::AiChat {
                                    id,
                                    title: stored.title.clone(),
                                    messages,
                                    input_draft: String::new(),
                                    is_loading: false,
                                    session_id: Some(session_id.clone()),
                                    session_provider: stored.provider.clone(),
                                    session_model_id: stored.model_id.clone(),
                                    workspace_id: stored.workspace_id.clone(),
                                });
                                this.ai.active_chat_id = id;
                                this.ai.chat_show_dashboard = false;
                                this.active_page_id = "ai.chat".to_string();
                                this.sync_settings_hub_expanded_from_active_page();
                                this.ai.active_rule_ids = stored.active_rule_ids.clone();
                                this.ai.active_skill_ids = stored.active_skill_ids.clone();
                                this.ai.chat_workspace_id = stored.workspace_id.clone();
                                cx.notify();
                            }
                        }),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_h(px(100.))
                            .bg(msg_bg_preview)
                            .p_2()
                            .overflow_hidden()
                            .flex()
                            .flex_col()
                            .justify_center()
                            .gap_1()
                            .children(preview_msgs.into_iter().map(|(is_user, text)| {
                                let (bubble_bg, bubble_border, bubble_text) = if is_user {
                                    (p.panel_bg, p.panel_border, p.content_body)
                                } else {
                                    use crate::gui::app::sidebar::nav_items::lerp_color;
                                    (
                                        lerp_color(provider_pal.row_selected, provider_pal.icon_idle, 0.3),
                                        provider_pal.icon_idle,
                                        provider_pal.icon_active,
                                    )
                                };
                                div()
                                    .flex()
                                    .flex_row()
                                    .flex_shrink_0()
                                    .when(is_user, |d| d.justify_end())
                                    .when(!is_user, |d| d.justify_start())
                                    .child(
                                        div()
                                            .rounded(px(r.min(12.0)))
                                            .bg(bubble_bg)
                                            .border_1()
                                            .border_color(bubble_border)
                                            .px_2()
                                            .py(px(3.))
                                            .max_w(px(180.))
                                            .text_xs()
                                            .text_color(bubble_text)
                                            .child(if text.is_empty() {
                                                "…".to_string()
                                            } else {
                                                text
                                            }),
                                    )
                                    .into_any_element()
                            })),
                    )
                    .child({
                        use crate::gui::app::sidebar::nav_items::lerp_color;
                        let footer_bg = lerp_color(p.panel_bg, provider_pal.row_selected, ha);
                        let title_col = lerp_color(p.content_title, provider_pal.icon_active, ha);
                        let meta_col = lerp_color(p.content_meta, provider_pal.icon_active, ha);
                        let icon_col =
                            lerp_color(provider_pal.icon_idle, provider_pal.icon_active, ha);
                        div()
                            .p_3()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .bg(footer_bg)
                            .border_t_1()
                            .border_color(divider_col)
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .gap_1()
                                    .items_center()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(title_col)
                                            .flex_1()
                                            .child(title),
                                    )
                                    .when(is_loading, |d| {
                                        d.child(
                                            div()
                                                .w(px(6.))
                                                .h(px(6.))
                                                .rounded_full()
                                                .bg(chat_pal.icon_active)
                                                .into_any_element(),
                                        )
                                    }),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .gap_1()
                                    .items_center()
                                    .child(
                                        crate::gui::theme::render_icon(provider_icon)
                                            .size_3()
                                            .text_color(icon_col),
                                    )
                                    .child(
                                        div().text_xs().text_color(meta_col).child(
                                            workspace_label
                                                .unwrap_or_else(|| "No Workspace".to_string()),
                                        ),
                                    ),
                            )
                    })
                    .into_any_element()
            })
            .collect();

        let ws_phantoms: Vec<_> = (0..5)
            .map(|_| div().flex_1().min_w(px(220.)).into_any_element())
            .collect();
        let chat_phantoms: Vec<_> = (0..5)
            .map(|_| div().flex_1().min_w(px(220.)).into_any_element())
            .collect();

        let workspaces_body: openframe::AnyElement = if workspace_cards.is_empty() {
            div()
                .text_sm()
                .text_color(p.content_meta)
                .child("No workspaces registered. Add one in Settings → Workspaces.")
                .into_any_element()
        } else {
            div()
                .flex()
                .flex_row()
                .flex_wrap()
                .gap_4()
                .children(workspace_cards)
                .children(ws_phantoms)
                .into_any_element()
        };

        let chats_body: openframe::AnyElement = div()
            .flex()
            .flex_row()
            .flex_wrap()
            .gap_4()
            .children(chat_cards)
            .children(chat_phantoms)
            .into_any_element();

        div()
            .id("ai-chat-dashboard-scroll")
            .w_full()
            .h_full()
            .overflow_y_scroll()
            .bg(if is_dark {
                rgb(0x1a1f29)
            } else {
                rgb(0xfafafa)
            })
            .p_8()
            .flex()
            .flex_col()
            .gap_8()
            // Workspaces section
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .text_xl()
                            .font_weight(FontWeight::BOLD)
                            .text_color(p.content_title)
                            .child("Workspaces"),
                    )
                    .child(workspaces_body),
            )
            // Divider
            .child(div().w_full().h(px(1.)).bg(divider_col))
            // Open chats section
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .text_xl()
                            .font_weight(FontWeight::BOLD)
                            .text_color(p.content_title)
                            .child("Open Chats"),
                    )
                    .child(chats_body),
            )
    }

    fn ai_no_provider_panel(&self, cx: &mut Context<Self>, is_dark: bool) -> impl IntoElement {
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
                                    this.sync_settings_hub_expanded_from_active_page();
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
        let is_focused = self.ai.input_focus.is_focused(window);
        let active_id = self.ai.active_chat_id;

        let chat = self.ai.chats.iter().find(|c| c.id == active_id);
        let draft = chat.map(|c| c.input_draft.clone()).unwrap_or_default();
        let messages = chat.map(|c| c.messages.clone()).unwrap_or_default();
        let is_loading = chat.map(|c| c.is_loading).unwrap_or(false);

        let border_col = if is_focused {
            theme::ui_accent(cx)
        } else {
            p.panel_border
        };

        // Message list
        let mut msg_col = div().flex().flex_col().gap_4().w_full();

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
            let (bubble_bg, bubble_border, text_col) = if is_user {
                (p.panel_bg, p.panel_border, p.content_body)
            } else {
                use crate::gui::app::sidebar::nav_items::lerp_color;
                let pal = theme::nav_accent_palette(
                    crate::gui::app::ai_models_panel::provider_accent(&msg.provider),
                    is_dark,
                );
                (lerp_color(pal.row_selected, pal.icon_idle, 0.3), pal.icon_idle, pal.icon_active)
            };

            for segment in parse_msg_segments(&msg.content) {
                let (row_el, align_end): (AnyElement, bool) = match segment {
                    MsgSegment::Text(text) => {
                        let mut bubble = div()
                            .px_3()
                            .py_2()
                            .rounded(px(radius.min(12.0)))
                            .bg(bubble_bg)
                            .border_1()
                            .border_color(bubble_border)
                            .text_sm()
                            .text_color(text_col)
                            .max_w(px(540.))
                            .flex()
                            .flex_col()
                            .gap_0p5();
                        for line in text.lines() {
                            bubble = bubble.child(div().child(line.to_string()));
                        }
                        (bubble.into_any_element(), is_user)
                    }
                    MsgSegment::ToolCall { name, args } => {
                        let mut tool_bubble = div()
                            .px_3()
                            .py_2()
                            .rounded(px(radius.min(12.0)))
                            .bg(p.badge_muted_bg)
                            .border_1()
                            .border_color(p.panel_border)
                            .text_sm()
                            .max_w(px(360.))
                            .flex()
                            .flex_col()
                            .gap_0p5();
                        tool_bubble = tool_bubble.child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_1p5()
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(p.badge_muted_fg)
                                        .child("⚙"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(p.content_title)
                                        .child(name),
                                ),
                        );
                        if let Some(obj) = args.as_object() {
                            for (key, val) in obj {
                                let val_str = match val {
                                    serde_json::Value::String(s) => s.clone(),
                                    other => other.to_string(),
                                };
                                tool_bubble = tool_bubble.child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .gap_1()
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(p.content_meta)
                                                .child(format!("{}:", key)),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(p.badge_muted_fg)
                                                .child(val_str),
                                        ),
                                );
                            }
                        }
                        (tool_bubble.into_any_element(), false)
                    }
                    MsgSegment::PendingTool => {
                        let pill = div()
                            .px_3()
                            .py_2()
                            .rounded(px(radius.min(12.0)))
                            .bg(p.badge_muted_bg)
                            .border_1()
                            .border_color(p.panel_border)
                            .text_sm()
                            .text_color(p.badge_muted_fg)
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_1p5()
                            .child(div().text_xs().child("⚙"))
                            .child(div().text_xs().child("Calling tool…"));
                        (pill.into_any_element(), false)
                    }
                };

                msg_col = msg_col.child(
                    div()
                        .w_full()
                        .flex()
                        .flex_row()
                        .when(align_end, |d| d.justify_end())
                        .child(row_el),
                );
            }

            // Attached images (e.g. ImagePlayground output). Rendered after text segments
            // so a provider can stream a caption and then attach the generated PNG.
            for image_path in &msg.images {
                let img_bubble = div()
                    .p_1()
                    .rounded(px(radius.min(12.0)))
                    .bg(bubble_bg)
                    .border_1()
                    .border_color(bubble_border)
                    .flex()
                    .child(
                        img(image_path.clone())
                            .w(px(384.))
                            .h(px(384.))
                            .object_fit(ObjectFit::Contain),
                    );
                msg_col = msg_col.child(
                    div()
                        .w_full()
                        .flex()
                        .flex_row()
                        .when(is_user, |d| d.justify_end())
                        .child(img_bubble),
                );
            }
        }

        // Loading indicator: distinguish model-load phase (empty assistant placeholder)
        // from token-streaming phase (assistant message has content).
        let waiting_for_first_token = is_loading
            && messages
                .last()
                .map(|m| m.role == AiMessageRole::Assistant && m.content.is_empty())
                .unwrap_or(false);
        if waiting_for_first_token {
            let dim = p.content_meta;
            msg_col = msg_col.child(
                div().flex().flex_row().child(
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
                div().flex().flex_row().child(
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
        let weak = cx.weak_entity();
        let input_area = div()
            .flex_shrink_0()
            .w_full()
            .px_4()
            .py_3()
            .border_t_1()
            .border_color(p.panel_border)
            .child(
                text_input(
                    "ai-chat-input",
                    window,
                    weak,
                    &draft,
                    "Message…",
                    &self.ai.input_focus,
                    p.content_body,
                    p.content_meta,
                    move |this, new_text, cx| {
                        if let Some(chat) = this.ai.chats.iter_mut().find(|c| c.id == active_id) {
                            chat.input_draft = new_text;
                        }
                        cx.notify();
                    },
                )
                .w_full()
                .min_h(px(36.))
                .px_3()
                .py_2()
                .rounded(px(radius.min(8.0)))
                .border_1()
                .border_color(border_col)
                .bg(p.panel_bg)
                .text_sm()
                .cursor_text()
                .on_key_down(cx.listener(move |this, ev: &KeyDownEvent, window, cx| {
                    let key = &ev.keystroke.key;
                    if key == "escape" {
                        return;
                    }
                    if key == "enter" && !ev.keystroke.modifiers.shift {
                        this.ai_send_message(window, cx);
                        cx.notify();
                    }
                })),
            );

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

        div().size_full().flex().flex_row().child(main_col)
    }

    fn ai_rules_skills_strip(&mut self, cx: &mut Context<Self>, is_dark: bool) -> impl IntoElement {
        let p = theme::theme_palette(cx, is_dark);
        let g_snap = theme::glyph_snapshot(cx);
        let radius = g_snap.map(|g| g.border_radius).unwrap_or(p.radius_md);
        let accent = theme::ui_accent(cx);
        let accent_fg = theme::ui_accent_fg(cx);

        let active_rules = self.ai.active_rule_ids.clone();
        let active_skills = self.ai.active_skill_ids.clone();
        let rule_picker_open = self.ai.rule_picker_open;
        let skill_picker_open = self.ai.skill_picker_open;

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
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.ai.active_rule_ids.retain(|r| r != &rule_id);
                            cx.notify();
                        }),
                    );
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
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.ai.active_skill_ids.retain(|s| s != &skill_id);
                            cx.notify();
                        }),
                    );
                row = row.child(chip);
            }
        }

        // "+ Rule" toggle button
        let rule_btn_bg = if rule_picker_open { accent } else { p.panel_bg };
        let rule_btn_fg = if rule_picker_open {
            accent_fg
        } else {
            p.content_meta
        };
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
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, _, cx| {
                        this.ai.rule_picker_open = !this.ai.rule_picker_open;
                        this.ai.skill_picker_open = false;
                        cx.notify();
                    }),
                ),
        );

        // "+ Skill" toggle button
        let skill_btn_bg = if skill_picker_open {
            accent
        } else {
            p.panel_bg
        };
        let skill_btn_fg = if skill_picker_open {
            accent_fg
        } else {
            p.content_meta
        };
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
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, _, cx| {
                        this.ai.skill_picker_open = !this.ai.skill_picker_open;
                        this.ai.rule_picker_open = false;
                        cx.notify();
                    }),
                ),
        );

        // "Stage Writes" toggle chip
        let stage_writes = self.ai.stage_writes;
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
                .child(if stage_writes {
                    "Stage On"
                } else {
                    "Stage Off"
                })
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, _, cx| {
                        this.ai.stage_writes = !this.ai.stage_writes;
                        cx.notify();
                    }),
                ),
        );

        // Picker panels (inline dropdown below the strip)
        let picker = if rule_picker_open {
            let mut picker_col = div().w_full().px_4().pb_2().flex().flex_col().gap_1();
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
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, _, cx| {
                                if this.ai.active_rule_ids.contains(&rule_id) {
                                    this.ai.active_rule_ids.retain(|r| r != &rule_id);
                                } else {
                                    this.ai.active_rule_ids.push(rule_id.clone());
                                }
                                cx.notify();
                            }),
                        ),
                );
            }
            Some(picker_col)
        } else if skill_picker_open {
            let mut picker_col = div().w_full().px_4().pb_2().flex().flex_col().gap_1();
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
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, _, cx| {
                                if this.ai.active_skill_ids.contains(&skill_id) {
                                    this.ai.active_skill_ids.retain(|s| s != &skill_id);
                                } else {
                                    this.ai.active_skill_ids.push(skill_id.clone());
                                }
                                cx.notify();
                            }),
                        ),
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
        let active_id = self.ai.active_chat_id;

        // Collect input
        let input = self
            .ai.chats
            .iter()
            .find(|c| c.id == active_id)
            .map(|c| c.input_draft.trim().to_string())
            .unwrap_or_default();

        if input.is_empty() {
            return;
        }

        // If already inferring on this chat, ignore.
        if self.ai.stream_chat_id == Some(active_id) {
            return;
        }

        // Build provider routing from active provider + selected model.
        let provider = self.ai.active_provider_module.clone();
        let model_id = self.ai.chat_model_id.clone();

        // Image-generation providers fan out to a separate dispatch path. They take
        // a single prompt rather than a chat history and emit an `Image(path)` event
        // instead of streaming text tokens.
        if provider.as_str() == AI_IMAGE_PLAYGROUND_MODULE_NAME {
            self.dispatch_image_playground(active_id, &input, &provider, cx);
            self.ensure_ai_poll_task(window, cx);
            cx.notify();
            return;
        }

        let routing_result: Result<ProviderRouting, String> = match provider.as_str() {
            AI_LLAMA_CPP_MODULE_NAME => match model_id.as_deref() {
                None => Err("No model selected. Choose a model from the top bar.".to_string()),
                Some(id) => self
                    .ai.llama_cpp_models
                    .iter()
                    .find(|m| m.id == id)
                    .map(|m| ProviderRouting::LlamaCpp {
                        model_path: m.path.clone(),
                    })
                    .ok_or_else(|| {
                        format!("Model '{id}' not found. It may have been moved or deleted.")
                    }),
            },
            AI_OLLAMA_MODULE_NAME => {
                let cfg = OllamaConfig::load_or_create().unwrap_or_default();
                match model_id.as_deref() {
                    None => Err("No model selected. Choose a model from the top bar.".to_string()),
                    Some(id) => self
                        .ai.ollama_models
                        .iter()
                        .find(|m| m.id == id)
                        .map(|m| ProviderRouting::Ollama {
                            endpoint: cfg.endpoint.clone(),
                            model_name: m.model_tag.clone(),
                        })
                        .ok_or_else(|| format!("Model '{id}' not found in Ollama model list.")),
                }
            }
            AI_OPENAI_MODULE_NAME => match model_id.as_deref() {
                None => Err("No model selected. Choose a model from the top bar.".to_string()),
                Some(id) => self
                    .ai.openai_providers
                    .iter()
                    .find_map(|p| {
                        p.models.iter().find(|m| m.id == id).map(|m| {
                            ProviderRouting::OpenAi {
                                provider_id: p.id.clone(),
                                model_id: m.model_id.clone(),
                            }
                        })
                    })
                    .ok_or_else(|| format!("Model '{id}' not found.")),
            },
            AI_EXEC_CLAUDE_MODULE_NAME
            | AI_EXEC_CODEX_MODULE_NAME
            | AI_EXEC_GEMINI_MODULE_NAME
            | AI_EXEC_AIDER_MODULE_NAME => match cli_for_module(&provider) {
                Some((binary, model_flag)) => Ok(ProviderRouting::ExecCli {
                    binary: binary.to_string(),
                    model_flag: model_flag.map(str::to_string),
                    model_value: model_id.as_deref().unwrap_or("").to_string(),
                }),
                None => Err(format!("Unknown CLI provider module: {provider}")),
            },
            AI_APFEL_MODULE_NAME => Ok(ProviderRouting::Apfel),
            _ => Err("No AI provider enabled. Enable one in Modules.".to_string()),
        };

        // Resolve workspace context from cached entries (avoids a disk read per message).
        let workspace_context: Option<AiWorkspaceContext> =
            if self.is_module_enabled(WORKSPACE_MODULE_NAME) {
                self.ai.chat_workspace_id.as_deref().and_then(|ws_id| {
                    self.workspace_entries
                        .iter()
                        .find(|w| w.id == ws_id)
                        .map(AiWorkspaceContext::from_workspace_entry)
                })
            } else {
                None
            };

        // Push user message + clear draft. Track provider/model for session persistence.
        if let Some(chat) = self.ai.chats.iter_mut().find(|c| c.id == active_id) {
            chat.messages.push(AiMessage {
                role: AiMessageRole::User,
                content: input.clone(),
                provider: String::new(),
                images: Vec::new(),
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
                if let Some(chat) = self.ai.chats.iter_mut().find(|c| c.id == active_id) {
                    chat.messages.push(AiMessage {
                        role: AiMessageRole::Assistant,
                        content: msg,
                        provider: provider.clone(),
                        images: Vec::new(),
                    });
                }
                cx.notify();
                return;
            }
        };

        // Build full message history.
        let system = self.ai.default_system_prompt.clone();
        let history: Vec<(String, String)> = self
            .ai.chats
            .iter()
            .find(|c| c.id == active_id)
            .map(|c| {
                c.messages
                    .iter()
                    .map(|m| {
                        let role = match m.role {
                            AiMessageRole::User => "user".to_string(),
                            AiMessageRole::Assistant => "assistant".to_string(),
                        };
                        (role, m.content.clone())
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Placeholder assistant message (tokens stream into it).
        if let Some(chat) = self.ai.chats.iter_mut().find(|c| c.id == active_id) {
            chat.messages.push(AiMessage {
                role: AiMessageRole::Assistant,
                content: String::new(),
                provider: provider.clone(),
                images: Vec::new(),
            });
            chat.is_loading = true;
        }

        self.ai.stream_chat_id = Some(active_id);

        let tools = if workspace_context.is_some() {
            arcadia_core::modules::ai_tools::WORKSPACE_TOOLS.to_vec()
        } else {
            vec![]
        };

        let runtime = self.ai.runtime.get_or_insert_with(AiRuntimeHandle::start);
        let _ = runtime.request_tx.try_send(AiRuntimeRequest::Generate {
            routing,
            request: TextGenerationRequest {
                system,
                messages: history,
                max_tokens: 512,
                workspace_context,
                tools,
                active_rule_ids: self.ai.active_rule_ids.clone(),
                active_skill_ids: self.ai.active_skill_ids.clone(),
                stage_writes: self.ai.stage_writes,
            },
        });

        self.ensure_ai_poll_task(window, cx);
        cx.notify();
    }

    /// Dispatch path for image-generation providers. Pushes the user prompt + an
    /// empty assistant placeholder, then sends an `AiRuntimeRequest::GenerateImage`
    /// to the inference thread. The resulting PNG path arrives via
    /// `RuntimeEvent::Image` and is appended to the placeholder's `images` vec.
    fn dispatch_image_playground(
        &mut self,
        active_id: usize,
        input: &str,
        provider: &str,
        _cx: &mut Context<Self>,
    ) {
        use arcadia_core::config::image_playground::ImagePlaygroundConfig;
        use arcadia_core::config::ConfigFile;

        // Push user message + clear draft.
        if let Some(chat) = self.ai.chats.iter_mut().find(|c| c.id == active_id) {
            chat.messages.push(AiMessage {
                role: AiMessageRole::User,
                content: input.to_string(),
                provider: String::new(),
                images: Vec::new(),
            });
            chat.input_draft.clear();
            chat.session_provider = provider.to_string();
        }

        // Placeholder assistant message — the generated image attaches here.
        if let Some(chat) = self.ai.chats.iter_mut().find(|c| c.id == active_id) {
            chat.messages.push(AiMessage {
                role: AiMessageRole::Assistant,
                content: String::new(),
                provider: provider.to_string(),
                images: Vec::new(),
            });
            chat.is_loading = true;
        }

        self.ai.stream_chat_id = Some(active_id);

        let style = ImagePlaygroundConfig::load_or_create()
            .map(|c| c.default_style.as_swift_token().to_string())
            .ok();

        let workspace_context: Option<AiWorkspaceContext> =
            if self.is_module_enabled(WORKSPACE_MODULE_NAME) {
                self.ai.chat_workspace_id.as_deref().and_then(|ws_id| {
                    self.workspace_entries
                        .iter()
                        .find(|w| w.id == ws_id)
                        .map(AiWorkspaceContext::from_workspace_entry)
                })
            } else {
                None
            };

        let runtime = self.ai.runtime.get_or_insert_with(AiRuntimeHandle::start);
        let _ = runtime.request_tx.try_send(AiRuntimeRequest::GenerateImage {
            routing: ImageProviderRouting::ImagePlayground,
            request: ImageGenerationRequest {
                prompt: input.to_string(),
                negative_prompt: None,
                width: 0,
                height: 0,
                workspace_context,
                style,
            },
        });
    }
}
