use arcadia_core::navigation;
#[cfg(feature = "gui")]
use arcadia_core::config::thin_client::ThinClientConfig;
#[cfg(feature = "gui")]
use arcadia_core::modules::lan::connected_approved_session_peers;
use openframe::{
    div, px, rgb, Context, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, WindowAppearance,
};
use openframe::prelude::FluentBuilder as _;
#[cfg(feature = "gui")]
use openframe::AnyElement;

use crate::gui::app::navigation::NavGroupRef;
use crate::gui::app::splash::SPLASH_TOTAL_MS;
use crate::gui::app::{window_controls_top_padding, ArcadiaRoot};
use crate::gui::theme::render_icon;

impl Render for ArcadiaRoot {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.splash_elapsed_ms < SPLASH_TOTAL_MS {
            self.ensure_splash_tick(window, cx);
            return self.render_splash();
        }
        #[cfg(feature = "gui")]
        self.sync_peer_remote_exec_side_effects(window, cx);
        #[cfg(feature = "gui")]
        self.ensure_shell_caret_task(window, cx);
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
            } else if is_dark {
                rgb(0x0f1115)
            } else {
                rgb(0xffffff)
            })
            .flex()
            .on_mouse_down(
                openframe::MouseButton::Left,
                cx.listener(|this, _, _, cx| {
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
                    if this.color_picker_modal.is_some() {
                        this.color_picker_modal = None;
                        changed = true;
                    }
                    if changed {
                        cx.notify();
                    }
                }),
            )
            .on_key_down(cx.listener({
                #[cfg(feature = "gui")]
                { Self::handle_global_key_down }
                #[cfg(not(feature = "gui"))]
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
                    .child(
                        if self.active_page_id.as_str() == "utility.shell"
                            || self.active_page_id.as_str() == "late.now_playing"
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
            .child(self.kill_existing_port_modal(cx, is_dark))
            .child(self.color_picker_modal(cx, is_dark))
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

        if self.terminal_context_menu_open {
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
        } else {
            div().into_any_element()
        }
    }
}
