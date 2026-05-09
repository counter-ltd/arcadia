use openframe::{div, Context, InteractiveElement, IntoElement, MouseButton, ParentElement, Styled};

use arcadia_core::modules::late::{send_ws, state};

use crate::gui::theme;

use crate::gui::app::ArcadiaRoot;

pub(super) fn late_vote_pills(cx: &mut Context<ArcadiaRoot>, is_dark: bool) -> impl IntoElement {
    let arc = state();
    let st = arc.lock().unwrap_or_else(|e| e.into_inner());
    let votes = st.votes.clone();
    drop(st);

    let genres: &[(&str, u32, &str)] = &[
        ("Lofi", votes.lofi, "lofi"),
        ("Ambient", votes.ambient, "ambient"),
        ("Classic", votes.classic, "classic"),
    ];

    div()
        .flex()
        .flex_row()
        .gap_1()
        .children(genres.iter().map(|(label, count, key)| {
            let genre_key = (*key).to_string();
            div()
                .cursor_pointer()
                .px_2()
                .py_0p5()
                .rounded_md()
                .text_xs()
                .bg(theme::top_bar_pill_bg(is_dark))
                .text_color(theme::top_bar_pill_text(is_dark))
                .hover(move |style| {
                    style.bg(theme::top_bar_pill_hover_bg(is_dark))
                })
                .child(format!("{label} {count}"))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |_this, _, _, _cx| {
                        send_ws(format!(
                            r#"{{"type":"vote","genre":"{}"}}"#,
                            genre_key
                        ));
                    }),
                )
        }))
}
