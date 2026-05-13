use arcadia_core::config::modules::REMOTE_SESSION_MODULE_NAME;
use arcadia_core::navigation;
#[cfg(not(feature = "gui"))]
use arcadia_core::config::thin_client::ThinClientConfig;
#[cfg(not(feature = "gui"))]
use arcadia_core::modules::lan::connected_approved_session_peers;
use openframe::{
    div, img, px, rgb, AnyElement, Context, Div, IntoElement, InteractiveElement, ParentElement,
    StatefulInteractiveElement, Styled, Window,
};
use openframe::prelude::FluentBuilder as _;

use crate::gui::app::navigation::NavGroupRef;
use crate::gui::app::{window_controls_top_padding, ArcadiaRoot};
use crate::gui::theme::{self};

impl ArcadiaRoot {
    pub(crate) fn render_sidebar(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
        visible_groups: &[NavGroupRef<'_>],
        active_group: &NavGroupRef<'_>,
        is_dark: bool,
    ) -> Div {
        let top_inset = window_controls_top_padding(window);
        // Pad content below traffic lights; outer column keeps full-height bg + border into titlebar.
        let content_top_pad = top_inset + px(12.);
        let glyph = theme::glyph_snapshot(cx);
        let sidebar_bg   = glyph.as_ref().map(|g| g.surface).unwrap_or_else(|| if is_dark { rgb(0x171b22) } else { rgb(0xf6f7fb) });
        let sidebar_border = glyph.as_ref().map(|g| g.accent).unwrap_or_else(|| if is_dark { rgb(0x2a3340) } else { rgb(0xe6e8ef) });
        let title_text   = glyph.as_ref().map(|g| g.text).unwrap_or_else(|| if is_dark { rgb(0xe5e7eb) } else { rgb(0x111827) });
        let is_glyph     = glyph.is_some();
        let radius       = glyph.as_ref().map(|g| g.border_radius).unwrap_or(8.0);
        div()
            .h_full()
            .w_64()
            .flex()
            .flex_col()
            .bg(sidebar_bg)
            .border_r_1()
            .border_color(sidebar_border)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .relative()
                    .px_5()
                    .pt(content_top_pad)
                    .pb_6()
                    .gap_4()
                    .child(
                        div()
                            .relative()
                            .flex()
                            .items_center()
                            .gap_2()
                            .on_mouse_down(
                                openframe::MouseButton::Right,
                                cx.listener(|this, event: &openframe::MouseDownEvent, _, cx| {
                                    this.session_route_menu_open = false;
                                    this.app_menu_open = true;
                                    #[cfg(feature = "gui")]
                                    {
                                        this.terminal_context_menu_open = false;
                                        this.terminal_kill_menu = None;
                                        this.context_menu_position = event.position;
                                    }
                                    #[cfg(not(feature = "gui"))]
                                    let _ = event;
                                    cx.notify();
                                }),
                            )
                            .child(if is_glyph {
                                img("icons/app-icon-tui.svg")
                                    .size_8()
                                    .text_color(title_text)
                                    .into_any_element()
                            } else {
                                img("icons/app-icon.png")
                                    .size_8()
                                    .rounded_sm()
                                    .into_any_element()
                            })
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(openframe::FontWeight::BOLD)
                                    .text_color(title_text)
                                    .child("Arcadia"),
                            )
                            .child({
                                let session_label = self
                                    .remote_route
                                    .as_deref()
                                    .and_then(|r| r.strip_prefix("lan:"))
                                    .unwrap_or("local")
                                    .to_string();
                                if !self.is_module_enabled(REMOTE_SESSION_MODULE_NAME) {
                                    div()
                                        .ml_2()
                                        .px_2()
                                        .py_0p5()
                                        .when(!is_glyph, |d| d.rounded_full())
                                        .rounded(px(radius))
                                        .border_1()
                                        .border_color(theme::sidebar_session_chip_border(is_dark))
                                        .bg(theme::sidebar_session_chip_bg(is_dark))
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(openframe::FontWeight::MEDIUM)
                                                .text_color(theme::sidebar_session_chip_text(is_dark))
                                                .child("local"),
                                        )
                                } else {
                                    div()
                                        .relative()
                                        .ml_2()
                                        .child(
                                            div()
                                                .px_2()
                                                .py_0p5()
                                                .when(!is_glyph, |d| d.rounded_full())
                                                .rounded(px(radius))
                                                .border_1()
                                                .border_color(theme::sidebar_session_chip_border(
                                                    is_dark,
                                                ))
                                                .bg(theme::sidebar_session_chip_bg(is_dark))
                                                .hover(move |style| {
                                                    style.bg(theme::sidebar_session_chip_hover_bg(
                                                        is_dark,
                                                    ))
                                                })
                                                .cursor_pointer()
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .font_weight(openframe::FontWeight::MEDIUM)
                                                        .text_color(
                                                            theme::sidebar_session_chip_text(is_dark),
                                                        )
                                                        .child(session_label),
                                                )
                                                .on_mouse_down(
                                                    openframe::MouseButton::Left,
                                                    cx.listener(|this, event: &openframe::MouseDownEvent, _, cx| {
                                                        cx.stop_propagation();
                                                        let opening = !this.session_route_menu_open;
                                                        this.session_route_menu_open = opening;
                                                        #[cfg(feature = "gui")]
                                                        if opening {
                                                            this.terminal_context_menu_open = false;
                                                            this.terminal_kill_menu = None;
                                                            this.session_route_menu_position =
                                                                event.position;
                                                        }
                                                        #[cfg(not(feature = "gui"))]
                                                        let _ = event;
                                                        this.app_menu_open = false;
                                                        cx.notify();
                                                    }),
                                                ),
                                        )
                                        .child({
                                            #[cfg(feature = "gui")]
                                            {
                                                div().hidden()
                                            }
                                            #[cfg(not(feature = "gui"))]
                                            {
                                                let peers = connected_approved_session_peers();
                                                if self.session_route_menu_open {
                                                    div()
                                                        .absolute()
                                                        .top(px(30.))
                                                        .left(px(0.))
                                                        .min_w(px(200.))
                                                        .p_1()
                                                        .rounded(px(radius))
                                                        .border_1()
                                                        .border_color(if is_dark {
                                                            rgb(0x374151)
                                                        } else {
                                                            rgb(0xd1d5db)
                                                        })
                                                        .bg(if is_dark {
                                                            rgb(0x111827)
                                                        } else {
                                                            rgb(0xffffff)
                                                        })
                                                        .child(
                                                            div()
                                                                .w_full()
                                                                .px_2()
                                                                .py_1()
                                                                .rounded(px(radius))
                                                                .cursor_pointer()
                                                                .text_sm()
                                                                .text_color(if is_dark {
                                                                    rgb(0xe5e7eb)
                                                                } else {
                                                                    rgb(0x1f2937)
                                                                })
                                                                .hover(move |style| {
                                                                    style.bg(if is_dark {
                                                                        rgb(0x1f2937)
                                                                    } else {
                                                                        rgb(0xf3f4f6)
                                                                    })
                                                                })
                                                                .child("Local")
                                                                .on_mouse_down(
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
                                                        .children(peers.into_iter().map(
                                                            |(ip, hostname)| {
                                                                let route = format!("lan:{ip}");
                                                                let label = format!("{hostname} ({ip})");
                                                                div()
                                                                    .w_full()
                                                                    .px_2()
                                                                    .py_1()
                                                                    .rounded(px(radius))
                                                                    .cursor_pointer()
                                                                    .text_sm()
                                                                    .text_color(if is_dark {
                                                                        rgb(0xe5e7eb)
                                                                    } else {
                                                                        rgb(0x1f2937)
                                                                    })
                                                                    .hover(move |style| {
                                                                        style.bg(if is_dark {
                                                                            rgb(0x1f2937)
                                                                        } else {
                                                                            rgb(0xf3f4f6)
                                                                        })
                                                                    })
                                                                    .child(label)
                                                                    .on_mouse_down(
                                                                        openframe::MouseButton::Left,
                                                                        cx.listener(move |this, _, _, cx| {
                                                                            let _ = ThinClientConfig::set_preferred_remote_route(Some(&route));
                                                                            this.remote_route =
                                                                                Some(route.clone());
                                                                            this.session_route_menu_open =
                                                                                false;
                                                                            this.reload_modules();
                                                                            cx.notify();
                                                                        }),
                                                                    )
                                                            },
                                                        ))
                                                        .into_any_element()
                                                } else {
                                                    div().hidden().into_any_element()
                                                }
                                            }
                                        })
                                }
                            }),
                    )
                    .child({
                        #[cfg(feature = "gui")]
                        {
                            div().hidden()
                        }
                        #[cfg(not(feature = "gui"))]
                        {
                            if self.app_menu_open {
                                div()
                                    .absolute()
                                    .top(px(40.))
                                    .left(px(0.))
                                    .min_w(px(128.))
                                    .p_1()
                                    .rounded(px(radius))
                                    .border_1()
                                    .border_color(if is_dark {
                                        rgb(0x374151)
                                    } else {
                                        rgb(0xd1d5db)
                                    })
                                    .bg(if is_dark {
                                        rgb(0x111827)
                                    } else {
                                        rgb(0xffffff)
                                    })
                                    .child({
                                        let logs_fg = if is_dark {
                                            rgb(0xe5e7eb)
                                        } else {
                                            rgb(0x1f2937)
                                        };
                                        let logs_hover = if is_dark {
                                            rgb(0x1f2937)
                                        } else {
                                            rgb(0xf3f4f6)
                                        };
                                        div()
                                            .w_full()
                                            .px_2()
                                            .py_1()
                                            .rounded(px(radius))
                                            .cursor_pointer()
                                            .text_sm()
                                            .hover(move |style| style.bg(logs_hover))
                                            .flex()
                                            .gap_2()
                                            .items_center()
                                            .child(
                                                theme::render_icon("logs")
                                                    .size_4()
                                                    .text_color(logs_fg),
                                            )
                                            .child(div().text_color(logs_fg).child("Logs"))
                                            .on_mouse_down(
                                                openframe::MouseButton::Left,
                                                cx.listener(|this, _, _, cx| {
                                                    this.active_page_id =
                                                        arcadia_core::navigation::LOGS_PAGE_ID
                                                            .to_string();
                                                    this.app_menu_open = false;
                                                    cx.notify();
                                                }),
                                            )
                                    })
                                    .child({
                                        let quit_fg = if is_dark {
                                            rgb(0xfca5a5)
                                        } else {
                                            rgb(0x991b1b)
                                        };
                                        let quit_hover = if is_dark {
                                            rgb(0x1f2937)
                                        } else {
                                            rgb(0xfef2f2)
                                        };
                                        div()
                                            .w_full()
                                            .px_2()
                                            .py_1()
                                            .rounded(px(radius))
                                            .cursor_pointer()
                                            .text_sm()
                                            .hover(move |style| style.bg(quit_hover))
                                            .flex()
                                            .gap_2()
                                            .items_center()
                                            .child(
                                                theme::render_icon("log-out")
                                                    .size_4()
                                                    .text_color(quit_fg),
                                            )
                                            .child(div().text_color(quit_fg).child("Quit"))
                                            .on_mouse_down(
                                                openframe::MouseButton::Left,
                                                cx.listener(|this, _, _, _| {
                                                    this.app_menu_open = false;
                                                    #[cfg(feature = "gui")]
                                                    this.run_internal_quit_command();
                                                }),
                                            )
                                    })
                                    .into_any_element()
                            } else {
                                div().hidden().into_any_element()
                            }
                        }
                    })
                    .child(
                        div()
                            .id("sidebar-group-tabs")
                            .w_full()
                            .overflow_x_scroll()
                            .child(
                                div()
                                    .flex()
                                    .gap_2()
                                    .w_full()
                                    .justify_center()
                                    .items_start()
                                    .children(visible_groups.iter().copied().map(|group| {
                                        Self::sidebar_group_item(
                                            cx,
                                            openframe::SharedString::from(group.label().to_string()),
                                            openframe::SharedString::from(group.glyph().to_string()),
                                            group.id().to_string(),
                                            self.active_group_id == group.id(),
                                            is_dark,
                                            group.accent().to_string(),
                                            glyph,
                                        )
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .id("sidebar-subtabs")
                            .flex_1()
                            .overflow_y_scroll()
                            .relative()
                            .child({
                                let mut items: Vec<AnyElement> = Vec::new();
                                for page_id in active_group.page_ids() {
                                    if !self.is_page_visible(page_id) {
                                        continue;
                                    }
                                    let Some(page) = self.page_ref(page_id) else {
                                        continue;
                                    };
                                    let is_page_active = self.active_page_id == page.id();
                                    items.push(
                                        Self::sidebar_item(
                                            cx,
                                            openframe::SharedString::from(page.title().to_string()),
                                            openframe::SharedString::from(page.glyph().to_string()),
                                            page.id().to_string(),
                                            is_page_active,
                                            is_dark,
                                            page.accent().to_string(),
                                            glyph,
                                        )
                                        .into_any_element(),
                                    );
                                    #[cfg(feature = "gui")]
                                    if page_id == "utility.shell" && self.terminals.len() > 1 {
                                        for i in 0..self.terminals.len() {
                                            let label = self.terminals[i].label.clone();
                                            let is_sub_active = is_page_active
                                                && self.active_terminal_id == i;
                                            items.push(
                                                Self::sidebar_sub_item(
                                                    cx,
                                                    openframe::SharedString::from(label),
                                                    i,
                                                    is_sub_active,
                                                    is_dark,
                                                    glyph,
                                                )
                                                .into_any_element(),
                                            );
                                        }
                                    }
                                    #[cfg(feature = "gui")]
                                    if page_id == "editor.main" {
                                        for i in 0..self.code_editor_tabs.len() {
                                            let label = self.code_editor_tabs[i].title.clone();
                                            let is_sub_active = is_page_active
                                                && self.active_code_editor_tab == i
                                                && !self.code_editor_show_dashboard;
                                            let ws_label = self.code_editor_tabs[i]
                                                .workspace_path
                                                .as_deref()
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
                                            items.push(
                                                Self::sidebar_code_editor_sub_item(
                                                    cx,
                                                    openframe::SharedString::from(label),
                                                    ws_label,
                                                    i,
                                                    is_sub_active,
                                                    is_dark,
                                                    glyph,
                                                )
                                                .into_any_element(),
                                            );
                                        }
                                        let dim = if is_dark { rgb(0x4a5568) } else { rgb(0x9ca3af) };
                                        let dim_hover = if is_dark { rgb(0x718096) } else { rgb(0x6b7280) };
                                        items.push(
                                            div()
                                                .ml_7()
                                                .pl_2()
                                                .py_1()
                                                .text_xs()
                                                .text_color(dim)
                                                .cursor_pointer()
                                                .hover(|d| d.text_color(dim_hover))
                                                .on_mouse_down(openframe::MouseButton::Left, cx.listener(|this, _, window, cx| {
                                                    let id = this.code_editor_next_id;
                                                    let title = format!("untitled-{id}");
                                                    this.code_editor_tabs.push(crate::gui::app::CodeEditorTab {
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
                                                    this.code_editor_focus.focus(window);
                                                    cx.notify();
                                                }))
                                                .child("Click to Create")
                                                .into_any_element(),
                                        );
                                    }
                                    if page_id == "ai.models" {
                                        let providers = arcadia_core::modules::ai::enabled_ai_providers(&self.module_rows);
                                        for provider in providers {
                                            let module_name = provider.module_name.to_string();
                                            let label = provider.display_name.to_string();
                                            let is_sub_active = is_page_active
                                                && self.active_ai_provider_module == provider.module_name
                                                && self.active_llama_cpp_model_id.is_none();
                                            items.push(
                                                Self::sidebar_ai_provider_sub_item(
                                                    cx,
                                                    openframe::SharedString::from(label),
                                                    module_name.clone(),
                                                    is_sub_active,
                                                    is_dark,
                                                    glyph,
                                                )
                                                .into_any_element(),
                                            );
                                            if module_name == arcadia_core::config::modules::AI_LLAMA_CPP_MODULE_NAME {
                                                for model in &self.llama_cpp_models {
                                                    let model_id = model.id.clone();
                                                    let model_label = model.name.clone();
                                                    let is_model_active = is_page_active
                                                        && self.active_llama_cpp_model_id.as_deref() == Some(model.id.as_str());
                                                    items.push(
                                                        Self::sidebar_llama_cpp_model_sub_item(
                                                            cx,
                                                            openframe::SharedString::from(model_label),
                                                            model_id,
                                                            is_model_active,
                                                            is_dark,
                                                            glyph,
                                                        )
                                                        .into_any_element(),
                                                    );
                                                }
                                            }
                                        }
                                    }
                                    if page_id == "ai.chat" {
                                        for chat in &self.ai_chats {
                                            let label = chat.title.clone();
                                            let chat_id = chat.id;
                                            let is_sub_active = is_page_active
                                                && self.active_ai_chat_id == chat_id;
                                            items.push(
                                                Self::sidebar_ai_chat_sub_item(
                                                    cx,
                                                    openframe::SharedString::from(label),
                                                    chat_id,
                                                    is_sub_active,
                                                    is_dark,
                                                    glyph,
                                                )
                                                .into_any_element(),
                                            );
                                        }
                                        let dim = if is_dark { rgb(0x4a5568) } else { rgb(0x9ca3af) };
                                        let dim_hover = if is_dark { rgb(0x718096) } else { rgb(0x6b7280) };
                                        items.push(
                                            div()
                                                .ml_7()
                                                .pl_2()
                                                .py_1()
                                                .text_xs()
                                                .text_color(dim)
                                                .cursor_pointer()
                                                .hover(|d| d.text_color(dim_hover))
                                                .on_mouse_down(openframe::MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                    let id = this.ai_next_id;
                                                    this.ai_chats.push(crate::gui::app::AiChat {
                                                        id,
                                                        title: format!("Chat {id}"),
                                                        messages: vec![],
                                                        input_draft: String::new(),
                                                        is_loading: false,
                                                    });
                                                    this.active_ai_chat_id = id;
                                                    this.ai_next_id += 1;
                                                    this.active_page_id = "ai.chat".to_string();
                                                    cx.notify();
                                                }))
                                                .child("Click to Create")
                                                .into_any_element(),
                                        );
                                    }
                                }
                                div().flex().flex_col().gap_1().children(items)
                            })
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_0p5()
                            .children({
                                self.global_page_ids_effective()
                                    .into_iter()
                                    .filter_map(|page_id| {
                                        let page = self.page_ref(page_id)?;
                                        if page_id == navigation::SETTINGS_HUB_ROOT_PAGE_ID {
                                            Some(
                                                self.sidebar_settings_hub_section(cx, page, is_dark, glyph)
                                                    .into_any_element(),
                                            )
                                        } else {
                                            Some(
                                                Self::sidebar_global_item(
                                                    cx,
                                                    openframe::SharedString::from(
                                                        page.title().to_string(),
                                                    ),
                                                    openframe::SharedString::from(
                                                        page.glyph().to_string(),
                                                    ),
                                                    page.id().to_string(),
                                                    self.active_page_id == page.id(),
                                                    is_dark,
                                                    page.accent().to_string(),
                                                    glyph,
                                                )
                                                .into_any_element(),
                                            )
                                        }
                                    })
                                    .collect::<Vec<AnyElement>>()
                            }),
                    ),
            )
    }
}
