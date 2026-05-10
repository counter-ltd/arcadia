use openframe::{div, Context, InteractiveElement, IntoElement, ParentElement, Styled};

use arcadia_core::modules::late::state;
use arcadia_core::modules;

use super::visualizer::late_visualizer_inline;
use super::vote_panel::late_vote_pills;
use crate::gui::theme;

use crate::gui::app::ArcadiaRoot;

fn track_bar_vertical_rule(border_c: openframe::Rgba) -> openframe::Div {
    div()
        .w_px()
        .h_6()
        .flex_shrink_0()
        .mx_2()
        .bg(border_c)
}

pub(super) fn late_top_bar(cx: &mut Context<ArcadiaRoot>, is_dark: bool) -> impl IntoElement {
    let arc = state();
    let st = arc.lock().unwrap_or_else(|e| e.into_inner());
    let track = st.now_playing.track.clone();
    let artist = st.now_playing.artist.clone();
    let connected = st.connected;
    drop(st);

    let rule_c = theme::ui_border(cx, is_dark);
    let reconnect_bg = theme::action_pill_bg(cx, is_dark);
    let reconnect_tc = theme::action_pill_text(cx, is_dark);
    let reconnect_hover = theme::action_pill_hover_bg(cx, is_dark);
    let track_primary = theme::content_emphasis_text(cx, is_dark);
    let track_note = theme::content_muted_text(cx, is_dark);
    let track_empty = theme::content_subdued_text(cx, is_dark);

    let reconnect_btn = div()
        .px_2()
        .py_0p5()
        .rounded_md()
        .cursor_pointer()
        .text_xs()
        .bg(reconnect_bg)
        .text_color(reconnect_tc)
        .hover(move |style| {
            style.bg(reconnect_hover)
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

    let topbar_border = theme::ui_border(cx, is_dark);
    let topbar_bg = theme::ui_bg(cx, is_dark);
    div()
        .px_3()
        .py_2()
        .bg(topbar_bg)
        .border_b_1()
        .border_color(topbar_border)
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
                                .text_color(track_note)
                                .child("♫"),
                        )
                        .child(if track.is_empty() {
                            div()
                                .text_xs()
                                .text_color(track_empty)
                                .child("No track info — connect to late.sh to stream.")
                        } else {
                            div()
                                .text_xs()
                                .text_color(track_primary)
                                .child(format!("{track} · {artist}"))
                        }),
                )
                .child(track_bar_vertical_rule(rule_c))
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
                .child(track_bar_vertical_rule(rule_c))
                .child(if connected {
                    div()
                        .flex()
                        .flex_row()
                        .flex_shrink_0()
                        .items_center()
                        .gap_1()
                        .child(late_vote_pills(cx, is_dark))
                        .child(track_bar_vertical_rule(rule_c))
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
