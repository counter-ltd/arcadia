//! `utility.shell` on iOS: `shell.execute` and LAN routing only — no PTY/TUI.

use arcadia_core::capabilities;
use arcadia_core::modules;
use openframe::{
    div, px, Context, InteractiveElement, KeyDownEvent, ParentElement, StatefulInteractiveElement,
    Styled, Window, WindowAppearance,
};

use super::text_input_caret::TEXT_INPUT_CARET_CHAR;
use super::ArcadiaRoot;
use crate::gui::theme;

impl ArcadiaRoot {
    pub(crate) fn ios_execute_shell_panel(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> openframe::Div {
        let is_dark = matches!(
            window.appearance(),
            WindowAppearance::Dark | WindowAppearance::VibrantDark
        );
        let shell_bg = theme::ui_surface(cx, is_dark);
        let shell_border = theme::ui_border(cx, is_dark);
        let shell_accent = theme::ui_accent(cx);
        let is_focused = self.ios_shell_focus.is_focused(window);
        let hint: &'static str = if self.remote_route.is_none() {
            capabilities::SHELL_EXECUTE_UNAVAILABLE_IOS_LOCAL
        } else {
            "Commands execute on the LAN host (thin-client route)."
        };

        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .rounded(px(theme::ui_radius(cx)))
            .border_1()
            .border_color(shell_border)
            .bg(shell_bg)
            .child(
                div()
                    .px_3()
                    .py_2()
                    .text_sm()
                    .text_color(theme::ui_subtext(cx, is_dark))
                    .child(hint),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .id("arcadia-ios-shell-output")
                    .overflow_y_scroll()
                    .track_scroll(&self.ios_shell_scroll)
                    .child(
                        div().w_full().p_3().flex().flex_col().gap_1().children(
                            self.ios_shell_history
                                .iter()
                                .filter(|l| !l.is_empty())
                                .map(|line| {
                                    div()
                                        .text_sm()
                                        .text_color(theme::ui_text(cx, is_dark))
                                        .font_family("monospace")
                                        .child(line.clone())
                                }),
                        ),
                    ),
            )
            .child(
                div()
                    .w_full()
                    .flex_shrink_0()
                    .px_3()
                    .py_2()
                    .flex()
                    .gap_2()
                    .items_center()
                    .border_t_1()
                    .border_color(if is_focused {
                        shell_accent
                    } else {
                        shell_border
                    })
                    .track_focus(&self.ios_shell_focus)
                    .accessibility_label("Shell command input")
                    .accessibility_hint("Enter sends command to host.")
                    .on_mouse_down(
                        openframe::MouseButton::Left,
                        cx.listener(|this, _, window, _| {
                            this.ios_shell_focus.focus(window);
                        }),
                    )
                    .on_key_down(cx.listener(Self::handle_ios_shell_key_down))
                    .child(div().text_sm().text_color(shell_accent).child("$"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme::ui_text(cx, is_dark))
                            .child(self.ios_shell_input_with_cursor(is_focused)),
                    ),
            )
    }

    pub(crate) fn ios_shell_input_with_cursor(&self, is_focused: bool) -> String {
        let chars = self.ios_shell_input.chars().collect::<Vec<_>>();
        let cursor = self.ios_shell_cursor.min(chars.len());
        let mut out = String::with_capacity(chars.len() + 1);
        for (idx, ch) in chars.iter().enumerate() {
            if idx == cursor && is_focused && self.text_caret_blink_visible {
                out.push(TEXT_INPUT_CARET_CHAR);
            }
            out.push(*ch);
        }
        if cursor == chars.len() && is_focused && self.text_caret_blink_visible {
            out.push(TEXT_INPUT_CARET_CHAR);
        }
        out
    }

    pub(crate) fn handle_ios_shell_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.active_page_id.as_str() != "utility.shell" {
            return;
        }
        let key = event.keystroke.key.as_str();
        let mods = event.keystroke.modifiers;
        match key {
            "enter" => {
                let command = self.ios_shell_input.trim().to_string();
                if command.eq_ignore_ascii_case("clear") || command.eq_ignore_ascii_case("cls") {
                    self.ios_shell_history.clear();
                    self.ios_shell_input.clear();
                    self.ios_shell_cursor = 0;
                    self.ios_shell_history_index = None;
                    cx.notify();
                    return;
                }
                if !command.is_empty() {
                    let ctx = self.execution_context();
                    let display = command.clone();
                    let out = modules::execute_command("shell.execute", &[display.as_str()], &ctx);
                    self.ios_shell_history.push(format!("$ {display}"));
                    match out {
                        Ok(Some(s)) => {
                            if s.is_empty() {
                                self.ios_shell_history.push("(no output)".to_string());
                            } else {
                                for line in s.lines() {
                                    self.ios_shell_history.push(line.to_string());
                                }
                            }
                        }
                        Ok(None) => self
                            .ios_shell_history
                            .push("Unknown shell command token.".to_string()),
                        Err(e) => self.ios_shell_history.push(e),
                    }
                    self.ios_shell_command_history.push(command);
                    self.ios_shell_input.clear();
                    self.ios_shell_cursor = 0;
                    self.ios_shell_history_index = None;
                    self.ios_shell_scroll.scroll_to_bottom();
                }
                cx.notify();
            }
            "backspace" => {
                if self.ios_shell_cursor > 0 {
                    let mut chars = self.ios_shell_input.chars().collect::<Vec<_>>();
                    chars.remove(self.ios_shell_cursor - 1);
                    self.ios_shell_input = chars.into_iter().collect();
                    self.ios_shell_cursor -= 1;
                    cx.notify();
                }
            }
            "left" => {
                self.ios_shell_cursor = self.ios_shell_cursor.saturating_sub(1);
                cx.notify();
            }
            "right" => {
                let len = self.ios_shell_input.chars().count();
                self.ios_shell_cursor = (self.ios_shell_cursor + 1).min(len);
                cx.notify();
            }
            "up" => {
                if !self.ios_shell_command_history.is_empty() {
                    let next_index = match self.ios_shell_history_index {
                        Some(index) => index.saturating_sub(1),
                        None => self.ios_shell_command_history.len().saturating_sub(1),
                    };
                    self.ios_shell_history_index = Some(next_index);
                    self.ios_shell_input = self.ios_shell_command_history[next_index].clone();
                    self.ios_shell_cursor = self.ios_shell_input.chars().count();
                    cx.notify();
                }
            }
            "down" => {
                if let Some(index) = self.ios_shell_history_index {
                    let next_index = index + 1;
                    if next_index < self.ios_shell_command_history.len() {
                        self.ios_shell_history_index = Some(next_index);
                        self.ios_shell_input = self.ios_shell_command_history[next_index].clone();
                        self.ios_shell_cursor = self.ios_shell_input.chars().count();
                    } else {
                        self.ios_shell_history_index = None;
                        self.ios_shell_input.clear();
                        self.ios_shell_cursor = 0;
                    }
                    cx.notify();
                }
            }
            "home" => {
                self.ios_shell_cursor = 0;
                cx.notify();
            }
            "end" => {
                self.ios_shell_cursor = self.ios_shell_input.chars().count();
                cx.notify();
            }
            "space" => {
                let mut chars = self.ios_shell_input.chars().collect::<Vec<_>>();
                chars.insert(self.ios_shell_cursor, ' ');
                self.ios_shell_input = chars.into_iter().collect();
                self.ios_shell_cursor += 1;
                cx.notify();
            }
            _ => {
                if !mods.control && !mods.alt && !mods.platform && !mods.function {
                    if let Some(key_char) = &event.keystroke.key_char {
                        let mut chars = self.ios_shell_input.chars().collect::<Vec<_>>();
                        for ch in key_char.chars() {
                            chars.insert(self.ios_shell_cursor, ch);
                            self.ios_shell_cursor += 1;
                        }
                        self.ios_shell_input = chars.into_iter().collect();
                        cx.notify();
                    }
                }
            }
        }
    }
}
