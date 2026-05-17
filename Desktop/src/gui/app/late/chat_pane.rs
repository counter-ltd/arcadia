use openframe::{
    div, px, rgb, Context, Element, InteractiveElement, IntoElement, KeyDownEvent, MouseButton,
    ParentElement, StatefulInteractiveElement, Styled, Window,
};

use arcadia_core::modules::late::{send_ws, state, LateMessage};

use crate::gui::app::text_input_caret::{text_with_trailing_caret, TEXT_INPUT_CARET_CHAR};
use crate::gui::app::ArcadiaRoot;
use crate::gui::theme;

impl ArcadiaRoot {
    pub(super) fn late_message_list(&self, is_dark: bool) -> impl IntoElement {
        let arc = state();
        let st = arc.lock().unwrap_or_else(|e| e.into_inner());
        let room = self.late.active_room;
        let messages: Vec<LateMessage> = st
            .messages
            .iter()
            .filter(|_m| {
                // Show all messages; filter by room once server sends room_id per message.
                let _ = room;
                true
            })
            .cloned()
            .collect();
        drop(st);

        div()
            .id("late-chat-messages")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .gap_1()
            .px_3()
            .py_2()
            .children(if messages.is_empty() {
                vec![div()
                    .text_sm()
                    .text_color(theme::module_description_text(is_dark))
                    .child("No messages yet — connect with late.connect and subscribe to a room.")
                    .into_any()]
            } else {
                messages
                    .into_iter()
                    .map(|msg| {
                        div()
                            .py_1()
                            .flex()
                            .flex_col()
                            .gap_0()
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .gap_2()
                                    .items_baseline()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(openframe::FontWeight::SEMIBOLD)
                                            .text_color(theme::module_title_text(is_dark))
                                            .child(msg.username.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme::module_meta_text(is_dark))
                                            .child(format_timestamp(&msg.timestamp)),
                                    ),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme::module_description_text(is_dark))
                                    .child(msg.body.clone()),
                            )
                            .child(if msg.reactions.is_empty() {
                                div().into_any()
                            } else {
                                div()
                                    .flex()
                                    .flex_row()
                                    .gap_1()
                                    .mt_1()
                                    .children(msg.reactions.iter().map(|r| {
                                        div()
                                            .px_2()
                                            .py_0p5()
                                            .rounded_full()
                                            .bg(if is_dark {
                                                rgb(0x1e293b)
                                            } else {
                                                rgb(0xf1f5f9)
                                            })
                                            .text_xs()
                                            .text_color(theme::module_title_text(is_dark))
                                            .child(format!("{} {}", r.emoji, r.count))
                                    }))
                                    .into_any()
                            })
                            .into_any()
                    })
                    .collect()
            })
    }

    pub(super) fn late_compose_box(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> impl IntoElement {
        let input_text = self.late.compose_text.clone();
        let room = self.late.active_room;
        let compose_focused = self.late.compose_focus.is_focused(window);
        let blink = self.text_caret_blink_visible;

        let compose_border = theme::ui_border(cx, is_dark);
        let compose_input_bg = theme::ui_surface(cx, is_dark);
        let compose_radius = theme::ui_radius(cx);
        div()
            .px_3()
            .py_2()
            .border_t_1()
            .border_color(compose_border)
            .flex()
            .flex_row()
            .gap_2()
            .items_center()
            .child(
                div()
                    .flex_1()
                    .px_3()
                    .py_2()
                    .rounded(px(compose_radius))
                    .bg(compose_input_bg)
                    .border_1()
                    .border_color(compose_border)
                    .text_sm()
                    .text_color(theme::module_title_text(is_dark))
                    .track_focus(&self.late.compose_focus)
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, window, _| {
                            this.late.compose_focus.focus(window);
                        }),
                    )
                    .child(if input_text.is_empty() {
                        if compose_focused && blink {
                            div()
                                .text_color(theme::module_title_text(is_dark))
                                .child(TEXT_INPUT_CARET_CHAR.to_string())
                        } else {
                            div()
                                .text_color(theme::module_meta_text(is_dark))
                                .child("Type a message…")
                        }
                    } else {
                        div().child(text_with_trailing_caret(
                            input_text.as_str(),
                            compose_focused,
                            blink,
                        ))
                    })
                    .on_key_down(cx.listener(move |this, event: &KeyDownEvent, _, cx| {
                        let key = event.keystroke.key.as_str();
                        let mods = event.keystroke.modifiers;
                        if key == "backspace" {
                            this.late.compose_text.pop();
                            cx.notify();
                        } else if key == "enter" || key == "return" {
                            let body = this.late.compose_text.trim().to_string();
                            if !body.is_empty() {
                                send_ws(format!(
                                    r#"{{"type":"send","room_id":{room},"body":{}}}"#,
                                    serde_json::json!(body)
                                ));
                                this.late.compose_text.clear();
                                cx.notify();
                            }
                        } else if !mods.control && !mods.alt && !mods.platform && !mods.function {
                            if let Some(key_char) = &event.keystroke.key_char {
                                this.late.compose_text.push_str(key_char);
                                cx.notify();
                            }
                        } else if key == "space" {
                            this.late.compose_text.push(' ');
                            cx.notify();
                        }
                    })),
            )
            .child(
                div()
                    .cursor_pointer()
                    .px_3()
                    .py_2()
                    .rounded(px(compose_radius))
                    .bg(theme::ui_accent(cx))
                    .text_sm()
                    .font_weight(openframe::FontWeight::SEMIBOLD)
                    .text_color(theme::ui_accent_fg(cx))
                    .child("Send")
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            let body = this.late.compose_text.trim().to_string();
                            if !body.is_empty() {
                                send_ws(format!(
                                    r#"{{"type":"send","room_id":{room},"body":{}}}"#,
                                    serde_json::json!(body)
                                ));
                                this.late.compose_text.clear();
                                cx.notify();
                            }
                        }),
                    ),
            )
    }
}

fn format_timestamp(ts: &str) -> String {
    // ISO-8601 → "HH:MM" extraction; fall back to raw string.
    if ts.len() >= 16 {
        if let Some(time_part) = ts.get(11..16) {
            return time_part.to_string();
        }
    }
    ts.to_string()
}
