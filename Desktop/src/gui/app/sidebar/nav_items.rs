use gpui::{div, rgb, Context, InteractiveElement, IntoElement, ParentElement, Styled};

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
                gpui::MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.sidebar_visible = !this.sidebar_visible;
                    cx.notify();
                }),
            )
    }

    pub fn sidebar_group_item(
        cx: &mut Context<Self>,
        label: gpui::SharedString,
        system_image: gpui::SharedString,
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
                gpui::FontWeight::BOLD
            } else {
                gpui::FontWeight::NORMAL
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
                gpui::MouseButton::Left,
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
        label: gpui::SharedString,
        system_image: gpui::SharedString,
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
                gpui::FontWeight::SEMIBOLD
            } else {
                gpui::FontWeight::NORMAL
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
                gpui::MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.active_page_id = page_id.clone();
                    if page_id == "global.modules" {
                        this.reload_modules();
                    }
                    cx.notify();
                }),
            )
    }

    pub fn sidebar_global_item(
        cx: &mut Context<Self>,
        label: gpui::SharedString,
        system_image: gpui::SharedString,
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
                gpui::FontWeight::BOLD
            } else {
                gpui::FontWeight::NORMAL
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
                gpui::MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.active_page_id = page_id.clone();
                    if page_id == "global.modules" {
                        this.reload_modules();
                    }
                    cx.notify();
                }),
            )
    }

    pub fn sidebar_item(
        cx: &mut Context<Self>,
        label: gpui::SharedString,
        system_image: gpui::SharedString,
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
            .py_2()
            .rounded_md()
            .cursor_pointer()
            .text_sm()
            .font_weight(if is_active {
                gpui::FontWeight::BOLD
            } else {
                gpui::FontWeight::NORMAL
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
                gpui::MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.active_page_id = page_id.clone();
                    cx.notify();
                }),
            )
    }
}
