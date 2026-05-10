use openframe::{div, px, rgb, Context, InteractiveElement, IntoElement, ParentElement, Styled};
use openframe::prelude::FluentBuilder as _;

use crate::gui::app::ArcadiaRoot;
#[cfg(feature = "gui")]
use crate::gui::app::ShellMode;
use crate::gui::theme::{self};

const LATE_ROOMS: &[(&str, u32)] = &[("1", 1), ("2", 2), ("3", 3), ("4", 4), ("5", 5)];

impl ArcadiaRoot {
    pub(crate) fn render_main_top_bar(
        &self,
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
}
