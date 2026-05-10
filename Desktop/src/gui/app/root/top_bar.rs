use openframe::{div, rgb, Context, InteractiveElement, IntoElement, ParentElement, Styled};

use crate::gui::app::ArcadiaRoot;
#[cfg(feature = "gui")]
use crate::gui::app::ShellMode;
use crate::gui::theme;

const LATE_ROOMS: &[(&str, u32)] = &[("1", 1), ("2", 2), ("3", 3), ("4", 4), ("5", 5)];

impl ArcadiaRoot {
    pub(crate) fn render_main_top_bar(
        &self,
        cx: &mut Context<Self>,
        active_page_title: openframe::SharedString,
        active_page_glyph: openframe::SharedString,
        is_dark: bool,
    ) -> impl IntoElement {
        div()
            .w_full()
            .px_3()
            .py_2()
            .border_b_1()
            .border_color(if is_dark {
                rgb(0x2a3340)
            } else {
                rgb(0xe6e8ef)
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
                            .child(Self::sidebar_toggle_button(cx, active_page_glyph.as_ref(), is_dark))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(openframe::FontWeight::SEMIBOLD)
                                    .text_color(if is_dark {
                                        rgb(0xe5e7eb)
                                    } else {
                                        rgb(0x1f2937)
                                    })
                                    .child(active_page_title),
                            )
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
                                            .rounded_md()
                                            .text_xs()
                                            .font_weight(openframe::FontWeight::SEMIBOLD)
                                            .bg(if is_active {
                                                if is_dark { rgb(0x0d9488) } else { rgb(0x99f6e4) }
                                            } else {
                                                theme::top_bar_pill_bg(is_dark)
                                            })
                                            .text_color(if is_active {
                                                if is_dark { rgb(0xf0fdfa) } else { rgb(0x134e4a) }
                                            } else {
                                                theme::top_bar_pill_text(is_dark)
                                            })
                                            .hover(move |style| {
                                                if !is_active {
                                                    style.bg(theme::top_bar_pill_hover_bg(is_dark))
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
                                    if self.active_page_id.as_str() == "utility.shell" {
                                        div()
                                            .px_2()
                                            .py_0p5()
                                            .rounded_md()
                                            .text_xs()
                                            .bg(if self.active_terminal().shell_mode == ShellMode::Generic {
                                                if is_dark { rgb(0x1e3a5f) } else { rgb(0xdbeafe) }
                                            } else {
                                                if is_dark { rgb(0x422006) } else { rgb(0xffedd5) }
                                            })
                                            .text_color(if self.active_terminal().shell_mode == ShellMode::Generic {
                                                if is_dark { rgb(0x93c5fd) } else { rgb(0x1d4ed8) }
                                            } else {
                                                if is_dark { rgb(0xfdba74) } else { rgb(0xc2410c) }
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
                                            .rounded_md()
                                            .text_xs()
                                            .bg(theme::top_bar_pill_bg(is_dark))
                                            .text_color(theme::top_bar_pill_text(is_dark))
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
                                            .rounded_md()
                                            .cursor_pointer()
                                            .text_xs()
                                            .bg(theme::top_bar_pill_bg(is_dark))
                                            .text_color(theme::top_bar_pill_text(is_dark))
                                            .hover(move |style| style.bg(theme::top_bar_pill_hover_bg(is_dark)))
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
                                            .rounded_md()
                                            .cursor_pointer()
                                            .text_xs()
                                            .bg(theme::top_bar_pill_bg(is_dark))
                                            .text_color(theme::top_bar_pill_text(is_dark))
                                            .hover(move |style| style.bg(theme::top_bar_pill_hover_bg(is_dark)))
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
                                    ))
                                },
                            )),
                    ),
            )
    }
}
