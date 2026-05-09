use arcadia_core::config::late::LateConfig;
use arcadia_core::config::ConfigFile;
use arcadia_core::modules;
use openframe::{
    div, rgb, Context, Element, FontWeight, InteractiveElement, IntoElement, KeyDownEvent,
    MouseButton, ParentElement, Styled,
};

use crate::gui::app::ArcadiaRoot;
use crate::gui::theme;

impl ArcadiaRoot {
    pub(crate) fn late_settings_panel(
        &self,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> impl IntoElement {
        let server_url = self.late_settings_server_url.clone();
        let username = self.late_settings_username.clone();
        let default_room = self.late_settings_default_room.clone();
        let feedback = self.late_settings_feedback.clone();
        let has_token = LateConfig::load_or_create()
            .map(|c| !c.auth_token.is_empty())
            .unwrap_or(false);
        let palette = theme::nav_accent_palette("violet", is_dark);

        let input_bg = if is_dark { rgb(0x0f172a) } else { rgb(0xf8fafc) };
        let input_border = if is_dark { rgb(0x1e293b) } else { rgb(0xe2e8f0) };

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_6()
            // Header
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_2xl()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme::module_title_text(is_dark))
                            .child("Late.sh"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme::module_meta_text(is_dark))
                            .child(
                                "Connection and preference settings. Changes take effect on next connect.",
                            ),
                    ),
            )
            // Connection fields
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    // Server URL
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme::module_title_text(is_dark))
                                    .child("Server URL"),
                            )
                            .child({
                                let url_val = server_url.clone();
                                div()
                                    .id("late-settings-url")
                                    .px_3()
                                    .py_2()
                                    .rounded_lg()
                                    .bg(input_bg)
                                    .border_1()
                                    .border_color(input_border)
                                    .text_sm()
                                    .text_color(theme::module_title_text(is_dark))
                                    .track_focus(&self.late_settings_server_url_focus)
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, window, _| {
                                            this.late_settings_server_url_focus.focus(window);
                                        }),
                                    )
                                    .child(if url_val.is_empty() {
                                        div()
                                            .text_color(theme::module_meta_text(is_dark))
                                            .child("https://late.sh")
                                    } else {
                                        div().child(url_val)
                                    })
                                    .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                                        let key = event.keystroke.key.as_str();
                                        let mods = event.keystroke.modifiers;
                                        if key == "backspace" {
                                            this.late_settings_server_url.pop();
                                            cx.notify();
                                        } else if key == "space" {
                                            this.late_settings_server_url.push(' ');
                                            cx.notify();
                                        } else if !mods.control
                                            && !mods.alt
                                            && !mods.platform
                                            && !mods.function
                                        {
                                            if let Some(ch) = &event.keystroke.key_char {
                                                this.late_settings_server_url.push_str(ch);
                                                cx.notify();
                                            }
                                        }
                                    }))
                            }),
                    )
                    // Username
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme::module_title_text(is_dark))
                                    .child("Username"),
                            )
                            .child({
                                let uname_val = username.clone();
                                div()
                                    .id("late-settings-username")
                                    .px_3()
                                    .py_2()
                                    .rounded_lg()
                                    .bg(input_bg)
                                    .border_1()
                                    .border_color(input_border)
                                    .text_sm()
                                    .text_color(theme::module_title_text(is_dark))
                                    .track_focus(&self.late_settings_username_focus)
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, window, _| {
                                            this.late_settings_username_focus.focus(window);
                                        }),
                                    )
                                    .child(if uname_val.is_empty() {
                                        div()
                                            .text_color(theme::module_meta_text(is_dark))
                                            .child("your-username")
                                    } else {
                                        div().child(uname_val)
                                    })
                                    .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                                        let key = event.keystroke.key.as_str();
                                        let mods = event.keystroke.modifiers;
                                        if key == "backspace" {
                                            this.late_settings_username.pop();
                                            cx.notify();
                                        } else if key == "space" {
                                            this.late_settings_username.push(' ');
                                            cx.notify();
                                        } else if !mods.control
                                            && !mods.alt
                                            && !mods.platform
                                            && !mods.function
                                        {
                                            if let Some(ch) = &event.keystroke.key_char {
                                                this.late_settings_username.push_str(ch);
                                                cx.notify();
                                            }
                                        }
                                    }))
                            }),
                    )
                    // Default Room
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme::module_title_text(is_dark))
                                    .child("Default Room"),
                            )
                            .child({
                                let room_val = default_room.clone();
                                div()
                                    .id("late-settings-room")
                                    .px_3()
                                    .py_2()
                                    .rounded_lg()
                                    .bg(input_bg)
                                    .border_1()
                                    .border_color(input_border)
                                    .text_sm()
                                    .text_color(theme::module_title_text(is_dark))
                                    .track_focus(&self.late_settings_default_room_focus)
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, window, _| {
                                            this.late_settings_default_room_focus.focus(window);
                                        }),
                                    )
                                    .child(if room_val.is_empty() {
                                        div()
                                            .text_color(theme::module_meta_text(is_dark))
                                            .child("1")
                                    } else {
                                        div().child(room_val)
                                    })
                                    .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                                        let key = event.keystroke.key.as_str();
                                        let mods = event.keystroke.modifiers;
                                        if key == "backspace" {
                                            this.late_settings_default_room.pop();
                                            cx.notify();
                                        } else if !mods.control
                                            && !mods.alt
                                            && !mods.platform
                                            && !mods.function
                                        {
                                            if let Some(ch) = &event.keystroke.key_char {
                                                if ch.chars().all(|c| c.is_ascii_digit()) {
                                                    this.late_settings_default_room.push_str(ch);
                                                    cx.notify();
                                                }
                                            }
                                        }
                                    }))
                            }),
                    ),
            )
            // Auth section
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme::module_title_text(is_dark))
                            .child("Authentication"),
                    )
                    .child(
                        div()
                            .px_4()
                            .py_3()
                            .rounded_lg()
                            .bg(theme::module_row_bg(is_dark))
                            .border_1()
                            .border_color(theme::module_row_stroke(is_dark))
                            .flex()
                            .justify_between()
                            .items_center()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(theme::module_title_text(is_dark))
                                            .child(if has_token {
                                                "Token: ●●●●●●●●"
                                            } else {
                                                "No token set"
                                            }),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme::module_meta_text(is_dark))
                                            .child(if has_token {
                                                "Authenticated. Use late.logout to revoke."
                                            } else {
                                                "Run late.login <ssh_key_path> in the terminal to authenticate."
                                            }),
                                    ),
                            )
                            .child(if has_token {
                                div()
                                    .cursor_pointer()
                                    .px_3()
                                    .py_1()
                                    .rounded_md()
                                    .bg(theme::module_button_disable_bg(is_dark))
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme::module_button_disable_text(is_dark))
                                    .child("Logout")
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            let ctx = this.execution_context();
                                            let result =
                                                modules::execute_command("late.logout", &[], &ctx);
                                            this.late_settings_feedback = match result {
                                                Ok(Some(msg)) => msg,
                                                Ok(None) => "Logged out.".to_string(),
                                                Err(e) => format!("Error: {e}"),
                                            };
                                            cx.notify();
                                        }),
                                    )
                                    .into_any()
                            } else {
                                div().into_any()
                            }),
                    ),
            )
            // Action buttons
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_3()
                    .items_center()
                    .child(
                        div()
                            .cursor_pointer()
                            .px_4()
                            .py_2()
                            .rounded_lg()
                            .bg(theme::module_button_enable_bg(is_dark))
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme::module_button_enable_text(is_dark))
                            .child("Save")
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _, cx| {
                                    let room: u8 =
                                        this.late_settings_default_room.trim().parse().unwrap_or(1);
                                    let result = LateConfig::load_or_create().and_then(|mut cfg| {
                                        cfg.server_url =
                                            this.late_settings_server_url.trim().to_string();
                                        cfg.username =
                                            this.late_settings_username.trim().to_string();
                                        cfg.default_room = room;
                                        cfg.save()
                                    });
                                    this.late_settings_feedback = match result {
                                        Ok(_) => "Saved.".to_string(),
                                        Err(e) => format!("Error: {e}"),
                                    };
                                    cx.notify();
                                }),
                            ),
                    )
                    .child(
                        div()
                            .cursor_pointer()
                            .px_4()
                            .py_2()
                            .rounded_lg()
                            .bg(palette.row_selected)
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(palette.icon_active)
                            .child("Connect")
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _, cx| {
                                    let ctx = this.execution_context();
                                    let result =
                                        modules::execute_command("late.connect", &[], &ctx);
                                    this.late_settings_feedback = match result {
                                        Ok(Some(msg)) => msg,
                                        Ok(None) => "Connecting…".to_string(),
                                        Err(e) => format!("Error: {e}"),
                                    };
                                    cx.notify();
                                }),
                            ),
                    )
                    .child(
                        div()
                            .cursor_pointer()
                            .px_4()
                            .py_2()
                            .rounded_lg()
                            .bg(theme::module_row_bg(is_dark))
                            .border_1()
                            .border_color(theme::module_row_stroke(is_dark))
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme::module_meta_text(is_dark))
                            .child("Disconnect")
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _, cx| {
                                    let ctx = this.execution_context();
                                    let result =
                                        modules::execute_command("late.disconnect", &[], &ctx);
                                    this.late_settings_feedback = match result {
                                        Ok(Some(msg)) => msg,
                                        Ok(None) => "Disconnected.".to_string(),
                                        Err(e) => format!("Error: {e}"),
                                    };
                                    cx.notify();
                                }),
                            ),
                    ),
            )
            // Feedback
            .child(if feedback.is_empty() {
                div().into_any()
            } else {
                div()
                    .text_sm()
                    .text_color(theme::module_meta_text(is_dark))
                    .child(feedback)
                    .into_any()
            })
    }
}
