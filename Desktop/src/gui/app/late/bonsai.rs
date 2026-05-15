use openframe::prelude::FluentBuilder as _;
use openframe::{
    div, px, Context, InteractiveElement, IntoElement, MouseButton, ParentElement, Styled,
};

use arcadia_core::modules;
use arcadia_core::modules::late::state;

use crate::gui::app::ArcadiaRoot;
use crate::gui::theme::{self, glyph_snapshot};

impl ArcadiaRoot {
    pub(super) fn late_bonsai(&self, cx: &mut Context<Self>, is_dark: bool) -> impl IntoElement {
        let arc = state();
        let st = arc.lock().unwrap_or_else(|e| e.into_inner());
        let art = st.bonsai_art.clone();
        drop(st);

        let glyph = glyph_snapshot(cx);
        let is_glyph = glyph.is_some();

        let panel_bg = glyph
            .as_ref()
            .map(|g| g.surface)
            .unwrap_or_else(|| theme::module_panel_bg(is_dark));
        let panel_stroke = glyph
            .as_ref()
            .map(|g| g.border)
            .unwrap_or_else(|| theme::module_panel_stroke(is_dark));
        let panel_radius = glyph.as_ref().map(|g| g.border_radius).unwrap_or(12.0);
        let title_c = glyph
            .as_ref()
            .map(|g| g.text)
            .unwrap_or_else(|| theme::module_title_text(is_dark));
        let desc_c = glyph
            .as_ref()
            .map(|g| g.dim)
            .unwrap_or_else(|| theme::module_description_text(is_dark));
        let btn_bg = glyph
            .as_ref()
            .map(|g| g.accent)
            .unwrap_or_else(|| theme::module_button_enable_bg(is_dark));
        let btn_text = glyph
            .as_ref()
            .map(|g| g.bg)
            .unwrap_or_else(|| theme::module_button_enable_text(is_dark));
        let btn_hover = glyph
            .as_ref()
            .map(|g| g.surface2)
            .unwrap_or_else(|| theme::module_button_enable_hover_bg(is_dark));
        let well_bg = glyph
            .as_ref()
            .map(|g| g.bg)
            .unwrap_or_else(|| theme::late_bonsai_well_bg(is_dark));
        let well_stroke = glyph
            .as_ref()
            .map(|g| g.border)
            .unwrap_or_else(|| theme::late_bonsai_well_stroke(is_dark));
        let pot_band = glyph
            .as_ref()
            .map(|g| g.surface2)
            .unwrap_or_else(|| theme::late_bonsai_pot_band(is_dark));
        let foliage_c = glyph
            .as_ref()
            .map(|g| g.accent)
            .unwrap_or_else(|| theme::late_bonsai_foliage_text(is_dark));
        let accent_bar = glyph
            .as_ref()
            .map(|g| g.accent)
            .unwrap_or_else(|| theme::nav_accent_palette("violet", is_dark).icon_active);

        div()
            .w_full()
            .p_3()
            .rounded(px(panel_radius))
            .bg(panel_bg)
            .border_1()
            .border_color(panel_stroke)
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
                                    .text_color(title_c)
                                    .child("Bonsai"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(desc_c)
                                    .child("Living ASCII from late.sh"),
                            ),
                    )
                    .child(
                        div()
                            .cursor_pointer()
                            .flex_shrink_0()
                            .px_2()
                            .py_1()
                            .rounded(px(panel_radius.min(6.0)))
                            .bg(btn_bg)
                            .text_xs()
                            .font_weight(openframe::FontWeight::SEMIBOLD)
                            .text_color(btn_text)
                            .hover(move |style| style.bg(btn_hover))
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
                    .when(!is_glyph, |d| d.rounded_lg())
                    .overflow_hidden()
                    .border_1()
                    .border_color(well_stroke)
                    .child(div().w(px(3.)).min_w(px(3.)).bg(accent_bar))
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .min_w_0()
                            .child(div().w_full().h(px(5.)).bg(pot_band))
                            .child(div().w_full().p_3().bg(well_bg).flex().flex_col().child(
                                if art.is_empty() {
                                    div()
                                        .w_full()
                                        .py_6()
                                        .flex()
                                        .justify_center()
                                        .items_center()
                                        .text_xs()
                                        .text_color(desc_c)
                                        .child("No bonsai yet — connect to late.sh.")
                                } else {
                                    div()
                                        .w_full()
                                        .flex()
                                        .flex_col()
                                        .font_family("monospace")
                                        .text_sm()
                                        .text_color(foliage_c)
                                        .children(
                                            art.into_iter()
                                                .map(|line| div().line_height(px(15.)).child(line)),
                                        )
                                },
                            )),
                    ),
            )
    }
}
