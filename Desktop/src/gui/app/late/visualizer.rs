use openframe::{div, rgb, IntoElement, ParentElement, Styled};

use arcadia_core::modules::late::state;

pub(super) fn late_visualizer_inline(is_dark: bool) -> impl IntoElement {
    let arc = state();
    let st = arc.lock().unwrap_or_else(|e| e.into_inner());
    let frame = st.visualizer_frame.clone();
    drop(st);

    div()
        .font_family("monospace")
        .text_xs()
        .text_color(if is_dark { rgb(0x5eead4) } else { rgb(0x0d9488) })
        .child(if frame.is_empty() {
            "· · · · · ·".to_string()
        } else {
            frame
        })
}
