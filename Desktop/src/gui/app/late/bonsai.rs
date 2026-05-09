use openframe::{
    div, px, Context, InteractiveElement, IntoElement, MouseButton, ParentElement, Styled,
};

use arcadia_core::modules;
use arcadia_core::modules::late::state;

use crate::gui::app::ArcadiaRoot;
use crate::gui::theme;

impl ArcadiaRoot {
    pub(super) fn late_bonsai(&self, cx: &mut Context<Self>, is_dark: bool) -> impl IntoElement {
        let arc = state();
        let st = arc.lock().unwrap_or_else(|e| e.into_inner());
        let art = st.bonsai_art.clone();
        drop(st);

        let accent = theme::nav_accent_palette("violet", is_dark);

        div()
            .w_full()
            .p_3()
            .rounded_xl()
            .bg(theme::module_panel_bg(is_dark))
            .border_1()
            .border_color(theme::module_panel_stroke(is_dark))
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .items_start()
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_0p5()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(openframe::FontWeight::SEMIBOLD)
                                    .text_color(theme::module_title_text(is_dark))
                                    .child("Bonsai"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::module_description_text(is_dark))
                                    .child("Living ASCII from late.sh"),
                            ),
                    )
                    .child(
                        div()
                            .cursor_pointer()
                            .flex_shrink_0()
                            .px_2()
                            .py_1()
                            .rounded_md()
                            .bg(theme::module_button_enable_bg(is_dark))
                            .text_xs()
                            .font_weight(openframe::FontWeight::SEMIBOLD)
                            .text_color(theme::module_button_enable_text(is_dark))
                            .hover(|style| style.bg(theme::module_button_enable_hover_bg(is_dark)))
                            .child("Water")
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, _, cx| {
                                    let ctx = this.execution_context();
                                    match modules::execute_command("late.water", &[], &ctx) {
                                        Ok(Some(msg)) => eprintln!("[late.gui] late.water: {msg}"),
                                        Ok(None) => eprintln!("[late.gui] late.water: no output"),
                                        Err(err) => eprintln!("[late.gui] late.water error: {err}"),
                                    }
                                    cx.notify();
                                }),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .rounded_lg()
                    .overflow_hidden()
                    .border_1()
                    .border_color(theme::late_bonsai_well_stroke(is_dark))
                    .child(
                        div()
                            .w(px(3.))
                            .min_w(px(3.))
                            .bg(accent.icon_active),
                    )
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .min_w_0()
                            .child(
                                div()
                                    .w_full()
                                    .h(px(5.))
                                    .bg(theme::late_bonsai_pot_band(is_dark)),
                            )
                            .child(
                                div()
                                    .w_full()
                                    .p_3()
                                    .bg(theme::late_bonsai_well_bg(is_dark))
                                    .flex()
                                    .flex_col()
                                    .child(if art.is_empty() {
                                        div()
                                            .w_full()
                                            .py_6()
                                            .flex()
                                            .justify_center()
                                            .items_center()
                                            .text_xs()
                                            .text_color(theme::module_description_text(is_dark))
                                            .child("No bonsai yet — connect to late.sh.")
                                    } else {
                                        div()
                                            .w_full()
                                            .flex()
                                            .flex_col()
                                            .font_family("monospace")
                                            .text_sm()
                                            .text_color(theme::late_bonsai_foliage_text(is_dark))
                                            .children(art.into_iter().map(|line| {
                                                div()
                                                    .line_height(px(15.))
                                                    .child(line)
                                            }))
                                    }),
                            ),
                    ),
            )
    }
}
