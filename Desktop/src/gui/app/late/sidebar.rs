use openframe::{
    div, Context, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement,
    Styled,
};

use arcadia_core::modules::late::state;

use crate::gui::app::ArcadiaRoot;
use crate::gui::theme;

pub(super) fn late_sidebar(root: &ArcadiaRoot, cx: &mut Context<ArcadiaRoot>, is_dark: bool) -> impl IntoElement {
    let arc = state();
    let st = arc.lock().unwrap_or_else(|e| e.into_inner());
    let users: Vec<String> = st.online_users.iter().map(|u| u.username.clone()).collect();
    let activity: Vec<(String, String, String)> = st
        .activity_feed
        .iter()
        .rev()
        .take(15)
        .map(|e| (e.kind.clone(), e.username.clone(), e.timestamp.clone()))
        .collect();
    drop(st);

    let glyph = theme::glyph_snapshot(cx);
    let subtext_c = glyph.as_ref().map(|g| g.dim).unwrap_or_else(|| theme::module_meta_text(is_dark));
    let desc_c    = glyph.as_ref().map(|g| g.dim).unwrap_or_else(|| theme::module_description_text(is_dark));
    let sidebar_border = glyph.as_ref().map(|g| g.border).unwrap_or_else(|| theme::ui_border(cx, is_dark));
    div()
        .w_72()
        .h_full()
        .flex()
        .flex_col()
        .gap_0()
        .border_l_1()
        .border_color(sidebar_border)
        .child(
            div()
                .px_3()
                .py_2()
                .border_b_1()
                .border_color(sidebar_border)
                .text_xs()
                .font_weight(openframe::FontWeight::SEMIBOLD)
                .text_color(subtext_c)
                .child(format!("● {} online", users.len())),
        )
        .child(
            div()
                .flex_1()
                .min_h_0()
                .flex()
                .flex_col()
                .gap_0()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .min_h_0()
                        .id("late-sidebar-scroll")
                        .overflow_y_scroll()
                        .child(
                            // Online users list
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .px_3()
                                .py_2()
                                .children(users.into_iter().map(|username| {
                                    div()
                                        .text_xs()
                                        .text_color(desc_c)
                                        .child(format!("@{username}"))
                                })),
                        )
                        .child(
                            div()
                                .mx_3()
                                .my_1()
                                .h_px()
                                .bg(sidebar_border),
                        )
                        .child(
                            div()
                                .px_3()
                                .py_1()
                                .text_xs()
                                .font_weight(openframe::FontWeight::SEMIBOLD)
                                .text_color(subtext_c)
                                .child("Activity"),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .px_3()
                                .pb_2()
                                .children(activity.into_iter().map(|(kind, username, ts)| {
                                    let icon = if kind == "join" { "→" } else { "←" };
                                    div()
                                        .text_xs()
                                        .text_color(desc_c)
                                        .child(format!(
                                            "{icon} @{username} {}",
                                            format_relative(&ts)
                                        ))
                                })),
                        ),
                )
                .child(
                    div()
                        .flex_shrink_0()
                        .px_3()
                        .pt_2()
                        .pb_3()
                        .border_t_1()
                        .border_color(sidebar_border)
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(root.late_bonsai(cx, is_dark)),
                ),
        )
}

fn format_relative(ts: &str) -> String {
    if ts.len() >= 16 {
        if let Some(t) = ts.get(11..16) {
            return t.to_string();
        }
    }
    ts.to_string()
}
