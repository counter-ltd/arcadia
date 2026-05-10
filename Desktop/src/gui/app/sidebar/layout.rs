use arcadia_core::config::modules::REMOTE_SESSION_MODULE_NAME;
use arcadia_core::navigation;
use arcadia_core::config::thin_client::ThinClientConfig;
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
                                cx.listener(|this, _, _, cx| {
                                    this.session_route_menu_open = false;
                                    this.app_menu_open = true;
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
                                    let peers = connected_approved_session_peers();
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
                                                    cx.listener(|this, _, _, cx| {
                                                        cx.stop_propagation();
                                                        this.session_route_menu_open =
                                                            !this.session_route_menu_open;
                                                        this.app_menu_open = false;
                                                        cx.notify();
                                                    }),
                                                ),
                                        )
                                        .child(if self.session_route_menu_open {
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
                                        } else {
                                            div().hidden()
                                        })
                                }
                            }),
                    )
                    .child(if self.app_menu_open {
                        div()
                            .absolute()
                            .top(px(40.))
                            .left(px(0.))
                            .w(px(112.))
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
                                        rgb(0xfca5a5)
                                    } else {
                                        rgb(0x991b1b)
                                    })
                                    .hover(move |style| {
                                        style.bg(if is_dark {
                                            rgb(0x1f2937)
                                        } else {
                                            rgb(0xfef2f2)
                                        })
                                    })
                                    .child("Quit")
                                    .on_mouse_down(
                                        openframe::MouseButton::Left,
                                        cx.listener(|this, _, _, _| {
                                            this.app_menu_open = false;
                                            #[cfg(feature = "gui")]
                                            this.run_internal_quit_command();
                                        }),
                                    ),
                            )
                    } else {
                        div().hidden()
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
