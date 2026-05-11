//! Search field shared by Modules and Extensions list panels.

use openframe::{
    Context, InteractiveElement, IntoElement, KeyDownEvent, MouseButton, ParentElement, Styled,
    Window, div, px,
};

use crate::gui::app::text_input_caret::{TEXT_INPUT_CARET_CHAR, text_with_trailing_caret};
use crate::gui::app::ArcadiaRoot;
use crate::gui::theme;

#[derive(Clone, Copy)]
pub(crate) enum ListPanelSearchKind {
    Modules,
    Extensions,
    Permissions,
}

pub(crate) fn list_panel_row_matches(q_lower: &str, primary: &str, extras: &[&str]) -> bool {
    if q_lower.is_empty() {
        return true;
    }
    primary
        .to_ascii_lowercase()
        .contains(q_lower)
        || extras
            .iter()
            .any(|s| s.to_ascii_lowercase().contains(q_lower))
}

impl ArcadiaRoot {
    pub(crate) fn list_panel_search_bar(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
        kind: ListPanelSearchKind,
    ) -> impl IntoElement {
        let (text, focus) = match kind {
            ListPanelSearchKind::Modules => (&self.modules_search_query, &self.modules_search_focus),
            ListPanelSearchKind::Extensions => {
                (&self.extensions_search_query, &self.extensions_search_focus)
            }
            ListPanelSearchKind::Permissions => {
                (&self.permissions_search_query, &self.permissions_search_focus)
            }
        };
        let placeholder = match kind {
            ListPanelSearchKind::Modules => "Search modules…",
            ListPanelSearchKind::Extensions => "Search extensions…",
            ListPanelSearchKind::Permissions => "Search permissions…",
        };

        let input_bg = theme::glyph_snapshot(cx)
            .map(|g| g.surface)
            .unwrap_or_else(|| theme::ui_surface(cx, is_dark));
        let input_border = theme::glyph_snapshot(cx)
            .map(|g| g.border)
            .unwrap_or_else(|| theme::ui_border(cx, is_dark));
        let radius = theme::ui_radius(cx);
        let title_c = theme::module_title_text(is_dark);
        let meta_c = theme::module_meta_text(is_dark);

        let fh = focus.clone();
        let focused = focus.is_focused(window);
        let blink = self.text_caret_blink_visible;

        div()
            .w_full()
            .px_3()
            .py_2()
            .rounded(px(radius))
            .bg(input_bg)
            .border_1()
            .border_color(input_border)
            .text_sm()
            .text_color(title_c)
            .track_focus(focus)
            .on_mouse_down(MouseButton::Left, cx.listener(move |_, _, window, _| {
                fh.focus(window);
            }))
            .child(if text.is_empty() {
                if focused && blink {
                    div()
                        .text_color(title_c)
                        .child(TEXT_INPUT_CARET_CHAR.to_string())
                } else {
                    div().text_color(meta_c).child(placeholder)
                }
            } else {
                div().child(text_with_trailing_caret(text, focused, blink))
            })
            .on_key_down(cx.listener(move |this, event: &KeyDownEvent, _, cx| {
                let key = event.keystroke.key.as_str();
                let mods = event.keystroke.modifiers;
                let buf = match kind {
                    ListPanelSearchKind::Modules => &mut this.modules_search_query,
                    ListPanelSearchKind::Extensions => &mut this.extensions_search_query,
                    ListPanelSearchKind::Permissions => &mut this.permissions_search_query,
                };
                if key == "backspace" {
                    buf.pop();
                    cx.notify();
                } else if !mods.control && !mods.alt && !mods.platform && !mods.function {
                    if let Some(key_char) = &event.keystroke.key_char {
                        buf.push_str(key_char);
                        cx.notify();
                    }
                } else if key == "space" {
                    buf.push(' ');
                    cx.notify();
                }
            }))
    }
}
