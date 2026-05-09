use openframe::{div, Context, IntoElement, ParentElement, Styled, Window};

use crate::gui::app::ArcadiaRoot;

use super::{sidebar::late_sidebar, top_bar::late_top_bar};

pub fn late_now_playing_panel(root: &ArcadiaRoot, cx: &mut Context<ArcadiaRoot>, is_dark: bool) -> impl IntoElement {
    div()
        .w_full()
        .h_full()
        .flex()
        .flex_row()
        // Main chat column
        .child(
            div()
                .flex_1()
                .min_w_0()
                .h_full()
                .flex()
                .flex_col()
                .child(late_top_bar(cx, is_dark))
                .child(root.late_message_list(is_dark))
                .child(root.late_compose_box(cx, is_dark)),
        )
        // Right sidebar
        .child(late_sidebar(root, cx, is_dark))
}

// ArcadiaRoot dispatch stubs referenced from navigation.rs
impl ArcadiaRoot {
    pub(crate) fn render_late_now_playing(
        &self,
        _window: &mut Window,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> openframe::Div {
        div()
            .flex_1()
            .h_full()
            .min_h_0()
            .child(late_now_playing_panel(self, cx, is_dark))
    }
}
