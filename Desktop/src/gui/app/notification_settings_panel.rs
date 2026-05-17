use arcadia_core::config::modules::ModulesConfig;
use arcadia_core::config::notifications::{NotificationDestination, NotificationsConfig};
use arcadia_core::config::permissions::{PermissionSubject, PermissionsConfig};
use arcadia_core::platform;
use arcadia_core::config::ConfigFile;
use arcadia_core::modules::python_registry;
use openframe::prelude::FluentBuilder as _;
use openframe::{
    div, px, Context, FontWeight, InteractiveElement, IntoElement, KeyDownEvent,
    MouseButton, ParentElement, Styled, Window,
};

use crate::gui::app::text_input_caret::{text_with_trailing_caret, TEXT_INPUT_CARET_CHAR};
use crate::gui::app::ArcadiaRoot;
use crate::gui::theme;

impl ArcadiaRoot {
    pub(crate) fn notification_settings_panel(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> impl IntoElement {
        let feedback = self.notification_settings_feedback.clone();
        let max_count_draft = self.notification_max_count_draft.clone();

        let notif_cfg = NotificationsConfig::load_or_create().unwrap_or_default();
        let current_max = notif_cfg.max_count;
        let current_destinations = notif_cfg.destinations.clone();
        let trust_all = notif_cfg.trust_all_sources;
        let dest_open = self.notification_dest_open;
        let perms_cfg = PermissionsConfig::load_or_create().unwrap_or_default();
        let receive_on = perms_cfg.global_allowed("notifications.receive");

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
        let input_bg = glyph
            .as_ref()
            .map(|g| g.surface)
            .unwrap_or_else(|| theme::ui_surface(cx, is_dark));
        let input_border = glyph
            .as_ref()
            .map(|g| g.border)
            .unwrap_or_else(|| theme::ui_border(cx, is_dark));

        // Collect enabled native modules + python extensions as notification sources
        let module_cfg = ModulesConfig::load_or_create().unwrap_or_default();
        let mut sources: Vec<(String, bool)> = module_cfg
            .modules
            .iter()
            .filter(|(_, enabled)| **enabled)
            .map(|(name, _)| {
                let subj = PermissionSubject::module(name.as_str());
                let send_granted = if trust_all {
                    !perms_cfg.subject_explicitly_denied(&subj, "notifications.send")
                } else {
                    perms_cfg.subject_grants(&subj, "notifications.send")
                };
                (name.clone(), send_granted)
            })
            .collect();

        // Python extensions
        let py_rows = python_registry::list_modules();
        for (ext_id, _, _, enabled, _, _) in &py_rows {
            if !enabled {
                continue;
            }
            let subj = PermissionSubject::python(ext_id.as_str());
            let send_granted = if trust_all {
                !perms_cfg.subject_explicitly_denied(&subj, "notifications.send")
            } else {
                perms_cfg.subject_grants(&subj, "notifications.send")
            };
            sources.push((format!("python:{ext_id}"), send_granted));
        }

        sources.sort_by(|a, b| a.0.cmp(&b.0));

        let max_count_focused = self.notification_max_count_focus.is_focused(window);
        let blink = self.text_caret_blink_visible;

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
                            .child("Notifications"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(subtext_c)
                            .child("Configure storage and per-source send permissions."),
                    ),
            )
            // Global receive toggle
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
                            .child("Global receive"),
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
                                    .gap_0p5()
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(text_c)
                                            .child("Receive notifications"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(subtext_c)
                                            .child("Global gate — off disables all notification storage."),
                                    ),
                            )
                            .child(
                                div()
                                    .cursor_pointer()
                                    .px_3()
                                    .py_1()
                                    .rounded(px(radius))
                                    .bg(if receive_on { btn_bg } else { dis_bg })
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(if receive_on { btn_text } else { dis_text })
                                    .child(if receive_on { "On" } else { "Off" })
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |this, _, _, cx| {
                                            if let Ok(mut cfg) = PermissionsConfig::load_or_create() {
                                                let _ = cfg
                                                    .set_global("notifications.receive", !receive_on);
                                                if let Err(e) = cfg.save() {
                                                    this.notification_settings_feedback =
                                                        format!("Error: {e}");
                                                    cx.notify();
                                                    return;
                                                }
                                            }
                                            this.notification_settings_feedback =
                                                "Saved.".to_string();
                                            cx.notify();
                                        }),
                                    ),
                            ),
                    ),
            )
            // Trust all sources toggle
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
                            .child("Trust all sources"),
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
                                    .gap_0p5()
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(text_c)
                                            .child("Trust all sources"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(subtext_c)
                                            .child("Automatically grant send permission to every enabled module and extension."),
                                    ),
                            )
                            .child(
                                div()
                                    .cursor_pointer()
                                    .px_3()
                                    .py_1()
                                    .rounded(px(radius))
                                    .bg(if trust_all { btn_bg } else { dis_bg })
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(if trust_all { btn_text } else { dis_text })
                                    .child(if trust_all { "On" } else { "Off" })
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |this, _, _, cx| {
                                            let result = NotificationsConfig::load_or_create()
                                                .and_then(|mut cfg| {
                                                    cfg.trust_all_sources = !trust_all;
                                                    cfg.save()
                                                });
                                            this.notification_settings_feedback = match result {
                                                Ok(_) => {
                                                    if !trust_all {
                                                        // Ensure global send gate is open
                                                        if let Ok(mut perms) = PermissionsConfig::load_or_create() {
                                                            let _ = perms.set_global("notifications.send", true);
                                                            let _ = perms.save();
                                                        }
                                                    }
                                                    "Saved.".to_string()
                                                }
                                                Err(e) => format!("Error: {e}"),
                                            };
                                            cx.notify();
                                        }),
                                    ),
                            ),
                    ),
            )
            // Max count field
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
                            .child("Maximum stored notifications"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_3()
                            .items_center()
                            .child(
                                div()
                                    .id("notif-max-count")
                                    .w(px(120.))
                                    .px_3()
                                    .py_2()
                                    .rounded(px(radius))
                                    .bg(input_bg)
                                    .border_1()
                                    .border_color(input_border)
                                    .text_sm()
                                    .text_color(text_c)
                                    .track_focus(&self.notification_max_count_focus)
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, window, _| {
                                            this.notification_max_count_focus.focus(window);
                                        }),
                                    )
                                    .child(if max_count_draft.is_empty() {
                                        if max_count_focused && blink {
                                            div()
                                                .text_color(text_c)
                                                .child(TEXT_INPUT_CARET_CHAR.to_string())
                                        } else {
                                            div()
                                                .text_color(subtext_c)
                                                .child(current_max.to_string())
                                        }
                                    } else {
                                        div().child(text_with_trailing_caret(
                                            max_count_draft.as_str(),
                                            max_count_focused,
                                            blink,
                                        ))
                                    })
                                    .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                                        let key = event.keystroke.key.as_str();
                                        let mods = event.keystroke.modifiers;
                                        if key == "backspace" {
                                            this.notification_max_count_draft.pop();
                                            cx.notify();
                                        } else if !mods.control
                                            && !mods.alt
                                            && !mods.platform
                                            && !mods.function
                                        {
                                            if let Some(ch) = &event.keystroke.key_char {
                                                if ch.chars().all(|c| c.is_ascii_digit()) {
                                                    this.notification_max_count_draft
                                                        .push_str(ch);
                                                    cx.notify();
                                                }
                                            }
                                        }
                                    })),
                            )
                            .child(
                                div()
                                    .cursor_pointer()
                                    .px_3()
                                    .py_1()
                                    .rounded(px(radius))
                                    .bg(btn_bg)
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(btn_text)
                                    .child("Save")
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            let n: usize = this
                                                .notification_max_count_draft
                                                .trim()
                                                .parse()
                                                .unwrap_or(0);
                                            if n == 0 {
                                                this.notification_settings_feedback =
                                                    "Enter a number greater than 0.".to_string();
                                                cx.notify();
                                                return;
                                            }
                                            let result =
                                                NotificationsConfig::load_or_create().and_then(
                                                    |mut cfg| {
                                                        cfg.max_count = n;
                                                        cfg.save()
                                                    },
                                                );
                                            this.notification_settings_feedback = match result {
                                                Ok(_) => {
                                                    this.notification_max_count_draft.clear();
                                                    "Saved.".to_string()
                                                }
                                                Err(e) => format!("Error: {e}"),
                                            };
                                            cx.notify();
                                        }),
                                    ),
                            ),
                    ),
            )
            // Test notification
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
                            .child("Test"),
                    )
                    .child(
                        div()
                            .cursor_pointer()
                            .px_3()
                            .py_1()
                            .rounded(px(radius))
                            .bg(btn_bg)
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(btn_text)
                            .child("Send test notification")
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _, cx| {
                                    let result = NotificationsConfig::load_or_create()
                                        .and_then(|mut cfg| {
                                            let id = std::time::SystemTime::now()
                                                .duration_since(std::time::UNIX_EPOCH)
                                                .map(|d| format!("test-{}", d.as_millis()))
                                                .unwrap_or_else(|_| "test".to_string());
                                            let destinations = cfg.destinations.clone();
                                            cfg.post(
                                                id,
                                                "Test Notification".to_string(),
                                                "This is a test notification from Arcadia.".to_string(),
                                                "notification-settings".to_string(),
                                            );
                                            cfg.save().map(|_| destinations)
                                        });
                                    match result {
                                        Ok(destinations) => {
                                            if destinations.contains(&NotificationDestination::System) {
                                                platform::send_system_notification(
                                                    "Test Notification",
                                                    "This is a test notification from Arcadia.",
                                                );
                                            }
                                            this.notification_unread_count = NotificationsConfig::load_or_create()
                                                .map(|c| c.unread_count())
                                                .unwrap_or(this.notification_unread_count + 1);
                                            this.start_notification_preview(
                                                "Test Notification".to_string(),
                                            );
                                            this.notification_settings_feedback =
                                                "Test notification sent.".to_string();
                                        }
                                        Err(e) => {
                                            this.notification_settings_feedback =
                                                format!("Error: {e}");
                                        }
                                    }
                                    cx.notify();
                                }),
                            ),
                    ),
            )
            // Notification destination dropdown
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
                            .child("Notification Destination"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            // Dropdown trigger button
                            .child(
                                div()
                                    .cursor_pointer()
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
                                            .text_sm()
                                            .text_color(text_c)
                                            .child(if current_destinations.is_empty() {
                                                "None selected".to_string()
                                            } else {
                                                current_destinations
                                                    .iter()
                                                    .map(|d| d.label())
                                                    .collect::<Vec<_>>()
                                                    .join(", ")
                                            }),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(subtext_c)
                                            .child(if dest_open { "▲" } else { "▼" }),
                                    )
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.notification_dest_open =
                                                !this.notification_dest_open;
                                            cx.notify();
                                        }),
                                    ),
                            )
                            // Expanded option list
                            .when(dest_open, |d| {
                                d.child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .children(
                                            [NotificationDestination::System]
                                                .into_iter()
                                                .map(|dest| {
                                                    let selected =
                                                        current_destinations.contains(&dest);
                                                    let dest_clone = dest.clone();
                                                    let label = dest.label();
                                                    let description = dest.description();
                                                    div()
                                                        .cursor_pointer()
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
                                                                .gap_0p5()
                                                                .child(
                                                                    div()
                                                                        .text_sm()
                                                                        .text_color(text_c)
                                                                        .child(label),
                                                                )
                                                                .child(
                                                                    div()
                                                                        .text_xs()
                                                                        .text_color(subtext_c)
                                                                        .child(description),
                                                                ),
                                                        )
                                                        .child(
                                                            div()
                                                                .cursor_pointer()
                                                                .px_3()
                                                                .py_1()
                                                                .rounded(px(radius))
                                                                .bg(if selected {
                                                                    btn_bg
                                                                } else {
                                                                    dis_bg
                                                                })
                                                                .text_xs()
                                                                .font_weight(FontWeight::SEMIBOLD)
                                                                .text_color(if selected {
                                                                    btn_text
                                                                } else {
                                                                    dis_text
                                                                })
                                                                .child(if selected {
                                                                    "On"
                                                                } else {
                                                                    "Off"
                                                                })
                                                                .on_mouse_down(
                                                                    MouseButton::Left,
                                                                    cx.listener(
                                                                        move |this, _, _, cx| {
                                                                            let result = NotificationsConfig::load_or_create()
                                                                                .and_then(|mut cfg| {
                                                                                    if selected {
                                                                                        cfg.destinations.retain(|d| d != &dest_clone);
                                                                                    } else {
                                                                                        cfg.destinations.push(dest_clone.clone());
                                                                                    }
                                                                                    cfg.save()
                                                                                });
                                                                            this.notification_settings_feedback = match result {
                                                                                Ok(_) => "Saved.".to_string(),
                                                                                Err(e) => format!("Error: {e}"),
                                                                            };
                                                                            cx.notify();
                                                                        },
                                                                    ),
                                                                ),
                                                        )
                                                }),
                                        ),
                                )
                            }),
                    ),
            )
            // Per-source send permissions
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_0p5()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(text_c)
                                    .child("Source permissions"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(subtext_c)
                                    .child(if trust_all {
                                        "Trust all sources is on — sources are allowed by default. Toggle to explicitly deny individual sources."
                                    } else {
                                        "Each enabled module and extension must be granted notifications.send before it can post notifications."
                                    }),
                            ),
                    )
                    .child(if sources.is_empty() {
                        div()
                            .px_4()
                            .py_3()
                            .rounded(px(radius))
                            .bg(row_bg)
                            .border_1()
                            .border_color(row_str)
                            .text_sm()
                            .text_color(subtext_c)
                            .child("No modules or extensions are currently enabled.")
                            .into_any_element()
                    } else {
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .children(sources.into_iter().map(|(source_id, granted)| {
                                let sid = source_id.clone();
                                let is_python = sid.starts_with("python:");
                                let display = if is_python {
                                    sid.trim_start_matches("python:").to_string()
                                } else {
                                    sid.clone()
                                };
                                let kind = if is_python { "extension" } else { "module" };

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
                                            .gap_0p5()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(text_c)
                                                    .child(display),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(subtext_c)
                                                    .child(kind.to_string()),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .cursor_pointer()
                                            .px_3()
                                            .py_1()
                                            .rounded(px(radius))
                                            .bg(if granted { btn_bg } else { dis_bg })
                                            .text_xs()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(if granted { btn_text } else { dis_text })
                                            .child(if granted { "Allowed" } else { "Denied" })
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(move |this, _, _, cx| {
                                                    let subject = if sid.starts_with("python:") {
                                                        PermissionSubject::python(
                                                            sid.trim_start_matches("python:"),
                                                        )
                                                    } else {
                                                        PermissionSubject::module(&sid)
                                                    };
                                                    let result =
                                                        PermissionsConfig::load_or_create()
                                                            .and_then(|mut cfg| {
                                                                if trust_all {
                                                                    // trust_all mode: toggle explicit denial.
                                                                    // granted=true means not denied → deny it.
                                                                    // granted=false means denied → remove denial.
                                                                    let _ = cfg.set_subject_grant(
                                                                        &subject,
                                                                        "notifications.send",
                                                                        !granted,
                                                                    );
                                                                } else {
                                                                    // Normal mode: toggle explicit grant.
                                                                    // Ensure global send gate is open when granting.
                                                                    if !granted {
                                                                        let _ = cfg.set_global(
                                                                            "notifications.send",
                                                                            true,
                                                                        );
                                                                    }
                                                                    let _ = cfg.set_subject_grant(
                                                                        &subject,
                                                                        "notifications.send",
                                                                        !granted,
                                                                    );
                                                                }
                                                                cfg.save()
                                                            });
                                                    this.notification_settings_feedback =
                                                        match result {
                                                            Ok(_) => "Saved.".to_string(),
                                                            Err(e) => format!("Error: {e}"),
                                                        };
                                                    cx.notify();
                                                }),
                                            ),
                                    )
                            }))
                            .into_any_element()
                    }),
            )
            // Feedback
            .when(!feedback.is_empty(), |d| {
                d.child(
                    div()
                        .text_sm()
                        .text_color(subtext_c)
                        .child(feedback),
                )
            })
    }
}
