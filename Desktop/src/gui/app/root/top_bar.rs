use arcadia_core::config::ConfigFile as _;
use openframe::prelude::FluentBuilder as _;
use openframe::{
    canvas, div, point, px, rgb, Context, InteractiveElement, IntoElement, KeyDownEvent,
    MouseButton, ParentElement, PathBuilder, Styled, Window,
};

use arcadia_core::modules;

use crate::gui::app::text_input_caret::{text_with_trailing_caret, TEXT_INPUT_CARET_CHAR};
use crate::gui::app::ArcadiaRoot;
#[cfg(feature = "gui")]
use crate::gui::app::ShellMode;
use crate::gui::theme::{self};

const LATE_ROOMS: &[(&str, u32)] = &[("1", 1), ("2", 2), ("3", 3), ("4", 4), ("5", 5)];

impl ArcadiaRoot {
    pub(crate) fn render_main_top_bar(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
        active_page_title: openframe::SharedString,
        active_page_glyph: openframe::SharedString,
        is_dark: bool,
    ) -> impl IntoElement {
        let glyph = theme::glyph_snapshot(cx);
        let is_glyph = glyph.is_some();
        let glyph_border = glyph.as_ref().map(|g| g.border);
        let title_color = glyph.as_ref().map(|g| g.text).unwrap_or_else(|| {
            if is_dark {
                rgb(0xe5e7eb)
            } else {
                rgb(0x1f2937)
            }
        });
        let action_pill_bg = theme::action_pill_bg(cx, is_dark);
        let action_pill_tc = theme::action_pill_text(cx, is_dark);
        let action_pill_hover = theme::action_pill_hover_bg(cx, is_dark);
        let _shell_mode_generic_bg = theme::ui_surface(cx, is_dark);
        let _shell_mode_alt_bg = theme::ui_surface2(cx, is_dark);
        let shell_mode_generic_fg = theme::ui_accent(cx);
        let shell_mode_alt_fg = theme::ui_subtext(cx, is_dark);
        let radius = glyph.as_ref().map(|g| g.border_radius).unwrap_or(6.0);
        div()
            .w_full()
            .px_3()
            .py_2()
            .when(!is_glyph, |d| {
                d.border_b_1().border_color(if is_dark { rgb(0x2a3340) } else { rgb(0xe6e8ef) })
            })
            .child(
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(Self::sidebar_toggle_button(cx, active_page_glyph.as_ref(), is_dark, glyph))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(openframe::FontWeight::SEMIBOLD)
                                            .text_color(title_color)
                                            .child(active_page_title),
                                    )
                            .child({
                                if self.active_page_id.as_str() == "ai.chat"
                                    && !self.ai.chat_show_dashboard
                                    && !self.ai.chats.is_empty()
                                {
                                    let label = {
                                        use arcadia_core::config::modules::{
                                            AI_APFEL_MODULE_NAME, AI_EXEC_AIDER_MODULE_NAME,
                                            AI_EXEC_CLAUDE_MODULE_NAME, AI_EXEC_CODEX_MODULE_NAME,
                                            AI_EXEC_GEMINI_MODULE_NAME,
                                        };
                                        use arcadia_core::modules::ai::provider_display_name;
                                        let provider = self.ai.active_provider_module.as_str();
                                        let mid = self.ai.chat_model_id.as_deref();
                                        // CLI providers and on-device providers: display name needs no model ID
                                        if matches!(provider, p if p == AI_EXEC_CLAUDE_MODULE_NAME
                                            || p == AI_EXEC_CODEX_MODULE_NAME
                                            || p == AI_EXEC_GEMINI_MODULE_NAME
                                            || p == AI_EXEC_AIDER_MODULE_NAME
                                            || p == AI_APFEL_MODULE_NAME)
                                        {
                                            self.ai.detected_cli_providers.iter()
                                                .find(|c| c.id == provider)
                                                .map(|c| c.label.clone())
                                                .or_else(|| provider_display_name(provider).map(|s| s.to_string()))
                                                .unwrap_or_else(|| provider.to_string())
                                        } else {
                                            mid.and_then(|id| {
                                                self.ai.llama_cpp_models.iter().find(|m| m.id == id).map(|m| m.name.clone())
                                                    .or_else(|| self.ai.ollama_models.iter().find(|m| m.id == id).map(|m| m.name.clone()))
                                                    .or_else(|| self.ai.openai_providers.iter().flat_map(|p| p.models.iter()).find(|m| m.id == id).map(|m| m.name.clone()))
                                            })
                                            .unwrap_or_else(|| "No Model".to_string())
                                        }
                                    };
                                    div()
                                        .px_2()
                                        .py_0p5()
                                        .rounded(px(radius))
                                        .cursor_pointer()
                                        .text_xs()
                                        .bg(action_pill_bg)
                                        .text_color(action_pill_tc)
                                        .hover(move |style| style.bg(action_pill_hover))
                                        .child(label)
                                        .on_mouse_down(
                                            openframe::MouseButton::Left,
                                            cx.listener(|this, event: &openframe::MouseDownEvent, _, cx| {
                                                this.ai.chat_model_picker_open = !this.ai.chat_model_picker_open;
                                                #[cfg(feature = "gui")]
                                                { this.context_menu_position = event.position; }
                                                #[cfg(not(feature = "gui"))]
                                                let _ = event;
                                                cx.stop_propagation();
                                                cx.notify();
                                            }),
                                        )
                                } else {
                                    div()
                                }
                            })
                            .child({
                                if self.active_page_id.as_str() == "ai.chat"
                                    && !self.ai.chat_show_dashboard
                                    && !self.ai.chats.is_empty()
                                    && self.is_module_enabled(arcadia_core::config::modules::WORKSPACE_MODULE_NAME)
                                {
                                    let ws_label = self.ai.chat_workspace_id
                                        .as_deref()
                                        .and_then(|id| {
                                            arcadia_core::config::workspace::WorkspacesConfig::load_or_create().ok()
                                                .and_then(|cfg| cfg.workspaces.into_iter().find(|w| w.id == id))
                                                .map(|w| w.label)
                                        })
                                        .unwrap_or_else(|| "No Workspace".to_string());
                                    div()
                                        .px_2()
                                        .py_0p5()
                                        .rounded(px(radius))
                                        .cursor_pointer()
                                        .text_xs()
                                        .bg(action_pill_bg)
                                        .text_color(action_pill_tc)
                                        .hover(move |style| style.bg(action_pill_hover))
                                        .child(ws_label)
                                        .on_mouse_down(
                                            openframe::MouseButton::Left,
                                            cx.listener(|this, event: &openframe::MouseDownEvent, _, cx| {
                                                this.ai.chat_workspace_picker_open = !this.ai.chat_workspace_picker_open;
                                                this.ai.chat_model_picker_open = false;
                                                #[cfg(feature = "gui")]
                                                { this.context_menu_position = event.position; }
                                                #[cfg(not(feature = "gui"))]
                                                let _ = event;
                                                cx.stop_propagation();
                                                cx.notify();
                                            }),
                                        )
                                } else {
                                    div()
                                }
                            })
                            .child({
                                if self.active_page_id.as_str() == "ai.chat"
                                    && !self.ai.chat_show_dashboard
                                    && !self.ai.chats.is_empty()
                                {
                                    // Estimate context fill ratio (chars / 512K chars ≈ 128K tokens).
                                    let active_id = self.ai.active_chat_id;
                                    let msg_chars: usize = self.ai.chats.iter()
                                        .find(|c| c.id == active_id)
                                        .map(|c| c.messages.iter().map(|m| m.content.len()).sum())
                                        .unwrap_or(0);
                                    let system_chars = self.ai.default_system_prompt.len();
                                    let rules_chars: usize = {
                                        use arcadia_core::config::ai_rules::{all_rules, AiRulesConfig};
                                        let cfg = AiRulesConfig::load_or_create().unwrap_or_default();
                                        let all_r = all_rules(&cfg);
                                        self.ai.active_rule_ids.iter()
                                            .filter_map(|id| all_r.iter().find(|r| &r.id == id))
                                            .map(|r| r.system_fragment.len())
                                            .sum()
                                    };
                                    let skills_chars: usize = {
                                        use arcadia_core::config::ai_skills::{all_skills, AiSkillsConfig};
                                        let cfg = AiSkillsConfig::load_or_create().unwrap_or_default();
                                        let all_s = all_skills(&cfg);
                                        self.ai.active_skill_ids.iter()
                                            .filter_map(|id| all_s.iter().find(|s| &s.id == id))
                                            .map(|s| s.system_fragment.len())
                                            .sum()
                                    };
                                    let total_chars = (msg_chars + system_chars + rules_chars + skills_chars) as f32;
                                    let fill_ratio = (total_chars / 512_000.0_f32).clamp(0.0, 1.0);

                                    let track_color = {
                                        let mut c = action_pill_tc;
                                        c.a *= 0.3;
                                        c
                                    };
                                    let fill_color = theme::ui_accent(cx);
                                    let context_viewer_open = self.ai.chat_context_viewer_open;
                                    let btn_bg = if context_viewer_open { fill_color } else { action_pill_bg };
                                    let btn_tc = if context_viewer_open { theme::ui_accent_fg(cx) } else { action_pill_tc };

                                    let ring = canvas(
                                        |_, _, _| {},
                                        move |bounds, _, window, _| {
                                            let ox = f32::from(bounds.origin.x);
                                            let oy = f32::from(bounds.origin.y);
                                            let cx_f = ox + 7.0;
                                            let cy_f = oy + 7.0;
                                            let r = 4.5_f32;

                                            // Track — two semicircles to form a full ring.
                                            {
                                                let mut pb = PathBuilder::stroke(px(1.5));
                                                pb.move_to(point(px(cx_f), px(cy_f - r)));
                                                pb.arc_to(point(px(r), px(r)), px(0.0), false, true, point(px(cx_f), px(cy_f + r)));
                                                pb.arc_to(point(px(r), px(r)), px(0.0), false, true, point(px(cx_f), px(cy_f - r)));
                                                if let Ok(path) = pb.build() {
                                                    window.paint_path(path, track_color);
                                                }
                                            }

                                            // Fill arc — clockwise from 12 o'clock.
                                            if fill_ratio > 0.005 {
                                                let mut pb = PathBuilder::stroke(px(1.5));
                                                pb.move_to(point(px(cx_f), px(cy_f - r)));
                                                if fill_ratio >= 0.995 {
                                                    pb.arc_to(point(px(r), px(r)), px(0.0), false, true, point(px(cx_f), px(cy_f + r)));
                                                    pb.arc_to(point(px(r), px(r)), px(0.0), false, true, point(px(cx_f), px(cy_f - r)));
                                                } else {
                                                    let angle = fill_ratio * std::f32::consts::TAU;
                                                    let end_x = cx_f + r * angle.sin();
                                                    let end_y = cy_f - r * angle.cos();
                                                    pb.arc_to(point(px(r), px(r)), px(0.0), fill_ratio > 0.5, true, point(px(end_x), px(end_y)));
                                                }
                                                if let Ok(path) = pb.build() {
                                                    window.paint_path(path, fill_color);
                                                }
                                            }
                                        },
                                    )
                                    .w(px(14.))
                                    .h(px(14.));

                                    div()
                                        .px_2()
                                        .py_0p5()
                                        .rounded(px(radius))
                                        .cursor_pointer()
                                        .text_xs()
                                        .bg(btn_bg)
                                        .text_color(btn_tc)
                                        .flex()
                                        .items_center()
                                        .gap_1p5()
                                        .hover(move |style| style.bg(action_pill_hover))
                                        .child(ring)
                                        .child("Context")
                                        .on_mouse_down(
                                            openframe::MouseButton::Left,
                                            cx.listener(|this, event: &openframe::MouseDownEvent, _, cx| {
                                                this.ai.chat_context_viewer_open = !this.ai.chat_context_viewer_open;
                                                this.ai.chat_model_picker_open = false;
                                                this.ai.chat_workspace_picker_open = false;
                                                #[cfg(feature = "gui")]
                                                { this.context_menu_position = event.position; }
                                                #[cfg(not(feature = "gui"))]
                                                let _ = event;
                                                cx.stop_propagation();
                                                cx.notify();
                                            }),
                                        )
                                } else {
                                    div()
                                }
                            })
                            .child({
                                if self.active_page_id.as_str() == "extensions.settings" {
                                    div()
                                        .px_2()
                                        .py_0p5()
                                        .rounded(px(radius))
                                        .cursor_pointer()
                                        .text_xs()
                                        .bg(action_pill_bg)
                                        .text_color(action_pill_tc)
                                        .hover(move |style| style.bg(action_pill_hover))
                                        .child("Reload Extensions")
                                        .on_mouse_down(
                                            openframe::MouseButton::Left,
                                            cx.listener(|this, _, _, cx| {
                                                let ctx = this.execution_context();
                                                let _ = modules::execute_command(
                                                    "python-host.reload",
                                                    &[],
                                                    &ctx,
                                                );
                                                this.reload_extension_state(cx);
                                                cx.notify();
                                            }),
                                        )
                                } else {
                                    div()
                                }
                            })
                            .child({
                                if self.active_page_id.as_str() == "network.nodes" {
                                    div()
                                        .flex()
                                        .flex_row()
                                        .gap_1()
                                        .child(
                                            div()
                                                .px_2()
                                                .py_0p5()
                                                .rounded(px(radius))
                                                .cursor_pointer()
                                                .text_xs()
                                                .bg(action_pill_bg)
                                                .text_color(action_pill_tc)
                                                .hover(move |style| style.bg(action_pill_hover))
                                                .child("Refresh")
                                                .on_mouse_down(
                                                    openframe::MouseButton::Left,
                                                    cx.listener(|this, _, _, cx| {
                                                        this.lan_command_feedback =
                                                            "LAN nodes status refreshed.".to_string();
                                                        cx.notify();
                                                    }),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .px_2()
                                                .py_0p5()
                                                .rounded(px(radius))
                                                .cursor_pointer()
                                                .text_xs()
                                                .bg(action_pill_bg)
                                                .text_color(action_pill_tc)
                                                .hover(move |style| style.bg(action_pill_hover))
                                                .child("Scan")
                                                .on_mouse_down(
                                                    openframe::MouseButton::Left,
                                                    cx.listener(|this, _, _, cx| {
                                                        use arcadia_core::modules::lan::discover_lan_peers;
                                                        match discover_lan_peers(None) {
                                                            Ok(peers) => {
                                                                let n = peers.len();
                                                                this.lan_discovered_peers = peers;
                                                                this.lan_command_feedback =
                                                                    format!("Scan finished — {n} peer(s).");
                                                            }
                                                            Err(err) => {
                                                                this.lan_discovered_peers.clear();
                                                                this.lan_command_feedback = err;
                                                            }
                                                        }
                                                        cx.notify();
                                                    }),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .px_2()
                                                .py_0p5()
                                                .rounded(px(radius))
                                                .cursor_pointer()
                                                .text_xs()
                                                .bg(action_pill_bg)
                                                .text_color(action_pill_tc)
                                                .hover(move |style| style.bg(action_pill_hover))
                                                .child("Save connected (all)")
                                                .on_mouse_down(
                                                    openframe::MouseButton::Left,
                                                    cx.listener(|this, _, _, cx| {
                                                        this.lan_execute_feedback(
                                                            "lan.node",
                                                            vec!["save".into()],
                                                        );
                                                        cx.notify();
                                                    }),
                                                ),
                                        )
                                } else {
                                    div().flex().flex_row()
                                }
                            })
                            .child(if self.active_page_id.as_str() == "late.now_playing" {
                                div()
                                    .flex()
                                    .flex_row()
                                    .gap_1()
                                    .children(LATE_ROOMS.iter().map(|(label, room_id)| {
                                        let rid = *room_id;
                                        let is_active = rid == self.late.active_room;
                                        div()
                                            .cursor_pointer()
                                            .px_2()
                                            .py_0p5()
                                            .rounded(px(radius))
                                            .text_xs()
                                            .font_weight(openframe::FontWeight::SEMIBOLD)
                                            .bg(if is_active {
                                                if is_dark { rgb(0x0d9488) } else { rgb(0x99f6e4) }
                                            } else {
                                                action_pill_bg
                                            })
                                            .text_color(if is_active {
                                                if is_dark { rgb(0xf0fdfa) } else { rgb(0x134e4a) }
                                            } else {
                                                action_pill_tc
                                            })
                                            .hover(move |style| {
                                                if !is_active {
                                                    style.bg(action_pill_hover)
                                                } else {
                                                    style
                                                }
                                            })
                                            .child(*label)
                                            .on_mouse_down(
                                                openframe::MouseButton::Left,
                                                cx.listener(move |this, _, _, cx| {
                                                    this.late.active_room = rid;
                                                    arcadia_core::modules::late::send_ws(
                                                        format!(r#"{{"type":"subscribe","room_id":{rid}}}"#),
                                                    );
                                                    cx.notify();
                                                }),
                                            )
                                    }))
                            } else {
                                div()
                            })
                            .child({
                                #[cfg(feature = "gui")]
                                {
                                    if self.active_page_id.as_str() == "editor.main"
                                        && !self.code_editor.tabs.is_empty()
                                        && !self.code_editor.show_dashboard
                                    {
                                        let active_idx = self.code_editor.active_tab
                                            .min(self.code_editor.tabs.len().saturating_sub(1));
                                        let ws_label = self.code_editor.tabs
                                            .get(active_idx)
                                            .and_then(|t| t.workspace_path.as_deref())
                                            .map(|p| {
                                                arcadia_core::config::workspace::list_workspaces()
                                                    .into_iter()
                                                    .find(|w| w.path == p)
                                                    .map(|w| if w.label.is_empty() {
                                                        p.rsplit('/').next().unwrap_or(p).to_string()
                                                    } else {
                                                        w.label.clone()
                                                    })
                                                    .unwrap_or_else(|| {
                                                        p.rsplit('/').next().unwrap_or(p).to_string()
                                                    })
                                            });
                                        let has_file = self.code_editor.tabs
                                            .get(active_idx)
                                            .map(|t| t.file_path.is_some())
                                            .unwrap_or(false);
                                        let is_dirty = self.code_editor.tabs
                                            .get(active_idx)
                                            .map(|t| t.file_path.is_some() && t.content != t.saved_content)
                                            .unwrap_or(false);
                                        let has_explorer = self.code_editor.tabs
                                            .get(active_idx)
                                            .and_then(|t| t.workspace_path.as_deref())
                                            .map(|p| arcadia_core::config::workspace::any_workspace_grants(p, "workspace.read"))
                                            .unwrap_or(false);
                                        let make_pill = |label: &'static str| {
                                            div()
                                                .px_2()
                                                .py_0p5()
                                                .rounded(px(radius))
                                                .cursor_pointer()
                                                .text_xs()
                                                .bg(action_pill_bg)
                                                .text_color(action_pill_tc)
                                                .hover(move |style| style.bg(action_pill_hover))
                                                .child(label)
                                        };
                                        div()
                                            .flex()
                                            .flex_row()
                                            .gap_1()
                                            // Workspace picker
                                            .child(
                                                div()
                                                    .px_2()
                                                    .py_0p5()
                                                    .rounded(px(radius))
                                                    .cursor_pointer()
                                                    .text_xs()
                                                    .bg(action_pill_bg)
                                                    .text_color(action_pill_tc)
                                                    .hover(move |style| style.bg(action_pill_hover))
                                                    .child(ws_label.unwrap_or_else(|| "Open Workspace".to_string()))
                                                    .on_mouse_down(
                                                        openframe::MouseButton::Left,
                                                        cx.listener(|this, event: &openframe::MouseDownEvent, _, cx| {
                                                            this.code_editor.workspace_picker_open =
                                                                !this.code_editor.workspace_picker_open;
                                                            this.context_menu_position = event.position;
                                                            cx.stop_propagation();
                                                            cx.notify();
                                                        }),
                                                    )
                                            )
                                            // Explorer (workspace.read required)
                                            .when(has_explorer, |row| {
                                                row.child(
                                                    make_pill("Explorer")
                                                        .on_mouse_down(
                                                            openframe::MouseButton::Left,
                                                            cx.listener(|this, event: &openframe::MouseDownEvent, _, cx| {
                                                                this.code_editor.explorer_open =
                                                                    !this.code_editor.explorer_open;
                                                                this.context_menu_position = event.position;
                                                                cx.stop_propagation();
                                                                cx.notify();
                                                            }),
                                                        )
                                                )
                                            })
                                            // Open
                                            .child(
                                                make_pill("Open")
                                                    .on_mouse_down(
                                                        openframe::MouseButton::Left,
                                                        cx.listener(|_this, _, window, cx| {
                                                            let receiver = cx.prompt_for_paths(openframe::PathPromptOptions {
                                                                files: true,
                                                                directories: false,
                                                                multiple: false,
                                                                prompt: None,
                                                                allowed_extensions: Vec::new(),
                                                            });
                                                            cx.spawn_in(window, move |this: openframe::WeakEntity<ArcadiaRoot>, cx: &mut openframe::AsyncWindowContext| {
                                                                let mut cx = cx.clone();
                                                                async move {
                                                                    if let Ok(Ok(Some(paths))) = receiver.await {
                                                                        if let Some(path) = paths.into_iter().next() {
                                                                            if let Ok(text) = std::fs::read_to_string(&path) {
                                                                                let path_str = path.to_string_lossy().to_string();
                                                                                let title = path
                                                                                    .file_name()
                                                                                    .map(|n| n.to_string_lossy().to_string())
                                                                                    .unwrap_or_else(|| path_str.clone());
                                                                                let lang = crate::gui::app::code_editor_panel::detect_language(&title);
                                                                                cx.update(|_, app| {
                                                                                    this.update(app, |this, cx| {
                                                                                        let idx = this.code_editor.active_tab
                                                                                            .min(this.code_editor.tabs.len().saturating_sub(1));
                                                                                        if let Some(tab) = this.code_editor.tabs.get_mut(idx) {
                                                                                            tab.saved_content = text.clone();
                                                                                            tab.content = text;
                                                                                            tab.file_path = Some(path_str);
                                                                                            tab.title = title;
                                                                                            tab.language = lang;
                                                                                            tab.cursor = 0;
                                                                                            tab.selection_anchor = None;
                                                                                            tab.highlight_dirty = true;
                                                                                        }
                                                                                        this.save_editor_session();
                                                                                        cx.notify();
                                                                                    }).ok();
                                                                                }).ok();
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }).detach();
                                                        }),
                                                    )
                                            )
                                            // Save (only when a file is open)
                                            .when(has_file, |row| {
                                                row.child(
                                                    make_pill("Save")
                                                        .on_mouse_down(
                                                            openframe::MouseButton::Left,
                                                            cx.listener(|this, _, _, cx| {
                                                                let idx = this.code_editor.active_tab
                                                                    .min(this.code_editor.tabs.len().saturating_sub(1));
                                                                if let Some(tab) = this.code_editor.tabs.get_mut(idx) {
                                                                    if let Some(path) = tab.file_path.clone() {
                                                                        let _ = std::fs::write(&path, &tab.content);
                                                                        tab.saved_content = tab.content.clone();
                                                                    }
                                                                }
                                                                this.save_editor_session();
                                                                cx.notify();
                                                            }),
                                                        )
                                                )
                                            })
                                            // Save As
                                            .child(
                                                make_pill("Save As")
                                                    .on_mouse_down(
                                                        openframe::MouseButton::Left,
                                                        cx.listener(|this, _, window, cx| {
                                                            let idx = this.code_editor.active_tab
                                                                .min(this.code_editor.tabs.len().saturating_sub(1));
                                                            let (content, suggested) = this.code_editor.tabs
                                                                .get(idx)
                                                                .map(|t| (t.content.clone(), t.title.clone()))
                                                                .unwrap_or_default();
                                                            let init_dir = this.code_editor.tabs
                                                                .get(idx)
                                                                .and_then(|t| t.file_path.as_deref())
                                                                .and_then(|p| std::path::Path::new(p).parent())
                                                                .map(|p| p.to_path_buf())
                                                                .unwrap_or_default();
                                                            let receiver = cx.prompt_for_new_path(
                                                                &init_dir,
                                                                if suggested.is_empty() { None } else { Some(suggested.as_str()) },
                                                            );
                                                            cx.spawn_in(window, move |this: openframe::WeakEntity<ArcadiaRoot>, cx: &mut openframe::AsyncWindowContext| {
                                                                let mut cx = cx.clone();
                                                                async move {
                                                                    if let Ok(Ok(Some(path))) = receiver.await {
                                                                        let _ = std::fs::write(&path, &content);
                                                                        let path_str = path.to_string_lossy().to_string();
                                                                        let title = path
                                                                            .file_name()
                                                                            .map(|n| n.to_string_lossy().to_string())
                                                                            .unwrap_or_else(|| path_str.clone());
                                                                        let lang = crate::gui::app::code_editor_panel::detect_language(&title);
                                                                        cx.update(|_, app| {
                                                                            this.update(app, |this, cx| {
                                                                                let idx = this.code_editor.active_tab
                                                                                    .min(this.code_editor.tabs.len().saturating_sub(1));
                                                                                if let Some(tab) = this.code_editor.tabs.get_mut(idx) {
                                                                                    tab.file_path = Some(path_str);
                                                                                    tab.title = title;
                                                                                    tab.saved_content = tab.content.clone();
                                                                                    tab.language = lang;
                                                                                    tab.highlight_dirty = true;
                                                                                }
                                                                                this.save_editor_session();
                                                                                cx.notify();
                                                                            }).ok();
                                                                        }).ok();
                                                                    }
                                                                }
                                                            }).detach();
                                                        }),
                                                    )
                                            )
                                            // Undo history
                                            .when(self.code_editor.undo_enabled, |row| {
                                                row.child(
                                                    make_pill("History")
                                                        .on_mouse_down(
                                                            openframe::MouseButton::Left,
                                                            cx.listener(|this, event: &openframe::MouseDownEvent, _, cx| {
                                                                this.code_editor.undo_history_open =
                                                                    !this.code_editor.undo_history_open;
                                                                this.context_menu_position = event.position;
                                                                cx.stop_propagation();
                                                                cx.notify();
                                                            }),
                                                        )
                                                )
                                            })
                                            // Restore (only when file open and dirty)
                                            .when(is_dirty, |row| {
                                                row.child(
                                                    make_pill("Restore")
                                                        .on_mouse_down(
                                                            openframe::MouseButton::Left,
                                                            cx.listener(|this, _, _, cx| {
                                                                let idx = this.code_editor.active_tab
                                                                    .min(this.code_editor.tabs.len().saturating_sub(1));
                                                                if let Some(tab) = this.code_editor.tabs.get_mut(idx) {
                                                                    tab.content = tab.saved_content.clone();
                                                                    tab.cursor = 0;
                                                                    tab.selection_anchor = None;
                                                                    tab.highlight_dirty = true;
                                                                    cx.notify();
                                                                }
                                                            }),
                                                        )
                                                )
                                            })
                                    } else {
                                        div().flex().flex_row()
                                    }
                                }
                                #[cfg(not(feature = "gui"))]
                                { div().flex().flex_row() }
                            })
                            .child({
                                #[cfg(feature = "gui")]
                                {
                                    if self.active_page_id.as_str() == "editor.visual"
                                        && !self.visual_editor.tabs.is_empty()
                                        && !self.visual_editor.show_dashboard
                                    {
                                        let active_idx = self.visual_editor.active_tab
                                            .min(self.visual_editor.tabs.len().saturating_sub(1));
                                        let ws_label = self.visual_editor.tabs
                                            .get(active_idx)
                                            .and_then(|t| t.workspace_path.as_deref())
                                            .map(|p| {
                                                arcadia_core::config::workspace::list_workspaces()
                                                    .into_iter()
                                                    .find(|w| w.path == p)
                                                    .map(|w| if w.label.is_empty() {
                                                        p.rsplit('/').next().unwrap_or(p).to_string()
                                                    } else {
                                                        w.label.clone()
                                                    })
                                                    .unwrap_or_else(|| {
                                                        p.rsplit('/').next().unwrap_or(p).to_string()
                                                    })
                                            });
                                        let has_file = self.visual_editor.tabs
                                            .get(active_idx)
                                            .map(|t| t.file_path.is_some())
                                            .unwrap_or(false);
                                        let is_dirty = self.visual_editor.tabs
                                            .get(active_idx)
                                            .map(|t| t.file_path.is_some() && t.content != t.saved_content)
                                            .unwrap_or(false);
                                        let has_explorer = self.visual_editor.tabs
                                            .get(active_idx)
                                            .and_then(|t| t.workspace_path.as_deref())
                                            .map(|p| arcadia_core::config::workspace::any_workspace_grants(p, "workspace.read"))
                                            .unwrap_or(false);
                                        let make_pill = |label: &'static str| {
                                            div()
                                                .px_2()
                                                .py_0p5()
                                                .rounded(px(radius))
                                                .cursor_pointer()
                                                .text_xs()
                                                .bg(action_pill_bg)
                                                .text_color(action_pill_tc)
                                                .hover(move |style| style.bg(action_pill_hover))
                                                .child(label)
                                        };
                                        div()
                                            .flex()
                                            .flex_row()
                                            .gap_1()
                                            .child(
                                                div()
                                                    .px_2()
                                                    .py_0p5()
                                                    .rounded(px(radius))
                                                    .cursor_pointer()
                                                    .text_xs()
                                                    .bg(action_pill_bg)
                                                    .text_color(action_pill_tc)
                                                    .hover(move |style| style.bg(action_pill_hover))
                                                    .child(ws_label.unwrap_or_else(|| "Open Workspace".to_string()))
                                                    .on_mouse_down(
                                                        openframe::MouseButton::Left,
                                                        cx.listener(|this, event: &openframe::MouseDownEvent, _, cx| {
                                                            this.visual_editor.workspace_picker_open =
                                                                !this.visual_editor.workspace_picker_open;
                                                            this.context_menu_position = event.position;
                                                            cx.stop_propagation();
                                                            cx.notify();
                                                        }),
                                                    )
                                            )
                                            .when(has_explorer, |row| {
                                                row.child(
                                                    make_pill("Explorer")
                                                        .on_mouse_down(
                                                            openframe::MouseButton::Left,
                                                            cx.listener(|this, event: &openframe::MouseDownEvent, _, cx| {
                                                                this.visual_editor.explorer_open =
                                                                    !this.visual_editor.explorer_open;
                                                                this.context_menu_position = event.position;
                                                                cx.stop_propagation();
                                                                cx.notify();
                                                            }),
                                                        )
                                                )
                                            })
                                            .child(
                                                make_pill("Palette")
                                                    .on_mouse_down(
                                                        openframe::MouseButton::Left,
                                                        cx.listener(|this, _, _, cx| {
                                                            this.visual_editor.palette_open =
                                                                !this.visual_editor.palette_open;
                                                            cx.stop_propagation();
                                                            cx.notify();
                                                        }),
                                                    )
                                            )
                                            .child(
                                                make_pill("Open")
                                                    .on_mouse_down(
                                                        openframe::MouseButton::Left,
                                                        cx.listener(|_this, _, window, cx| {
                                                            let receiver = cx.prompt_for_paths(openframe::PathPromptOptions {
                                                                files: true,
                                                                directories: false,
                                                                multiple: false,
                                                                prompt: None,
                                                                allowed_extensions: vec!["py".into()],
                                                            });
                                                            cx.spawn_in(window, move |this: openframe::WeakEntity<ArcadiaRoot>, cx: &mut openframe::AsyncWindowContext| {
                                                                let mut cx = cx.clone();
                                                                async move {
                                                                    if let Ok(Ok(Some(paths))) = receiver.await {
                                                                        if let Some(path) = paths.into_iter().next() {
                                                                            if let Ok(text) = std::fs::read_to_string(&path) {
                                                                                let path_str = path.to_string_lossy().to_string();
                                                                                let title = path
                                                                                    .file_name()
                                                                                    .map(|n| n.to_string_lossy().to_string())
                                                                                    .unwrap_or_else(|| path_str.clone());
                                                                                cx.update(|_, app| {
                                                                                    this.update(app, |this, cx| {
                                                                                        let idx = this.visual_editor.active_tab
                                                                                            .min(this.visual_editor.tabs.len().saturating_sub(1));
                                                                                        if let Some(tab) = this.visual_editor.tabs.get_mut(idx) {
                                                                                            tab.saved_content = text.clone();
                                                                                            tab.content = text;
                                                                                            tab.file_path = Some(path_str);
                                                                                            tab.title = title;
                                                                                        }
                                                                                        this.save_visual_editor_session();
                                                                                        cx.notify();
                                                                                    }).ok();
                                                                                }).ok();
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }).detach();
                                                        }),
                                                    )
                                            )
                                            .when(has_file, |row| {
                                                row.child(
                                                    make_pill("Save")
                                                        .on_mouse_down(
                                                            openframe::MouseButton::Left,
                                                            cx.listener(|this, _, _, cx| {
                                                                let idx = this.visual_editor.active_tab
                                                                    .min(this.visual_editor.tabs.len().saturating_sub(1));
                                                                if let Some(tab) = this.visual_editor.tabs.get_mut(idx) {
                                                                    if let Some(path) = tab.file_path.clone() {
                                                                        let _ = std::fs::write(&path, &tab.content);
                                                                        tab.saved_content = tab.content.clone();
                                                                    }
                                                                }
                                                                this.save_visual_editor_session();
                                                                cx.notify();
                                                            }),
                                                        )
                                                )
                                            })
                                            .child(
                                                make_pill("Save As")
                                                    .on_mouse_down(
                                                        openframe::MouseButton::Left,
                                                        cx.listener(|this, _, window, cx| {
                                                            let idx = this.visual_editor.active_tab
                                                                .min(this.visual_editor.tabs.len().saturating_sub(1));
                                                            let (content, suggested) = this.visual_editor.tabs
                                                                .get(idx)
                                                                .map(|t| (t.content.clone(), t.title.clone()))
                                                                .unwrap_or_default();
                                                            let init_dir = this.visual_editor.tabs
                                                                .get(idx)
                                                                .and_then(|t| t.file_path.as_deref())
                                                                .and_then(|p| std::path::Path::new(p).parent())
                                                                .map(|p| p.to_path_buf())
                                                                .unwrap_or_default();
                                                            let receiver = cx.prompt_for_new_path(
                                                                &init_dir,
                                                                if suggested.is_empty() { None } else { Some(suggested.as_str()) },
                                                            );
                                                            cx.spawn_in(window, move |this: openframe::WeakEntity<ArcadiaRoot>, cx: &mut openframe::AsyncWindowContext| {
                                                                let mut cx = cx.clone();
                                                                async move {
                                                                    if let Ok(Ok(Some(path))) = receiver.await {
                                                                        let _ = std::fs::write(&path, &content);
                                                                        let path_str = path.to_string_lossy().to_string();
                                                                        let title = path
                                                                            .file_name()
                                                                            .map(|n| n.to_string_lossy().to_string())
                                                                            .unwrap_or_else(|| path_str.clone());
                                                                        cx.update(|_, app| {
                                                                            this.update(app, |this, cx| {
                                                                                let idx = this.visual_editor.active_tab
                                                                                    .min(this.visual_editor.tabs.len().saturating_sub(1));
                                                                                if let Some(tab) = this.visual_editor.tabs.get_mut(idx) {
                                                                                    tab.file_path = Some(path_str);
                                                                                    tab.title = title;
                                                                                    tab.saved_content = tab.content.clone();
                                                                                }
                                                                                this.save_visual_editor_session();
                                                                                cx.notify();
                                                                            }).ok();
                                                                        }).ok();
                                                                    }
                                                                }
                                                            }).detach();
                                                        }),
                                                    )
                                            )
                                            .when(is_dirty, |row| {
                                                row.child(
                                                    make_pill("Restore")
                                                        .on_mouse_down(
                                                            openframe::MouseButton::Left,
                                                            cx.listener(|this, _, _, cx| {
                                                                let idx = this.visual_editor.active_tab
                                                                    .min(this.visual_editor.tabs.len().saturating_sub(1));
                                                                if let Some(tab) = this.visual_editor.tabs.get_mut(idx) {
                                                                    tab.content = tab.saved_content.clone();
                                                                }
                                                                this.save_visual_editor_session();
                                                                cx.notify();
                                                            }),
                                                        )
                                                )
                                            })
                                    } else {
                                        div().flex().flex_row()
                                    }
                                }
                                #[cfg(not(feature = "gui"))]
                                { div().flex().flex_row() }
                            })
                            .child({
                                #[cfg(feature = "gui")]
                                {
                                    if self.active_page_id.as_str() == "utility.shell"
                                        && !self.terminal_show_dashboard
                                    {
                                        div()
                                            .px_2()
                                            .py_0p5()
                                            .rounded(px(radius))
                                            .text_xs()
                                            .bg(action_pill_bg)
                                            .text_color(if self.active_terminal().shell_mode == ShellMode::Generic {
                                                shell_mode_generic_fg
                                            } else {
                                                shell_mode_alt_fg
                                            })
                                            .child(self.active_terminal().shell_mode.label())
                                    } else {
                                        div()
                                    }
                                }
                                #[cfg(not(feature = "gui"))]
                                { div() }
                            })
                            .child({
                                #[cfg(feature = "gui")]
                                {
                                    if self.active_page_id.as_str() == "utility.shell"
                                        && !self.terminal_show_dashboard
                                        && self.active_terminal().shell_mode == ShellMode::Generic
                                    {
                                        div()
                                            .px_2()
                                            .py_0p5()
                                            .rounded(px(radius))
                                            .text_xs()
                                            .bg(action_pill_bg)
                                            .text_color(action_pill_tc)
                                            .child(self.shell_working_directory_label())
                                    } else {
                                        div().hidden()
                                    }
                                }
                                #[cfg(not(feature = "gui"))]
                                { div() }
                            })
                            .child({
                                #[cfg(feature = "gui")]
                                {
                                    if self.active_page_id.as_str() == "utility.shell"
                                        && !self.terminal_show_dashboard
                                    {
                                        div()
                                            .px_2()
                                            .py_0p5()
                                            .rounded(px(radius))
                                            .cursor_pointer()
                                            .text_xs()
                                            .bg(action_pill_bg)
                                            .text_color(action_pill_tc)
                                            .hover(move |style| style.bg(action_pill_hover))
                                            .child("Reset")
                                            .on_mouse_down(
                                                openframe::MouseButton::Left,
                                                cx.listener(|this, _, _, cx| {
                                                    this.reset_shell_state();
                                                    cx.notify();
                                                }),
                                            )
                                    } else {
                                        div()
                                    }
                                }
                                #[cfg(not(feature = "gui"))]
                                { div() }
                            })
                            .child({
                                #[cfg(feature = "gui")]
                                {
                                    if self.active_page_id.as_str() == "utility.shell"
                                        && !self.terminal_show_dashboard
                                    {
                                        div()
                                            .px_2()
                                            .py_0p5()
                                            .rounded(px(radius))
                                            .cursor_pointer()
                                            .text_xs()
                                            .bg(action_pill_bg)
                                            .text_color(action_pill_tc)
                                            .hover(move |style| style.bg(action_pill_hover))
                                            .child("Clear")
                                            .on_mouse_down(
                                                openframe::MouseButton::Left,
                                                cx.listener(|this, _, _, cx| {
                                                    this.active_terminal_mut().shell_history.clear();
                                                    this.active_terminal_mut().shell_output_scroll.scroll_to_bottom();
                                                    cx.notify();
                                                }),
                                            )
                                    } else {
                                        div()
                                    }
                                }
                                #[cfg(not(feature = "gui"))]
                                { div() }
                            })
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(self.render_top_bar_command_bar(window, cx, is_dark, radius))
                            .children(self.top_bar_page_ids_effective().into_iter().filter_map(
                                |page_id| {
                                    if !self.is_page_visible(page_id) {
                                        return None;
                                    }
                                    let page = self.page_ref(page_id)?;
                                    let is_animated_pill = matches!(
                                        page.id(),
                                        "notification.main"
                                            | "extensions.settings"
                                            | "global.modules"
                                    );
                                    let pill_expand_alpha = is_animated_pill.then(|| {
                                        *self
                                            .pill_expand_alphas
                                            .get(page.id())
                                            .unwrap_or(&0.0)
                                    });
                                    // For the notification pill, show preview text when active.
                                    let label = if page.id() == "notification.main"
                                        && !self.notification_preview_text.is_empty()
                                    {
                                        self.notification_preview_text.clone()
                                    } else {
                                        page.title().to_string()
                                    };
                                    let content_alpha = if page.id() == "notification.main" {
                                        self.notification_content_alpha
                                    } else {
                                        1.0
                                    };
                                    let preview_bg_alpha = if page.id() == "notification.main" {
                                        self.notification_preview_bg_alpha
                                    } else {
                                        0.0
                                    };
                                    let resolved_glyph = if page.id() == "notification.main"
                                        && self.notification_unread_count > 0
                                    {
                                        "notification-on".to_string()
                                    } else {
                                        page.glyph().to_string()
                                    };
                                    let shake_off = if page.id() == "notification.main" {
                                        arcadia_core::modules::animation::shake_offset(
                                            self.notification_shake_t,
                                            3.0,
                                            3.0,
                                        )
                                    } else {
                                        0.0
                                    };
                                    Some(Self::top_bar_global_item(
                                        cx,
                                        openframe::SharedString::from(label),
                                        openframe::SharedString::from(resolved_glyph),
                                        page.id().to_string(),
                                        self.active_page_id.as_str() == page.id(),
                                        is_dark,
                                        page.accent().to_string(),
                                        glyph,
                                        pill_expand_alpha,
                                        content_alpha,
                                        preview_bg_alpha,
                                        shake_off,
                                    ))
                                },
                            )),
                    ),
            )
            .when_some(glyph_border, |d, border| {
                d.child(
                    div()
                        .w_full()
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .text_xs()
                        .text_color(border)
                        .child("─".repeat(300)),
                )
            })
    }

    pub(crate) fn render_top_bar_command_bar(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
        radius: f32,
    ) -> impl IntoElement {
        if !self.command_bar_open {
            return div();
        }

        let focused = self.command_bar_focus.is_focused(window);
        let blink = self.text_caret_blink_visible;
        let text = self.command_bar_input.clone();
        let fh = self.command_bar_focus.clone();
        let bg = theme::action_pill_bg(cx, is_dark);
        let border_c = theme::ui_accent(cx);
        let tc = theme::action_pill_text(cx, is_dark);
        let meta_c = theme::ui_subtext(cx, is_dark);

        div()
            .flex()
            .items_center()
            .px_2()
            .py_0p5()
            .rounded(px(radius))
            .bg(bg)
            .border_1()
            .border_color(border_c)
            .min_w(px(200.))
            .text_xs()
            .text_color(tc)
            .track_focus(&self.command_bar_focus)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |_, _, window, _| {
                    fh.focus(window);
                }),
            )
            .child(if text.is_empty() {
                if focused && blink {
                    div()
                        .text_color(tc)
                        .child(TEXT_INPUT_CARET_CHAR.to_string())
                } else {
                    div().text_color(meta_c).child("Internal command…")
                }
            } else {
                div().child(text_with_trailing_caret(&text, focused, blink))
            })
            .on_key_down(
                cx.listener(|this, event: &KeyDownEvent, window: &mut Window, cx| {
                    let key = event.keystroke.key.as_str();
                    let mods = event.keystroke.modifiers;
                    match key {
                        "escape" => {
                            this.command_bar_open = false;
                            this.command_bar_input.clear();
                            cx.notify();
                        }
                        "enter" => {
                            let cmd = this.command_bar_input.trim().to_string();
                            this.command_bar_open = false;
                            this.command_bar_input.clear();
                            if !cmd.is_empty() {
                                let ctx = this.execution_context();
                                let _ = modules::execute_command("shell.internal", &[&cmd], &ctx);
                            }
                            cx.notify();
                        }
                        "backspace" => {
                            this.command_bar_input.pop();
                            cx.notify();
                        }
                        _ if !mods.control && !mods.alt && !mods.platform && !mods.function => {
                            if let Some(ch) = &event.keystroke.key_char {
                                this.command_bar_input.push_str(ch);
                                cx.notify();
                            }
                        }
                        _ => {}
                    }
                    let _ = window;
                }),
            )
    }
}
