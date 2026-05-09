use arcadia_core::navigation;
use openframe::{div, rgb, Context, InteractiveElement, IntoElement, ParentElement, Styled};

use crate::gui::app::navigation::NavPageRef;
use crate::gui::app::ArcadiaRoot;
use crate::gui::theme::{self, render_icon};

impl ArcadiaRoot {
    pub fn sidebar_toggle_button(
        cx: &mut Context<Self>,
        page_glyph: &str,
        is_dark: bool,
    ) -> impl IntoElement {
        div()
            .w_8()
            .h_8()
            .rounded_md()
            .cursor_pointer()
            .bg(if is_dark {
                rgb(0x1f2937)
            } else {
                rgb(0xf3f4f6)
            })
            .text_color(if is_dark {
                rgb(0xe5e7eb)
            } else {
                rgb(0x1f2937)
            })
            .hover(move |style| {
                style.bg(if is_dark {
                    rgb(0x243246)
                } else {
                    rgb(0xe5e7eb)
                })
            })
            .flex()
            .items_center()
            .justify_center()
            .child(render_icon(page_glyph).size_4().text_color(if is_dark {
                rgb(0xe5e7eb)
            } else {
                rgb(0x1f2937)
            }))
            .on_mouse_down(
                openframe::MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.sidebar_visible = !this.sidebar_visible;
                    cx.notify();
                }),
            )
    }

    pub fn sidebar_group_item(
        cx: &mut Context<Self>,
        label: openframe::SharedString,
        system_image: openframe::SharedString,
        group_id: String,
        is_active: bool,
        is_dark: bool,
        accent: String,
    ) -> impl IntoElement {
        let pal = theme::nav_accent_palette(accent.as_str(), is_dark);
        let icon_color = if is_active {
            pal.icon_active
        } else {
            pal.icon_idle
        };
        let label_color = if is_active {
            pal.icon_active
        } else {
            theme::sidebar_nav_idle_foreground(is_dark)
        };
        div()
            .w_16()
            .h_16()
            .flex()
            .items_center()
            .justify_center()
            .rounded_md()
            .cursor_pointer()
            .text_xs()
            .font_weight(if is_active {
                openframe::FontWeight::BOLD
            } else {
                openframe::FontWeight::NORMAL
            })
            .bg(if is_active {
                pal.row_selected
            } else if is_dark {
                rgb(0x171b22)
            } else {
                rgb(0xf6f7fb)
            })
            .text_color(label_color)
            .hover(move |style| {
                style.bg(if is_active {
                    pal.row_hover
                } else if is_dark {
                    rgb(0x243246)
                } else {
                    rgb(0xeef2ff)
                })
            })
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_1()
                    .text_center()
                    .child(
                        render_icon(system_image.as_ref()).size_5().text_color(icon_color),
                    )
                    .child(div().child(label)),
            )
            .on_mouse_down(
                openframe::MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.active_group_id = group_id.clone();
                    if let Some(group) = this.effective_group(group_id.as_str()) {
                        if let Some(first_page_id) = group
                            .page_ids()
                            .into_iter()
                            .find(|pid| this.is_page_visible(pid))
                        {
                            this.active_page_id = first_page_id.to_string();
                        }
                    }
                    cx.notify();
                }),
            )
    }

    /// Compact top-bar control (same visual weight as neutral badges). Use for header actions only;
    /// the sidebar still uses [`Self::sidebar_global_item`].
    pub fn top_bar_global_item(
        cx: &mut Context<Self>,
        label: openframe::SharedString,
        system_image: openframe::SharedString,
        page_id: String,
        is_active: bool,
        is_dark: bool,
        accent: String,
    ) -> impl IntoElement {
        let pal = theme::nav_accent_palette(accent.as_str(), is_dark);
        let icon_color = if is_active {
            pal.icon_active
        } else {
            pal.icon_idle
        };
        let label_color = if is_active {
            pal.icon_active
        } else {
            theme::sidebar_nav_idle_foreground(is_dark)
        };
        div()
            .px_2()
            .py_0p5()
            .rounded_md()
            .cursor_pointer()
            .text_xs()
            .font_weight(if is_active {
                openframe::FontWeight::SEMIBOLD
            } else {
                openframe::FontWeight::NORMAL
            })
            .bg(if is_active {
                pal.row_selected
            } else {
                theme::top_bar_pill_bg(is_dark)
            })
            .text_color(label_color)
            .hover(move |style| {
                style.bg(if is_active {
                    pal.row_hover
                } else {
                    theme::top_bar_pill_hover_bg(is_dark)
                })
            })
            .child(
                div()
                    .flex()
                    .gap_1()
                    .items_center()
                    .child(render_icon(system_image.as_ref()).size_4().text_color(icon_color))
                    .child(div().child(label)),
            )
            .on_mouse_down(
                openframe::MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.active_page_id = page_id.clone();
                    if page_id == "global.modules" {
                        this.reload_modules();
                    }
                    this.sync_settings_hub_expanded_from_active_page();
                    cx.notify();
                }),
            )
    }

    pub fn sidebar_global_item(
        cx: &mut Context<Self>,
        label: openframe::SharedString,
        system_image: openframe::SharedString,
        page_id: String,
        is_active: bool,
        is_dark: bool,
        accent: String,
    ) -> impl IntoElement {
        let pal = theme::nav_accent_palette(accent.as_str(), is_dark);
        let icon_color = if is_active {
            pal.icon_active
        } else {
            pal.icon_idle
        };
        let label_color = if is_active {
            pal.icon_active
        } else {
            theme::sidebar_nav_idle_foreground(is_dark)
        };
        div()
            .px_3()
            .py_1()
            .rounded_md()
            .cursor_pointer()
            .text_sm()
            .font_weight(if is_active {
                openframe::FontWeight::BOLD
            } else {
                openframe::FontWeight::NORMAL
            })
            .bg(if is_active {
                pal.row_selected
            } else if is_dark {
                rgb(0x171b22)
            } else {
                rgb(0xf6f7fb)
            })
            .text_color(label_color)
            .hover(move |style| {
                style.bg(if is_active {
                    pal.row_hover
                } else if is_dark {
                    rgb(0x243246)
                } else {
                    rgb(0xeef2ff)
                })
            })
            .child(
                div()
                    .flex()
                    .gap_2()
                    .items_center()
                    .child(render_icon(system_image.as_ref()).size_4().text_color(icon_color))
                    .child(div().child(label)),
            )
            .on_mouse_down(
                openframe::MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.active_page_id = page_id.clone();
                    if page_id == "global.modules" {
                        this.reload_modules();
                    }
                    this.sync_settings_hub_expanded_from_active_page();
                    cx.notify();
                }),
            )
    }

    /// Sidebar "Settings" hub: header row (registry [`SETTINGS_HUB_ROOT_PAGE_ID`]) + nested pages from
    /// [`ArcadiaRoot::settings_hub_page_ids_effective`].
    pub fn sidebar_settings_hub_section(
        &self,
        cx: &mut Context<Self>,
        hub_page: NavPageRef<'_>,
        is_dark: bool,
    ) -> impl IntoElement {
        let pal = theme::nav_accent_palette(hub_page.accent(), is_dark);
        let hub_active = self.active_page_id.as_str() == navigation::SETTINGS_HUB_ROOT_PAGE_ID
            || self
                .settings_hub_page_ids_effective()
                .iter()
                .any(|pid| *pid == self.active_page_id.as_str());
        let icon_color = if hub_active {
            pal.icon_active
        } else {
            pal.icon_idle
        };
        let label_color = if hub_active {
            pal.icon_active
        } else {
            theme::sidebar_nav_idle_foreground(is_dark)
        };
        let chevron = if self.settings_hub_expanded { "▼" } else { "▶" };
        div()
            .flex()
            .flex_col()
            .gap_0p5()
            .child(
                div()
                    .px_3()
                    .py_1()
                    .rounded_md()
                    .cursor_pointer()
                    .text_sm()
                    .font_weight(if hub_active {
                        openframe::FontWeight::BOLD
                    } else {
                        openframe::FontWeight::NORMAL
                    })
                    .bg(if hub_active {
                        pal.row_selected
                    } else if is_dark {
                        rgb(0x171b22)
                    } else {
                        rgb(0xf6f7fb)
                    })
                    .text_color(label_color)
                    .hover(move |style| {
                        style.bg(if hub_active {
                            pal.row_hover
                        } else if is_dark {
                            rgb(0x243246)
                        } else {
                            rgb(0xeef2ff)
                        })
                    })
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .flex()
                                    .gap_2()
                                    .items_center()
                                    .child(
                                        render_icon(hub_page.glyph()).size_4().text_color(icon_color),
                                    )
                                    .child(div().child(hub_page.title().to_string())),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(openframe::FontWeight::MEDIUM)
                                    .text_color(theme::sidebar_nav_idle_foreground(is_dark))
                                    .child(chevron),
                            ),
                    )
                    .on_mouse_down(
                        openframe::MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.active_page_id =
                                navigation::SETTINGS_HUB_ROOT_PAGE_ID.to_string();
                            this.settings_hub_expanded = !this.settings_hub_expanded;
                            cx.notify();
                        }),
                    ),
            )
            .child(if self.settings_hub_expanded {
                div()
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .pl_4()
                    .border_l_2()
                    .border_color(if is_dark {
                        rgb(0x374151)
                    } else {
                        rgb(0xe5e7eb)
                    })
                    .children(
                        self.settings_hub_page_ids_effective()
                            .into_iter()
                            .filter_map(|page_id| {
                                if !self.is_page_visible(page_id) {
                                    return None;
                                }
                                let page = self.page_ref(page_id)?;
                                let page_id_owned = page_id.to_string();
                                let is_active = self.active_page_id == page.id();
                                Some(
                                    div()
                                        .px_3()
                                        .py_1()
                                        .rounded_md()
                                        .cursor_pointer()
                                        .text_sm()
                                        .font_weight(if is_active {
                                            openframe::FontWeight::BOLD
                                        } else {
                                            openframe::FontWeight::NORMAL
                                        })
                                        .bg(if is_active {
                                            theme::nav_accent_palette(page.accent(), is_dark)
                                                .row_selected
                                        } else if is_dark {
                                            rgb(0x171b22)
                                        } else {
                                            rgb(0xf6f7fb)
                                        })
                                        .text_color(if is_active {
                                            theme::nav_accent_palette(page.accent(), is_dark)
                                                .icon_active
                                        } else {
                                            theme::sidebar_nav_idle_foreground(is_dark)
                                        })
                                        .hover(move |style| {
                                            let pal = theme::nav_accent_palette(
                                                page.accent(),
                                                is_dark,
                                            );
                                            style.bg(if is_active {
                                                pal.row_hover
                                            } else if is_dark {
                                                rgb(0x243246)
                                            } else {
                                                rgb(0xeef2ff)
                                            })
                                        })
                                        .child(
                                            div()
                                                .flex()
                                                .gap_2()
                                                .items_center()
                                                .child(
                                                    render_icon(page.glyph()).size_4().text_color(
                                                        if is_active {
                                                            theme::nav_accent_palette(
                                                                page.accent(),
                                                                is_dark,
                                                            )
                                                            .icon_active
                                                        } else {
                                                            theme::nav_accent_palette(
                                                                page.accent(),
                                                                is_dark,
                                                            )
                                                            .icon_idle
                                                        },
                                                    ),
                                                )
                                                .child(div().child(page.title().to_string())),
                                        )
                                        .on_mouse_down(
                                            openframe::MouseButton::Left,
                                            cx.listener(move |this, _, _, cx| {
                                                this.active_page_id = page_id_owned.clone();
                                                if page_id_owned == "global.modules" {
                                                    this.reload_modules();
                                                }
                                                this.sync_settings_hub_expanded_from_active_page();
                                                cx.notify();
                                            }),
                                        )
                                        .into_any_element(),
                                )
                            }),
                    )
            } else {
                div().hidden()
            })
    }

    pub fn sidebar_item(
        cx: &mut Context<Self>,
        label: openframe::SharedString,
        system_image: openframe::SharedString,
        page_id: String,
        is_active: bool,
        is_dark: bool,
        accent: String,
    ) -> impl IntoElement {
        let pal = theme::nav_accent_palette(accent.as_str(), is_dark);
        let icon_color = if is_active {
            pal.icon_active
        } else {
            pal.icon_idle
        };
        let label_color = if is_active {
            pal.icon_active
        } else {
            theme::sidebar_nav_idle_foreground(is_dark)
        };
        let is_shell_page = page_id == "utility.shell";
        let page_id_left = page_id.clone();
        div()
            .px_3()
            .py_2()
            .rounded_md()
            .cursor_pointer()
            .text_sm()
            .font_weight(if is_active {
                openframe::FontWeight::BOLD
            } else {
                openframe::FontWeight::NORMAL
            })
            .bg(if is_active {
                pal.row_selected
            } else if is_dark {
                rgb(0x171b22)
            } else {
                rgb(0xf6f7fb)
            })
            .text_color(label_color)
            .hover(move |style| {
                style.bg(if is_active {
                    pal.row_hover
                } else if is_dark {
                    rgb(0x243246)
                } else {
                    rgb(0xeef2ff)
                })
            })
            .child(
                div()
                    .flex()
                    .gap_2()
                    .items_center()
                    .child(render_icon(system_image.as_ref()).size_4().text_color(icon_color))
                    .child(div().child(label)),
            )
            .on_mouse_down(
                openframe::MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.active_page_id = page_id_left.clone();
                    cx.notify();
                }),
            )
            .on_mouse_down(
                openframe::MouseButton::Right,
                cx.listener(move |this, event: &openframe::MouseDownEvent, _, cx| {
                    if is_shell_page {
                        this.terminal_context_menu_open = true;
                        this.terminal_kill_menu = None;
                        this.context_menu_position = event.position;
                        cx.notify();
                    }
                }),
            )
    }

    pub fn sidebar_sub_item(
        cx: &mut Context<Self>,
        label: openframe::SharedString,
        terminal_id: usize,
        is_active: bool,
        is_dark: bool,
    ) -> impl IntoElement {
        let label_color = if is_active {
            theme::nav_accent_palette("emerald", is_dark).icon_active
        } else {
            theme::sidebar_nav_idle_foreground(is_dark)
        };
        let pal = theme::nav_accent_palette("emerald", is_dark);
        div()
            .ml_7()
            .pl_2()
            .pr_2()
            .py_1()
            .rounded_md()
            .cursor_pointer()
            .text_xs()
            .font_weight(if is_active {
                openframe::FontWeight::MEDIUM
            } else {
                openframe::FontWeight::NORMAL
            })
            .bg(if is_active {
                pal.row_selected
            } else if is_dark {
                rgb(0x171b22)
            } else {
                rgb(0xf6f7fb)
            })
            .text_color(label_color)
            .hover(move |style| {
                style.bg(if is_active {
                    pal.row_hover
                } else if is_dark {
                    rgb(0x1e2530)
                } else {
                    rgb(0xeef2ff)
                })
            })
            .child(div().child(label))
            .on_mouse_down(
                openframe::MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    if terminal_id < this.terminals.len() {
                        this.active_terminal_id = terminal_id;
                        this.active_page_id = "utility.shell".to_string();
                    }
                    cx.notify();
                }),
            )
            .on_mouse_down(
                openframe::MouseButton::Right,
                cx.listener(move |this, event: &openframe::MouseDownEvent, _, cx| {
                    this.terminal_kill_menu = Some(terminal_id);
                    this.terminal_context_menu_open = false;
                    this.context_menu_position = event.position;
                    cx.notify();
                }),
            )
    }
}
