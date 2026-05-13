use openframe::{div, px, rgb, Context, InteractiveElement, IntoElement, KeyDownEvent, MouseButton, ParentElement, Styled, Window};
use openframe::prelude::FluentBuilder as _;
use arcadia_core::config::ConfigFile as _;

use arcadia_core::modules;

use crate::gui::app::ArcadiaRoot;
#[cfg(feature = "gui")]
use crate::gui::app::ShellMode;
use crate::gui::app::text_input_caret::{TEXT_INPUT_CARET_CHAR, text_with_trailing_caret};
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
        let is_glyph    = glyph.is_some();
        let glyph_border = glyph.as_ref().map(|g| g.border);
        let title_color  = glyph.as_ref().map(|g| g.text).unwrap_or_else(|| if is_dark { rgb(0xe5e7eb) } else { rgb(0x1f2937) });
        let action_pill_bg = theme::action_pill_bg(cx, is_dark);
        let action_pill_tc = theme::action_pill_text(cx, is_dark);
        let action_pill_hover = theme::action_pill_hover_bg(cx, is_dark);
        let shell_mode_generic_bg = theme::ui_surface(cx, is_dark);
        let shell_mode_alt_bg = theme::ui_surface2(cx, is_dark);
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
                                    .text_sm()
                                    .font_weight(openframe::FontWeight::SEMIBOLD)
                                    .text_color(title_color)
                                    .child(active_page_title),
                            )
                            .child({
                                if self.active_page_id.as_str() == "ai.chat" {
                                    let label = {
                                        use arcadia_core::config::modules::{
                                            AI_EXEC_AIDER_MODULE_NAME, AI_EXEC_CLAUDE_MODULE_NAME,
                                            AI_EXEC_CODEX_MODULE_NAME, AI_EXEC_GEMINI_MODULE_NAME,
                                        };
                                        let provider = self.active_ai_provider_module.as_str();
                                        let mid = self.ai_chat_model_id.as_deref();
                                        // CLI providers: display name from detected list or module name
                                        if matches!(provider, p if p == AI_EXEC_CLAUDE_MODULE_NAME
                                            || p == AI_EXEC_CODEX_MODULE_NAME
                                            || p == AI_EXEC_GEMINI_MODULE_NAME
                                            || p == AI_EXEC_AIDER_MODULE_NAME)
                                        {
                                            self.detected_cli_providers.iter()
                                                .find(|c| c.id == provider)
                                                .map(|c| c.label.clone())
                                                .unwrap_or_else(|| provider.to_string())
                                        } else {
                                            mid.and_then(|id| {
                                                self.llama_cpp_models.iter().find(|m| m.id == id).map(|m| m.name.clone())
                                                    .or_else(|| self.ollama_models.iter().find(|m| m.id == id).map(|m| m.name.clone()))
                                                    .or_else(|| self.openai_models.iter().find(|m| m.id == id).map(|m| m.name.clone()))
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
                                                this.ai_chat_model_picker_open = !this.ai_chat_model_picker_open;
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
                                    && self.is_module_enabled(arcadia_core::config::modules::WORKSPACE_MODULE_NAME)
                                {
                                    let ws_label = self.ai_chat_workspace_id
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
                                                this.ai_chat_workspace_picker_open = !this.ai_chat_workspace_picker_open;
                                                this.ai_chat_model_picker_open = false;
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
                                if self.active_page_id.as_str() == "python.settings" {
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
                                                this.reload_python_extensions(cx);
                                                cx.notify();
                                            }),
                                        )
                                } else {
                                    div()
                                }
                            })
                            .child(if self.active_page_id.as_str() == "late.now_playing" {
                                div()
                                    .flex()
                                    .flex_row()
                                    .gap_1()
                                    .children(LATE_ROOMS.iter().map(|(label, room_id)| {
                                        let rid = *room_id;
                                        let is_active = rid == self.late_active_room;
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
                                                    this.late_active_room = rid;
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
                                        && !self.code_editor_tabs.is_empty()
                                    {
                                        let active_idx = self.active_code_editor_tab
                                            .min(self.code_editor_tabs.len().saturating_sub(1));
                                        let ws_label = self.code_editor_tabs
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
                                        let has_file = self.code_editor_tabs
                                            .get(active_idx)
                                            .map(|t| t.file_path.is_some())
                                            .unwrap_or(false);
                                        let is_dirty = self.code_editor_tabs
                                            .get(active_idx)
                                            .map(|t| t.file_path.is_some() && t.content != t.saved_content)
                                            .unwrap_or(false);
                                        let has_explorer = self.code_editor_tabs
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
                                                            this.code_editor_workspace_picker_open =
                                                                !this.code_editor_workspace_picker_open;
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
                                                                this.code_editor_explorer_open =
                                                                    !this.code_editor_explorer_open;
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
                                                                                        let idx = this.active_code_editor_tab
                                                                                            .min(this.code_editor_tabs.len().saturating_sub(1));
                                                                                        if let Some(tab) = this.code_editor_tabs.get_mut(idx) {
                                                                                            tab.saved_content = text.clone();
                                                                                            tab.content = text;
                                                                                            tab.file_path = Some(path_str);
                                                                                            tab.title = title;
                                                                                            tab.language = lang;
                                                                                            tab.cursor = 0;
                                                                                            tab.selection_anchor = None;
                                                                                            tab.highlight_dirty = true;
                                                                                        }
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
                                                                let idx = this.active_code_editor_tab
                                                                    .min(this.code_editor_tabs.len().saturating_sub(1));
                                                                if let Some(tab) = this.code_editor_tabs.get_mut(idx) {
                                                                    if let Some(path) = tab.file_path.clone() {
                                                                        let _ = std::fs::write(&path, &tab.content);
                                                                        tab.saved_content = tab.content.clone();
                                                                        cx.notify();
                                                                    }
                                                                }
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
                                                            let idx = this.active_code_editor_tab
                                                                .min(this.code_editor_tabs.len().saturating_sub(1));
                                                            let (content, suggested) = this.code_editor_tabs
                                                                .get(idx)
                                                                .map(|t| (t.content.clone(), t.title.clone()))
                                                                .unwrap_or_default();
                                                            let init_dir = this.code_editor_tabs
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
                                                                                let idx = this.active_code_editor_tab
                                                                                    .min(this.code_editor_tabs.len().saturating_sub(1));
                                                                                if let Some(tab) = this.code_editor_tabs.get_mut(idx) {
                                                                                    tab.file_path = Some(path_str);
                                                                                    tab.title = title;
                                                                                    tab.saved_content = tab.content.clone();
                                                                                    tab.language = lang;
                                                                                    tab.highlight_dirty = true;
                                                                                }
                                                                                cx.notify();
                                                                            }).ok();
                                                                        }).ok();
                                                                    }
                                                                }
                                                            }).detach();
                                                        }),
                                                    )
                                            )
                                            // Restore (only when file open and dirty)
                                            .when(is_dirty, |row| {
                                                row.child(
                                                    make_pill("Restore")
                                                        .on_mouse_down(
                                                            openframe::MouseButton::Left,
                                                            cx.listener(|this, _, _, cx| {
                                                                let idx = this.active_code_editor_tab
                                                                    .min(this.code_editor_tabs.len().saturating_sub(1));
                                                                if let Some(tab) = this.code_editor_tabs.get_mut(idx) {
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
                                    if self.active_page_id.as_str() == "utility.shell" {
                                        div()
                                            .px_2()
                                            .py_0p5()
                                            .rounded(px(radius))
                                            .text_xs()
                                            .bg(if self.active_terminal().shell_mode == ShellMode::Generic {
                                                shell_mode_generic_bg
                                            } else {
                                                shell_mode_alt_bg
                                            })
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
                                    if self.active_page_id.as_str() == "utility.shell" {
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
                                    if self.active_page_id.as_str() == "utility.shell" {
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
                            }),
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
                                    Some(Self::top_bar_global_item(
                                        cx,
                                        openframe::SharedString::from(page.title().to_string()),
                                        openframe::SharedString::from(page.glyph().to_string()),
                                        page.id().to_string(),
                                        self.active_page_id.as_str() == page.id(),
                                        is_dark,
                                        page.accent().to_string(),
                                        glyph,
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
            .on_mouse_down(MouseButton::Left, cx.listener(move |_, _, window, _| {
                fh.focus(window);
            }))
            .child(if text.is_empty() {
                if focused && blink {
                    div().text_color(tc).child(TEXT_INPUT_CARET_CHAR.to_string())
                } else {
                    div().text_color(meta_c).child("Internal command…")
                }
            } else {
                div().child(text_with_trailing_caret(&text, focused, blink))
            })
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window: &mut Window, cx| {
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
            }))
    }
}
