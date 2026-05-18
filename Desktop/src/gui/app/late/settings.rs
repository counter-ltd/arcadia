use arcadia_core::config::late::LateConfig;
use arcadia_core::config::ConfigFile;
use arcadia_core::modules;
use openframe::{
    div, px, text_input, Context, Element, FontWeight, InteractiveElement, IntoElement,
    MouseButton, ParentElement, Styled, Window,
};

use crate::gui::app::ArcadiaRoot;
use crate::gui::theme;

impl ArcadiaRoot {
    pub(crate) fn late_settings_panel(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> impl IntoElement {
        let server_url = self.late.settings_server_url.clone();
        let username = self.late.settings_username.clone();
        let default_room = self.late.settings_default_room.clone();
        let feedback = self.late.settings_feedback.clone();
        let has_token = LateConfig::load_or_create()
            .map(|c| !c.auth_token.is_empty())
            .unwrap_or(false);
        let palette = theme::nav_accent_palette("violet", is_dark);

        let glyph = theme::glyph_snapshot(cx);
        let text_c = glyph
            .as_ref()
            .map(|g| g.text)
            .unwrap_or_else(|| theme::module_title_text(is_dark));
        let subtext_c = glyph
            .as_ref()
            .map(|g| g.dim)
            .unwrap_or_else(|| theme::module_meta_text(is_dark));
        let row_bg = glyph
            .as_ref()
            .map(|g| g.surface2)
            .unwrap_or_else(|| theme::module_row_bg(is_dark));
        let row_str = glyph
            .as_ref()
            .map(|g| g.border)
            .unwrap_or_else(|| theme::module_row_stroke(is_dark));
        let btn_bg = glyph
            .as_ref()
            .map(|g| g.accent)
            .unwrap_or_else(|| theme::button_positive_bg(is_dark));
        let btn_text = glyph
            .as_ref()
            .map(|g| g.bg)
            .unwrap_or_else(|| theme::button_positive_text(is_dark));
        let dis_bg = glyph
            .as_ref()
            .map(|g| g.surface2)
            .unwrap_or_else(|| theme::button_negative_bg(is_dark));
        let dis_text = glyph
            .as_ref()
            .map(|g| g.dim)
            .unwrap_or_else(|| theme::button_negative_text(is_dark));
        let conn_bg = glyph
            .as_ref()
            .map(|g| g.surface2)
            .unwrap_or(palette.row_selected);
        let conn_text = glyph
            .as_ref()
            .map(|g| g.accent)
            .unwrap_or(palette.icon_active);
        let radius = glyph.as_ref().map(|g| g.border_radius).unwrap_or(8.0);
        let input_bg = glyph
            .as_ref()
            .map(|g| g.surface)
            .unwrap_or_else(|| theme::ui_surface(cx, is_dark));
        let input_border = glyph
            .as_ref()
            .map(|g| g.border)
            .unwrap_or_else(|| theme::ui_border(cx, is_dark));
        let settings_radius = radius;

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
                            .text_color(text_c)
                            .child("Late.sh"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(subtext_c)
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
                                    .text_color(text_c)
                                    .child("Server URL"),
                            )
                            .child({
                                let weak = cx.weak_entity();
                                text_input(
                                    "late-settings-url",
                                    window,
                                    weak,
                                    &server_url,
                                    "https://late.sh",
                                    &self.late.settings_server_url_focus,
                                    text_c,
                                    subtext_c,
                                    |this, new_text, cx| {
                                        this.late.settings_server_url = new_text;
                                        cx.notify();
                                    },
                                )
                                .px_3()
                                .py_2()
                                .rounded(px(settings_radius))
                                .bg(input_bg)
                                .border_1()
                                .border_color(input_border)
                                .text_sm()
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
                                    .text_color(text_c)
                                    .child("Username"),
                            )
                            .child({
                                let weak = cx.weak_entity();
                                text_input(
                                    "late-settings-username",
                                    window,
                                    weak,
                                    &username,
                                    "your-username",
                                    &self.late.settings_username_focus,
                                    text_c,
                                    subtext_c,
                                    |this, new_text, cx| {
                                        this.late.settings_username = new_text;
                                        cx.notify();
                                    },
                                )
                                .px_3()
                                .py_2()
                                .rounded(px(settings_radius))
                                .bg(input_bg)
                                .border_1()
                                .border_color(input_border)
                                .text_sm()
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
                                    .text_color(text_c)
                                    .child("Default Room"),
                            )
                            .child({
                                let weak = cx.weak_entity();
                                text_input(
                                    "late-settings-room",
                                    window,
                                    weak,
                                    &default_room,
                                    "1",
                                    &self.late.settings_default_room_focus,
                                    text_c,
                                    subtext_c,
                                    |this, new_text, cx| {
                                        // Only keep digits
                                        let filtered: String = new_text.chars().filter(|c| c.is_ascii_digit()).collect();
                                        this.late.settings_default_room = filtered;
                                        cx.notify();
                                    },
                                )
                                .px_3()
                                .py_2()
                                .rounded(px(settings_radius))
                                .bg(input_bg)
                                .border_1()
                                .border_color(input_border)
                                .text_sm()
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
                            .text_color(text_c)
                            .child("Authentication"),
                    )
                    .child(
                        div()
                            .px_4()
                            .py_3()
                            .rounded(px(radius))
                            .bg(row_bg)
                            .border_1()
                            .border_color(row_str)
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
                                            .text_color(text_c)
                                            .child(if has_token {
                                                "Token: ●●●●●●●●"
                                            } else {
                                                "No token set"
                                            }),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(subtext_c)
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
                                    .rounded(px(settings_radius))
                                    .bg(dis_bg)
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(dis_text)
                                    .child("Logout")
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            let ctx = this.execution_context();
                                            let result =
                                                modules::execute_command("late.logout", &[], &ctx);
                                            this.late.settings_feedback = match result {
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
                            .rounded(px(radius))
                            .bg(btn_bg)
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(btn_text)
                            .child("Save")
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _, cx| {
                                    let room: u8 =
                                        this.late.settings_default_room.trim().parse().unwrap_or(1);
                                    let result = LateConfig::load_or_create().and_then(|mut cfg| {
                                        cfg.server_url =
                                            this.late.settings_server_url.trim().to_string();
                                        cfg.username =
                                            this.late.settings_username.trim().to_string();
                                        cfg.default_room = room;
                                        cfg.save()
                                    });
                                    this.late.settings_feedback = match result {
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
                            .rounded(px(radius))
                            .bg(conn_bg)
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(conn_text)
                            .child("Connect")
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _, cx| {
                                    let ctx = this.execution_context();
                                    let result =
                                        modules::execute_command("late.connect", &[], &ctx);
                                    this.late.settings_feedback = match result {
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
                            .rounded(px(radius))
                            .bg(row_bg)
                            .border_1()
                            .border_color(row_str)
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(subtext_c)
                            .child("Disconnect")
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _, cx| {
                                    let ctx = this.execution_context();
                                    let result =
                                        modules::execute_command("late.disconnect", &[], &ctx);
                                    this.late.settings_feedback = match result {
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
                    .text_color(subtext_c)
                    .child(feedback)
                    .into_any()
            })
    }
}
