use openframe::{div, rgb, Context, InteractiveElement, IntoElement, ParentElement, Styled};

use arcadia_core::modules::late::state;
use arcadia_core::modules;

use super::visualizer::late_visualizer_inline;
use super::vote_panel::late_vote_pills;
use crate::gui::theme;

use crate::gui::app::ArcadiaRoot;

fn track_bar_vertical_rule(is_dark: bool) -> openframe::Div {
    div()
        .w_px()
        .h_6()
        .flex_shrink_0()
        .mx_2()
        .bg(if is_dark {
            rgb(0x475569)
        } else {
            rgb(0xcbd5e1)
        })
}

pub(super) fn late_top_bar(cx: &mut Context<ArcadiaRoot>, is_dark: bool) -> impl IntoElement {
    let arc = state();
    let st = arc.lock().unwrap_or_else(|e| e.into_inner());
    let track = st.now_playing.track.clone();
    let artist = st.now_playing.artist.clone();
    let connected = st.connected;
    drop(st);

    let reconnect_btn = div()
        .px_2()
        .py_0p5()
        .rounded_md()
        .cursor_pointer()
        .text_xs()
        .bg(theme::top_bar_pill_bg(is_dark))
        .text_color(theme::top_bar_pill_text(is_dark))
        .hover(move |style| {
            style.bg(theme::top_bar_pill_hover_bg(is_dark))
        })
        .child(if connected { "Reconnect" } else { "Connect" })
        .on_mouse_down(
            openframe::MouseButton::Left,
            cx.listener(|this, _, _, cx| {
                let ctx = this.execution_context();
                match modules::execute_command("late.connect", &[], &ctx) {
                    Ok(Some(msg)) => eprintln!("[late.gui] late.connect: {msg}"),
                    Ok(None) => eprintln!("[late.gui] late.connect: no output"),
                    Err(err) => eprintln!("[late.gui] late.connect error: {err}"),
                }
                cx.notify();
            }),
        );

    div()
        .px_3()
        .py_2()
        .bg(if is_dark { rgb(0x0a0f1a) } else { rgb(0xf0fdf4) })
        .border_b_1()
        .border_color(if is_dark { rgb(0x1e293b) } else { rgb(0xd1fae5) })
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .w_full()
                .min_w_0()
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .flex_shrink_0()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme::module_meta_text(is_dark))
                                .child("♫"),
                        )
                        .child(if track.is_empty() {
                            div()
                                .text_xs()
                                .text_color(theme::module_description_text(is_dark))
                                .child("No track info — connect to late.sh to stream.")
                        } else {
                            div()
                                .text_xs()
                                .text_color(theme::module_title_text(is_dark))
                                .child(format!("{track} · {artist}"))
                        }),
                )
                .child(track_bar_vertical_rule(is_dark))
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .min_w_0()
                        .flex_row()
                        .items_center()
                        .justify_center()
                        .overflow_hidden()
                        .px_2()
                        .child(late_visualizer_inline(is_dark)),
                )
                .child(track_bar_vertical_rule(is_dark))
                .child(if connected {
                    div()
                        .flex()
                        .flex_row()
                        .flex_shrink_0()
                        .items_center()
                        .gap_1()
                        .child(late_vote_pills(cx, is_dark))
                        .child(track_bar_vertical_rule(is_dark))
                        .child(reconnect_btn)
                } else {
                    div()
                        .flex()
                        .flex_row()
                        .flex_shrink_0()
                        .items_center()
                        .child(reconnect_btn)
                }),
        )
}
