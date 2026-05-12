use arcadia_core::navigation;
use arcadia_core::config::ConfigFile as _;
#[cfg(feature = "gui")]
use arcadia_core::config::thin_client::ThinClientConfig;
#[cfg(feature = "gui")]
use arcadia_core::modules::lan::connected_approved_session_peers;
use openframe::{
    div, px, Context, FontWeight, InteractiveElement, IntoElement, MouseButton, ParentElement,
    Render, StatefulInteractiveElement, Styled, Window, WindowAppearance,
};
use openframe::prelude::FluentBuilder as _;
#[cfg(feature = "gui")]
use openframe::AnyElement;

use crate::gui::app::navigation::NavGroupRef;
use crate::gui::app::splash::SPLASH_TOTAL_MS;
use crate::gui::app::{window_controls_top_padding, AiChat, ArcadiaRoot, CodeEditorTab};
#[cfg(feature = "gui")]
use crate::gui::theme::render_icon;

impl ArcadiaRoot {
    pub(crate) fn render_remote_stale_banner(
        &mut self,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> openframe::Div {
        let p = crate::gui::theme::theme_palette(cx, is_dark);
        div()
            .w_full()
            .flex()
            .flex_shrink_0()
            .items_center()
            .justify_between()
            .gap_3()
            .px_3()
            .py_2()
            .bg(p.badge_info_bg)
            .border_b_1()
            .border_color(p.border)
            .child(
                div()
                    .text_sm()
                    .text_color(p.badge_info_fg)
                    .child("Host surface revision changed — reload to sync modules and navigation."),
            )
            .child(
                div()
                    .text_sm()
                    .font_weight(openframe::FontWeight::SEMIBOLD)
                    .cursor_pointer()
                    .text_color(p.accent)
                    .child("Reload")
                    .on_mouse_down(
                        openframe::MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.reload_modules();
                            this.ensure_valid_navigation_selection();
                            cx.notify();
                        }),
                    ),
            )
    }
}

impl Render for ArcadiaRoot {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.splash_elapsed_ms < SPLASH_TOTAL_MS {
            self.ensure_splash_tick(window, cx);
            return self.render_splash();
        }
        #[cfg(feature = "gui")]
        self.sync_peer_remote_exec_side_effects(window, cx);
        #[cfg(all(feature = "gui", not(target_os = "ios")))]
        crate::gui::app::shortcuts::poll_global_hotkey_events(self, window, cx);
        #[cfg(any(feature = "gui", feature = "ios-gui"))]
        self.ensure_text_caret_blink_task(window, cx);
        #[cfg(any(feature = "gui", feature = "ios-gui"))]
        self.ensure_remote_revision_poll_task(window, cx);
        self.ensure_lan_poll_task(window, cx);
        self.ensure_late_poll_task(window, cx);
        #[cfg(feature = "gui")]
        if self.terminals[self.active_terminal_id].tui_session.is_some() {
            self.sync_tui_size(window);
        }
        let is_dark = matches!(
            window.appearance(),
            WindowAppearance::Dark | WindowAppearance::VibrantDark
        );
        self.refresh_style_for_mode(is_dark, cx);
        if self.thin_client_nav_waiting_host() {
            let glyph = crate::gui::theme::active_glyph(cx);
            let ui_font_family = crate::gui::theme::active_ui_font_family(cx).map(str::to_string);
            return div()
                .relative()
                .size_full()
                .when_some(ui_font_family, |d, f| d.font_family(f))
                .bg(if let Some(g) = glyph {
                    g.bg
                } else {
                    crate::gui::theme::ui_bg(cx, is_dark)
                })
                .flex()
                .items_center()
                .justify_center()
                .child(self.render_thin_client_waiting_panel(cx, is_dark));
        }
        let visible_groups = self.visible_groups_effective();
        let fallback_group = NavGroupRef::Static(
            navigation::group_by_id(navigation::DEFAULT_GROUP_ID)
                .unwrap_or(&navigation::GROUP_DEFINITIONS[0]),
        );
        let active_group = visible_groups
            .iter()
            .find(|g| g.id() == self.active_group_id.as_str())
            .or_else(|| visible_groups.first())
            .unwrap_or(&fallback_group);
        let active_page = self
            .active_page_if_visible()
            .or_else(|| self.page_ref(self.effective_default_page()));
        let active_page_title = openframe::SharedString::from(
            active_page
                .map(|page| page.title().to_string())
                .unwrap_or_else(|| "Arcadia".to_string()),
        );
        let active_page_glyph = openframe::SharedString::from(
            active_page
                .map(|page| page.glyph().to_string())
                .unwrap_or_else(|| "tools".to_string()),
        );

        let glyph = crate::gui::theme::active_glyph(cx);
        let ui_font_family = crate::gui::theme::active_ui_font_family(cx).map(str::to_string);
        div()
            .relative()
            .size_full()
            .when_some(ui_font_family, |d, f| d.font_family(f))
            .bg(if let Some(g) = glyph {
                g.bg
            } else {
                crate::gui::theme::ui_bg(cx, is_dark)
            })
            .flex()
            .on_mouse_down(
                openframe::MouseButton::Left,
                cx.listener(|this, ev, window, cx| {
                    this.handle_shortcut_mouse_down(ev, window, cx);
                    let mut changed = false;
                    if this.app_menu_open {
                        this.app_menu_open = false;
                        changed = true;
                    }
                    if this.session_route_menu_open {
                        this.session_route_menu_open = false;
                        changed = true;
                    }
                    #[cfg(feature = "gui")]
                    if this.terminal_context_menu_open {
                        this.terminal_context_menu_open = false;
                        changed = true;
                    }
                    #[cfg(feature = "gui")]
                    if this.terminal_kill_menu.is_some() {
                        this.terminal_kill_menu = None;
                        changed = true;
                    }
                    #[cfg(feature = "gui")]
                    if this.code_editor_context_menu_open {
                        this.code_editor_context_menu_open = false;
                        changed = true;
                    }
                    #[cfg(feature = "gui")]
                    if this.code_editor_workspace_picker_open {
                        this.code_editor_workspace_picker_open = false;
                        changed = true;
                    }
                    if this.ai_context_menu_open {
                        this.ai_context_menu_open = false;
                        changed = true;
                    }
                    if this.ai_chat_menu.is_some() {
                        this.ai_chat_menu = None;
                        changed = true;
                    }
                    if this.llama_cpp_provider_menu.is_some() {
                        this.llama_cpp_provider_menu = None;
                        changed = true;
                    }
                    if this.ai_chat_model_picker_open {
                        this.ai_chat_model_picker_open = false;
                        changed = true;
                    }
                    if this.ai_chat_workspace_picker_open {
                        this.ai_chat_workspace_picker_open = false;
                        changed = true;
                    }
                    if this.color_picker_modal.is_some() {
                        this.color_picker_modal = None;
                        changed = true;
                    }
                    if changed {
                        cx.notify();
                    }
                }),
            )
            .on_mouse_up(
                openframe::MouseButton::Left,
                cx.listener(|this, ev, window, cx| {
                    this.handle_shortcut_mouse_up(ev, window, cx);
                }),
            )
            .on_mouse_move(cx.listener(|this, ev, window, cx| {
                this.handle_shortcut_pointer_move(ev, window, cx);
            }))
            .on_key_down(cx.listener({
                #[cfg(any(feature = "gui", feature = "ios-gui"))]
                { Self::handle_global_key_down }
                #[cfg(not(any(feature = "gui", feature = "ios-gui")))]
                { |_this: &mut ArcadiaRoot, _ev, _window, _cx| {} }
            }))
            .child(if self.sidebar_visible {
                self.render_sidebar(window, cx, &visible_groups, active_group, is_dark)
            } else {
                div()
            })
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .pt(if self.sidebar_visible {
                        px(0.)
                    } else {
                        window_controls_top_padding(window)
                    })
                    .child(self.render_main_top_bar(
                        cx,
                        active_page_title,
                        active_page_glyph,
                        is_dark,
                    ))
                    .child(if self.remote_route.is_some() && self.remote_surface_stale {
                        self.render_remote_stale_banner(cx, is_dark)
                    } else {
                        div()
                    })
                    .child(
                        if self.active_page_id.as_str() == "utility.shell"
                            || self.active_page_id.as_str() == "late.now_playing"
                            || self.active_page_id.as_str() == "editor.main"
                            || self.active_page_id.as_str() == "ai.chat"
                        {
                            div()
                                .flex_1()
                                .min_h_0()
                                .w_full()
                                .id("arcadia-page-full")
                                .child(self.render_active_content(window, cx, is_dark))
                        } else {
                            div()
                                .flex_1()
                                .w_full()
                                .id("arcadia-page-scroll")
                                .overflow_y_scroll()
                                .child(self.render_active_content(window, cx, is_dark))
                        },
                    ),
            )
            .child(self.requirements_modal(cx, is_dark))
            .child(self.permission_grant_modal(cx, is_dark))
            .child(self.kill_existing_port_modal(cx, is_dark))
            .child(self.color_picker_modal(cx, is_dark))
            .child(self.shortcut_create_modal(window, cx, is_dark))
            .child(self.workspace_create_modal(window, cx, is_dark))
            .child(self.llama_cpp_create_model_modal(window, cx, is_dark))
            .child(self.code_editor_close_confirm_modal(cx, is_dark))
            .child({
                #[cfg(feature = "gui")]
                { self.render_context_menu_overlay(cx, is_dark) }
                #[cfg(not(feature = "gui"))]
                { div().into_any_element() }
            })
    }
}

#[cfg(feature = "gui")]
impl ArcadiaRoot {
    fn render_context_menu_overlay(
        &self,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> AnyElement {
        let pos = self.context_menu_position;
        let border_color = crate::gui::theme::ui_border(cx, is_dark);
        let bg_color = crate::gui::theme::ui_surface(cx, is_dark);
        let text_color = crate::gui::theme::ui_text(cx, is_dark);
        let hover_bg = crate::gui::theme::ui_surface2(cx, is_dark);
        let radius = crate::gui::theme::ui_radius(cx);

        let menu_row = move |glyph: &'static str,
                             label: openframe::SharedString,
                             text_color: openframe::Rgba,
                             icon_color: openframe::Rgba| {
            div()
                .w_full()
                .px_2()
                .py_1()
                .rounded(px(radius))
                .cursor_pointer()
                .text_sm()
                .hover(move |s| s.bg(hover_bg))
                .flex()
                .gap_2()
                .items_center()
                .child(render_icon(glyph).size_4().text_color(icon_color))
                .child(div().text_color(text_color).child(label))
        };

        if self.ai_chat_model_picker_open {
            let all_models: Vec<(String, String, &'static str)> = {
                let mut v = Vec::new();
                for model in &self.llama_cpp_models {
                    v.push((model.id.clone(), model.name.clone(), model.model_kind.label()));
                }
                v
            };
            let mut picker = div()
                .absolute()
                .left(pos.x)
                .top(pos.y)
                .min_w(px(220.))
                .max_w(px(320.))
                .p_1()
                .rounded_md()
                .border_1()
                .border_color(border_color)
                .bg(bg_color)
                .occlude()
                .on_mouse_down(openframe::MouseButton::Left, cx.listener(|_, _, _, cx| {
                    cx.stop_propagation();
                }));
            if all_models.is_empty() {
                picker = picker.child(
                    div()
                        .px_2()
                        .py_1()
                        .text_xs()
                        .text_color(text_color)
                        .child("No models configured. Enable ai-provider-llama-cpp and create a model."),
                );
            } else {
                for (model_id, model_name, type_label) in all_models {
                    let is_active = self.ai_chat_model_id.as_deref() == Some(model_id.as_str());
                    let model_id2 = model_id.clone();
                    picker = picker.child(
                        div()
                            .w_full()
                            .px_2()
                            .py_1()
                            .rounded(px(crate::gui::theme::ui_radius(cx)))
                            .cursor_pointer()
                            .text_sm()
                            .hover(move |s| s.bg(hover_bg))
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_color(if is_active { crate::gui::theme::ui_accent(cx) } else { text_color })
                                    .child(model_name),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(text_color)
                                    .opacity(0.5)
                                    .child(type_label),
                            )
                            .on_mouse_down(openframe::MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                this.ai_chat_model_id = if is_active { None } else { Some(model_id2.clone()) };
                                this.ai_chat_model_picker_open = false;
                                cx.notify();
                            })),
                    );
                }
            }
            picker.into_any_element()
        } else if self.ai_chat_workspace_picker_open {
            let workspaces = arcadia_core::config::workspace::WorkspacesConfig::load_or_create()
                .map(|cfg| cfg.workspaces)
                .unwrap_or_default();
            let mut picker = div()
                .absolute()
                .left(pos.x)
                .top(pos.y)
                .min_w(px(220.))
                .max_w(px(320.))
                .p_1()
                .rounded_md()
                .border_1()
                .border_color(border_color)
                .bg(bg_color)
                .occlude()
                .on_mouse_down(openframe::MouseButton::Left, cx.listener(|_, _, _, cx| {
                    cx.stop_propagation();
                }));
            if workspaces.is_empty() {
                picker = picker.child(
                    div()
                        .px_2()
                        .py_1()
                        .text_xs()
                        .text_color(text_color)
                        .child("No workspaces configured."),
                );
            } else {
                // "None" option to clear selection
                let is_none = self.ai_chat_workspace_id.is_none();
                picker = picker.child(
                    div()
                        .w_full()
                        .px_2()
                        .py_1()
                        .rounded(px(crate::gui::theme::ui_radius(cx)))
                        .cursor_pointer()
                        .text_sm()
                        .hover(move |s| s.bg(hover_bg))
                        .text_color(if is_none { crate::gui::theme::ui_accent(cx) } else { text_color })
                        .child("None")
                        .on_mouse_down(openframe::MouseButton::Left, cx.listener(move |this, _, _, cx| {
                            this.ai_chat_workspace_id = None;
                            this.ai_chat_workspace_picker_open = false;
                            cx.notify();
                        })),
                );
                for ws in workspaces {
                    let is_active = self.ai_chat_workspace_id.as_deref() == Some(ws.id.as_str());
                    let ws_id = ws.id.clone();
                    let ws_path = ws.path.clone();
                    picker = picker.child(
                        div()
                            .w_full()
                            .px_2()
                            .py_1()
                            .rounded(px(crate::gui::theme::ui_radius(cx)))
                            .cursor_pointer()
                            .text_sm()
                            .hover(move |s| s.bg(hover_bg))
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_color(if is_active { crate::gui::theme::ui_accent(cx) } else { text_color })
                                    .child(ws.label.clone()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(text_color)
                                    .opacity(0.5)
                                    .child(ws_path),
                            )
                            .on_mouse_down(openframe::MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                this.ai_chat_workspace_id = if is_active { None } else { Some(ws_id.clone()) };
                                this.ai_chat_workspace_picker_open = false;
                                cx.notify();
                            })),
                    );
                }
            }
            picker.into_any_element()
        } else if self.code_editor_context_menu_open {
            div()
                .absolute()
                .left(pos.x)
                .top(pos.y)
                .min_w(px(172.))
                .p_1()
                .rounded_md()
                .border_1()
                .border_color(border_color)
                .bg(bg_color)
                .occlude()
                .child(
                    menu_row("code", "New Editor".into(), text_color, text_color)
                        .on_mouse_down(openframe::MouseButton::Left, cx.listener(|this, _, window, cx| {
                            let id = this.code_editor_next_id;
                            let title = format!("untitled-{id}");
                            this.code_editor_tabs.push(CodeEditorTab {
                                id,
                                title,
                                content: String::new(),
                                cursor: 0,
                                selection_anchor: None,
                                language: None,
                                hl_spans: vec![],
                                decorations: vec![],
                                highlight_dirty: true,
                                workspace_path: None,
                                file_path: None,
                                saved_content: String::new(),
                            });
                            this.active_code_editor_tab = this.code_editor_tabs.len() - 1;
                            this.code_editor_next_id += 1;
                            this.active_page_id = "editor.main".to_string();
                            this.code_editor_context_menu_open = false;
                            this.code_editor_focus.focus(window);
                            cx.notify();
                        })),
                )
                .into_any_element()
        } else if self.terminal_context_menu_open {
            div()
                .absolute()
                .left(pos.x)
                .top(pos.y)
                .min_w(px(172.))
                .p_1()
                .rounded_md()
                .border_1()
                .border_color(border_color)
                .bg(bg_color)
                .occlude()
                .child(
                    menu_row(
                        "terminal",
                        "New Terminal".into(),
                        text_color,
                        text_color,
                    )
                        .on_mouse_down(openframe::MouseButton::Left, cx.listener(|this, _, _, cx| {
                            this.create_new_terminal();
                            cx.notify();
                        })),
                )
                .into_any_element()
        } else if let Some(kill_idx) = self.terminal_kill_menu {
            let label = self.terminals.get(kill_idx)
                .map(|t| t.label.clone())
                .unwrap_or_else(|| "Terminal".to_string());
            let kill_label = format!("Kill {label}");
            let danger = crate::gui::theme::ui_danger(cx, is_dark);
            div()
                .absolute()
                .left(pos.x)
                .top(pos.y)
                .min_w(px(172.))
                .p_1()
                .rounded_md()
                .border_1()
                .border_color(border_color)
                .bg(bg_color)
                .occlude()
                .child(
                    menu_row("x", kill_label.into(), danger, danger).on_mouse_down(
                        openframe::MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.kill_terminal(kill_idx);
                            cx.notify();
                        }),
                    ),
                )
                .into_any_element()
        } else if self.session_route_menu_open {
            let sess_pos = self.session_route_menu_position;
            let peers = connected_approved_session_peers();
            div()
                .absolute()
                .left(sess_pos.x)
                .top(sess_pos.y)
                .min_w(px(200.))
                .p_1()
                .rounded_md()
                .border_1()
                .border_color(border_color)
                .bg(bg_color)
                .occlude()
                .child(
                    menu_row("home", "Local".into(), text_color, text_color).on_mouse_down(
                        openframe::MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            let _ = ThinClientConfig::set_preferred_remote_route(None);
                            this.remote_route = None;
                            this.session_route_menu_open = false;
                            this.reload_modules();
                            cx.notify();
                        }),
                    ),
                )
                .children(peers.into_iter().map(|(ip, hostname)| {
                    let route = format!("lan:{ip}");
                    let label = format!("{hostname} ({ip})");
                    menu_row("nodes", label.into(), text_color, text_color).on_mouse_down(
                        openframe::MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            let _ =
                                ThinClientConfig::set_preferred_remote_route(Some(&route));
                            this.remote_route = Some(route.clone());
                            this.session_route_menu_open = false;
                            this.reload_modules();
                            cx.notify();
                        }),
                    )
                }))
                .into_any_element()
        } else if self.code_editor_workspace_picker_open {
            let workspaces = arcadia_core::config::workspace::list_workspaces();
            let active_idx = self
                .active_code_editor_tab
                .min(self.code_editor_tabs.len().saturating_sub(1));
            let current_path = self
                .code_editor_tabs
                .get(active_idx)
                .and_then(|t| t.workspace_path.clone());
            let mut picker = div()
                .absolute()
                .left(pos.x)
                .top(pos.y)
                .min_w(px(220.))
                .max_w(px(320.))
                .p_1()
                .rounded_md()
                .border_1()
                .border_color(border_color)
                .bg(bg_color)
                .occlude()
                .on_mouse_down(openframe::MouseButton::Left, cx.listener(|_, _, _, cx| {
                    cx.stop_propagation();
                }));
            if workspaces.is_empty() {
                picker = picker.child(
                    div()
                        .px_2()
                        .py_1()
                        .text_xs()
                        .text_color(text_color)
                        .child("No workspaces registered."),
                );
            } else {
                for ws in &workspaces {
                    let is_active = current_path.as_deref() == Some(ws.path.as_str());
                    let ws_path = ws.path.clone();
                    let ws_label = ws.label.clone();
                    let display = if ws_label.is_empty() {
                        ws_path
                            .rsplit('/')
                            .next()
                            .unwrap_or(&ws_path)
                            .to_string()
                    } else {
                        ws_label.clone()
                    };
                    let ws_path2 = ws.path.clone();
                    picker = picker.child(
                        menu_row("folder", display.into(), text_color, if is_active { crate::gui::theme::ui_accent(cx) } else { text_color })
                            .on_mouse_down(openframe::MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                let idx = this.active_code_editor_tab.min(this.code_editor_tabs.len().saturating_sub(1));
                                if let Some(tab) = this.code_editor_tabs.get_mut(idx) {
                                    tab.workspace_path = if is_active { None } else { Some(ws_path2.clone()) };
                                }
                                this.code_editor_workspace_picker_open = false;
                                cx.notify();
                            })),
                    );
                }
            }
            if current_path.is_some() {
                picker = picker.child(
                    menu_row("x", "Clear Workspace".into(), text_color, text_color)
                        .on_mouse_down(openframe::MouseButton::Left, cx.listener(|this, _, _, cx| {
                            let idx = this.active_code_editor_tab.min(this.code_editor_tabs.len().saturating_sub(1));
                            if let Some(tab) = this.code_editor_tabs.get_mut(idx) {
                                tab.workspace_path = None;
                            }
                            this.code_editor_workspace_picker_open = false;
                            cx.notify();
                        })),
                );
            }
            picker.into_any_element()
        } else if let Some(menu_pos) = self.llama_cpp_provider_menu {
            div()
                .absolute()
                .left(menu_pos.x)
                .top(menu_pos.y)
                .min_w(px(172.))
                .p_1()
                .rounded_md()
                .border_1()
                .border_color(border_color)
                .bg(bg_color)
                .occlude()
                .child(
                    menu_row("modules", "Create Model".into(), text_color, text_color)
                        .on_mouse_down(openframe::MouseButton::Left, cx.listener(|this, _, _, cx| {
                            this.llama_cpp_provider_menu = None;
                            this.llama_cpp_create_draft = Some(crate::gui::app::LlamaCppModelCreateDraft {
                                name: String::new(),
                                path: String::new(),
                                mmproj_path: String::new(),
                                model_kind: arcadia_core::config::llama_cpp::LlamaCppModelKind::TextGeneration,
                                error: None,
                            });
                            cx.notify();
                        })),
                )
                .into_any_element()
        } else if self.ai_context_menu_open {
            div()
                .absolute()
                .left(pos.x)
                .top(pos.y)
                .min_w(px(172.))
                .p_1()
                .rounded_md()
                .border_1()
                .border_color(border_color)
                .bg(bg_color)
                .occlude()
                .child(
                    menu_row("message", "New Chat".into(), text_color, text_color)
                        .on_mouse_down(openframe::MouseButton::Left, cx.listener(|this, _, _, cx| {
                            let id = this.ai_next_id;
                            this.ai_chats.push(AiChat {
                                id,
                                title: format!("Chat {id}"),
                                messages: vec![],
                                input_draft: String::new(),
                                is_loading: false,
                            });
                            this.active_ai_chat_id = id;
                            this.ai_next_id += 1;
                            this.active_page_id = "ai.chat".to_string();
                            this.ai_context_menu_open = false;
                            cx.notify();
                        })),
                )
                .into_any_element()
        } else if let Some((chat_id, chat_pos)) = self.ai_chat_menu {
            let danger = crate::gui::theme::ui_danger(cx, is_dark);
            div()
                .absolute()
                .left(chat_pos.x)
                .top(chat_pos.y)
                .min_w(px(172.))
                .p_1()
                .rounded_md()
                .border_1()
                .border_color(border_color)
                .bg(bg_color)
                .occlude()
                .child(
                    menu_row("x", "Close Chat".into(), danger, danger).on_mouse_down(
                        openframe::MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.ai_chat_menu = None;
                            this.ai_chats.retain(|c| c.id != chat_id);
                            if !this.ai_chats.is_empty() {
                                let last_id = this.ai_chats.last().map(|c| c.id).unwrap_or(0);
                                if !this.ai_chats.iter().any(|c| c.id == this.active_ai_chat_id) {
                                    this.active_ai_chat_id = last_id;
                                }
                            }
                            cx.notify();
                        }),
                    ),
                )
                .into_any_element()
        } else if self.app_menu_open {
            let danger = crate::gui::theme::ui_danger(cx, is_dark);
            div()
                .absolute()
                .left(pos.x)
                .top(pos.y)
                .min_w(px(176.))
                .p_1()
                .rounded_md()
                .border_1()
                .border_color(border_color)
                .bg(bg_color)
                .occlude()
                .child(
                    menu_row("logs", "Logs".into(), text_color, text_color).on_mouse_down(
                        openframe::MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.active_page_id = navigation::LOGS_PAGE_ID.to_string();
                            this.app_menu_open = false;
                            cx.notify();
                        }),
                    ),
                )
                .child(
                    menu_row("log-out", "Quit".into(), danger, danger).on_mouse_down(
                        openframe::MouseButton::Left,
                        cx.listener(|this, _, _, _| {
                            this.app_menu_open = false;
                            this.run_internal_quit_command();
                        }),
                    ),
                )
                .into_any_element()
        } else if let Some((tab_idx, tab_pos)) = self.code_editor_tab_menu {
            let danger = crate::gui::theme::ui_danger(cx, is_dark);
            div()
                .absolute()
                .left(tab_pos.x)
                .top(tab_pos.y)
                .min_w(px(172.))
                .p_1()
                .rounded_md()
                .border_1()
                .border_color(border_color)
                .bg(bg_color)
                .occlude()
                .child(
                    menu_row("x", "Close Editor".into(), danger, danger).on_mouse_down(
                        openframe::MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.code_editor_tab_menu = None;
                            let is_dirty = this
                                .code_editor_tabs
                                .get(tab_idx)
                                .map(|t| t.file_path.is_some() && t.content != t.saved_content)
                                .unwrap_or(false);
                            if is_dirty {
                                this.code_editor_close_confirm = Some(tab_idx);
                            } else {
                                if tab_idx < this.code_editor_tabs.len() {
                                    this.code_editor_tabs.remove(tab_idx);
                                    if this.code_editor_tabs.is_empty() {
                                        this.code_editor_show_dashboard = true;
                                    } else {
                                        this.active_code_editor_tab = this
                                            .active_code_editor_tab
                                            .min(this.code_editor_tabs.len() - 1);
                                    }
                                }
                            }
                            cx.notify();
                        }),
                    ),
                )
                .into_any_element()
        } else {
            div().into_any_element()
        }
    }
}

impl ArcadiaRoot {
    pub fn code_editor_close_confirm_modal(
        &self,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> impl IntoElement {
        let Some(tab_idx) = self.code_editor_close_confirm else {
            return div();
        };
        let filename = self
            .code_editor_tabs
            .get(tab_idx)
            .map(|t| t.title.clone())
            .unwrap_or_else(|| "this file".to_string());
        let has_path = self
            .code_editor_tabs
            .get(tab_idx)
            .and_then(|t| t.file_path.clone())
            .is_some();
        let radius = crate::gui::theme::ui_radius(cx);

        div()
            .absolute()
            .top_0()
            .left_0()
            .right_0()
            .bottom_0()
            .child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .bottom_0()
                    .bg(crate::gui::theme::OVERLAY_MASK_COLOR)
                    .opacity(0.4)
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.code_editor_close_confirm = None;
                            cx.notify();
                        }),
                    ),
            )
            .child(
                div()
                    .size_full()
                    .flex()
                    .justify_center()
                    .items_center()
                    .child(
                        div()
                            .w_96()
                            .p_5()
                            .rounded(px(radius))
                            .bg(crate::gui::theme::ui_surface(cx, is_dark))
                            .border_1()
                            .border_color(crate::gui::theme::ui_border(cx, is_dark))
                            .flex()
                            .flex_col()
                            .gap_4()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(crate::gui::theme::ui_text(cx, is_dark))
                                    .child("Unsaved Changes"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(crate::gui::theme::ui_text(cx, is_dark))
                                    .child(format!("\"{filename}\" has unsaved changes.")),
                            )
                            .child(
                                div()
                                    .flex()
                                    .gap_3()
                                    .justify_end()
                                    // Cancel
                                    .child(
                                        div()
                                            .px_3()
                                            .py_1()
                                            .rounded(px(radius.min(8.0)))
                                            .border_1()
                                            .border_color(crate::gui::theme::ui_border(cx, is_dark))
                                            .text_sm()
                                            .text_color(crate::gui::theme::ui_text(cx, is_dark))
                                            .cursor_pointer()
                                            .child("Cancel")
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(|this, _, _, cx| {
                                                    this.code_editor_close_confirm = None;
                                                    cx.notify();
                                                }),
                                            ),
                                    )
                                    // Discard
                                    .child(
                                        div()
                                            .px_3()
                                            .py_1()
                                            .rounded(px(radius.min(8.0)))
                                            .border_1()
                                            .border_color(crate::gui::theme::ui_danger(cx, is_dark))
                                            .text_sm()
                                            .text_color(crate::gui::theme::ui_danger(cx, is_dark))
                                            .cursor_pointer()
                                            .child("Discard")
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(move |this, _, _, cx| {
                                                    this.code_editor_close_confirm = None;
                                                    if tab_idx < this.code_editor_tabs.len() {
                                                        this.code_editor_tabs.remove(tab_idx);
                                                        if this.code_editor_tabs.is_empty() {
                                                            this.code_editor_show_dashboard = true;
                                                        } else {
                                                            this.active_code_editor_tab = this
                                                                .active_code_editor_tab
                                                                .min(this.code_editor_tabs.len() - 1);
                                                        }
                                                    }
                                                    cx.notify();
                                                }),
                                            ),
                                    )
                                    // Save (only when file path exists)
                                    .when(has_path, |row| {
                                        row.child(
                                            div()
                                                .px_3()
                                                .py_1()
                                                .rounded(px(radius.min(8.0)))
                                                .bg(crate::gui::theme::ui_accent(cx))
                                                .text_sm()
                                                .text_color(crate::gui::theme::ui_accent_fg(cx))
                                                .cursor_pointer()
                                                .child("Save")
                                                .on_mouse_down(
                                                    MouseButton::Left,
                                                    cx.listener(move |this, _, _, cx| {
                                                        this.code_editor_close_confirm = None;
                                                        if let Some(tab) = this.code_editor_tabs.get_mut(tab_idx) {
                                                            if let Some(path) = tab.file_path.clone() {
                                                                let _ = std::fs::write(&path, &tab.content);
                                                                tab.saved_content = tab.content.clone();
                                                            }
                                                        }
                                                        if tab_idx < this.code_editor_tabs.len() {
                                                            this.code_editor_tabs.remove(tab_idx);
                                                            if this.code_editor_tabs.is_empty() {
                                                                this.code_editor_show_dashboard = true;
                                                            } else {
                                                                this.active_code_editor_tab = this
                                                                    .active_code_editor_tab
                                                                    .min(this.code_editor_tabs.len() - 1);
                                                            }
                                                        }
                                                        cx.notify();
                                                    }),
                                                ),
                                        )
                                    }),
                            ),
                    ),
            )
    }
}
