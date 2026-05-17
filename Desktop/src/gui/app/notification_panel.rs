use arcadia_core::config::notifications::NotificationsConfig;
use arcadia_core::config::ConfigFile;
use arcadia_core::modules;
use openframe::prelude::FluentBuilder as _;
use openframe::{
    div, px, Context, FontWeight, InteractiveElement, IntoElement, MouseButton,
    ParentElement, SharedString, Styled, Window,
};

use crate::gui::app::ArcadiaRoot;
use crate::gui::theme;

impl ArcadiaRoot {
    pub(crate) fn notification_panel(
        &mut self,
        _window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> impl IntoElement {
        let entries = NotificationsConfig::load_or_create()
            .map(|c| c.entries.clone())
            .unwrap_or_default();
        let unread = entries.iter().filter(|e| !e.read).count();

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
        let radius = glyph.as_ref().map(|g| g.border_radius).unwrap_or(8.0);
        let accent_dot = theme::nav_accent_palette("amber", is_dark).icon_active;

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_6()
            // Header row
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .items_center()
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
                                    .child("Notifications"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(subtext_c)
                                    .child(if unread == 0 {
                                        "No unread notifications.".to_string()
                                    } else {
                                        format!("{unread} unread")
                                    }),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_2()
                            .when(!entries.is_empty(), |d| {
                                d.child(
                                    div()
                                        .cursor_pointer()
                                        .px_3()
                                        .py_1()
                                        .rounded(px(radius))
                                        .bg(btn_bg)
                                        .text_xs()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(btn_text)
                                        .child("Mark all read")
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(|this, _, _, cx| {
                                                let ctx = this.execution_context();
                                                let _ = modules::execute_command(
                                                    "notification.mark_all_read",
                                                    &[],
                                                    &ctx,
                                                );
                                                this.notification_unread_count = 0;
                                                cx.notify();
                                            }),
                                        ),
                                )
                            })
                            .when(!entries.is_empty(), |d| {
                                d.child(
                                    div()
                                        .cursor_pointer()
                                        .px_3()
                                        .py_1()
                                        .rounded(px(radius))
                                        .bg(dis_bg)
                                        .text_xs()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(dis_text)
                                        .child("Clear all")
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(|this, _, _, cx| {
                                                let ctx = this.execution_context();
                                                let _ = modules::execute_command(
                                                    "notification.clear",
                                                    &[],
                                                    &ctx,
                                                );
                                                this.notification_unread_count = 0;
                                                cx.notify();
                                            }),
                                        ),
                                )
                            }),
                    ),
            )
            // Notification list
            .child(if entries.is_empty() {
                div()
                    .px_4()
                    .py_8()
                    .rounded(px(radius))
                    .bg(row_bg)
                    .border_1()
                    .border_color(row_str)
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_sm()
                    .text_color(subtext_c)
                    .child("No notifications yet.")
                    .into_any_element()
            } else {
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .children(entries.into_iter().map(|entry| {
                        let entry_id = entry.id.clone();
                        let is_unread = !entry.read;

                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0);
                        let age_secs = now.saturating_sub(entry.timestamp);
                        let time_str = if age_secs < 60 {
                            "just now".to_string()
                        } else if age_secs < 3600 {
                            format!("{}m ago", age_secs / 60)
                        } else if age_secs < 86400 {
                            format!("{}h ago", age_secs / 3600)
                        } else {
                            format!("{}d ago", age_secs / 86400)
                        };

                        div()
                            .id(SharedString::from(entry_id.clone()))
                            .cursor_pointer()
                            .px_4()
                            .py_3()
                            .rounded(px(radius))
                            .bg(row_bg)
                            .border_1()
                            .border_color(row_str)
                            .flex()
                            .flex_row()
                            .items_start()
                            .gap_3()
                            // Unread indicator dot
                            .child(
                                div()
                                    .w(px(8.))
                                    .h(px(8.))
                                    .mt(px(5.))
                                    .flex_shrink_0()
                                    .rounded_full()
                                    .when(is_unread, |d| d.bg(accent_dot))
                                    .when(!is_unread, |d| d.bg(row_str)),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_0p5()
                                    .flex_1()
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .justify_between()
                                            .items_baseline()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(if is_unread {
                                                        FontWeight::SEMIBOLD
                                                    } else {
                                                        FontWeight::NORMAL
                                                    })
                                                    .text_color(text_c)
                                                    .child(entry.title.clone()),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(subtext_c)
                                                    .child(time_str),
                                            ),
                                    )
                                    .when(!entry.body.is_empty(), |d| {
                                        d.child(
                                            div()
                                                .text_xs()
                                                .text_color(subtext_c)
                                                .child(entry.body.clone()),
                                        )
                                    })
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(subtext_c)
                                            .child(format!("from {}", entry.source)),
                                    ),
                            )
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, _, cx| {
                                    let ctx = this.execution_context();
                                    let _ = modules::execute_command(
                                        "notification.mark_read",
                                        &[entry_id.as_str()],
                                        &ctx,
                                    );
                                    this.notification_unread_count =
                                        this.notification_unread_count.saturating_sub(1);
                                    cx.notify();
                                }),
                            )
                    }))
                    .into_any_element()
            })
    }
}
