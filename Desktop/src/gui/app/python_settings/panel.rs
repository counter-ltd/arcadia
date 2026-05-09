use arcadia_core::modules;
use openframe::{div, rgb, FontWeight, InteractiveElement, IntoElement, MouseButton, ParentElement, Styled};
use openframe::Context;

use crate::gui::app::ArcadiaRoot;
use crate::gui::theme;

impl ArcadiaRoot {
    pub fn python_settings_panel(&self, cx: &mut Context<Self>, is_dark: bool) -> impl IntoElement {
        let header_text = if is_dark { rgb(0xe5e7eb) } else { rgb(0x111827) };
        let subtext = if is_dark { rgb(0x6b7280) } else { rgb(0x9ca3af) };

        let rows = self.python_extension_rows.clone();

        let content = if rows.is_empty() {
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_2()
                .py_10()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(header_text)
                        .child("No extensions loaded"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(subtext)
                        .child("Drop a .py file or a folder with main.py into ~/Arcadia/Extensions/ and reload."),
                )
                .child(Self::python_reload_button(cx, is_dark))
        } else {
            div()
                .flex()
                .flex_col()
                .gap_3()
                .children(rows.into_iter().map(|(name, version, description, enabled)| {
                    Self::python_extension_row(cx, name, version, description, enabled, is_dark)
                }))
                .child(div().pt_2().child(Self::python_reload_button(cx, is_dark)))
        };

        div()
            .w_full()
            .p_4()
            .rounded_lg()
            .bg(theme::module_panel_bg(is_dark))
            .border_1()
            .border_color(theme::module_panel_stroke(is_dark))
            .flex()
            .flex_col()
            .gap_3()
            .child(content)
    }

    fn python_extension_row(
        cx: &mut Context<Self>,
        name: String,
        version: String,
        description: String,
        enabled: bool,
        is_dark: bool,
    ) -> impl IntoElement {
        let state_label = if enabled { "Enabled" } else { "Disabled" };

        div()
            .w_full()
            .px_4()
            .py_3()
            .rounded_lg()
            .bg(theme::module_row_bg(is_dark))
            .border_1()
            .border_color(theme::module_row_stroke(is_dark))
            .flex()
            .justify_between()
            .items_center()
            .gap_4()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_base()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme::module_title_text(is_dark))
                            .child(name.clone()),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::module_meta_text(is_dark))
                                    .child(format!("v{version}")),
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py_0p5()
                                    .rounded_full()
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .bg(if enabled {
                                        theme::module_state_enabled_bg(is_dark)
                                    } else {
                                        theme::module_state_disabled_bg(is_dark)
                                    })
                                    .text_color(if enabled {
                                        theme::module_state_enabled_text(is_dark)
                                    } else {
                                        theme::module_state_disabled_text(is_dark)
                                    })
                                    .child(state_label),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::module_description_text(is_dark))
                            .child(description),
                    ),
            )
            .child(
                // Toggle switch — mirrors the modules page pattern.
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_2()
                    .py_1()
                    .rounded_full()
                    .cursor_pointer()
                    .bg(if enabled {
                        theme::module_state_enabled_bg(is_dark)
                    } else {
                        theme::module_state_disabled_bg(is_dark)
                    })
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(if enabled {
                                theme::module_state_enabled_text(is_dark)
                            } else {
                                theme::module_state_disabled_text(is_dark)
                            })
                            .child(if enabled { "ON" } else { "OFF" }),
                    )
                    .child(if enabled {
                        div()
                            .w_10()
                            .h_6()
                            .px_0p5()
                            .rounded_full()
                            .border_1()
                            .border_color(theme::module_row_stroke(is_dark))
                            .bg(theme::module_button_enable_bg(is_dark))
                            .flex()
                            .items_center()
                            .justify_end()
                            .child(
                                div()
                                    .w_4()
                                    .h_4()
                                    .rounded_full()
                                    .bg(theme::module_button_enable_text(is_dark)),
                            )
                    } else {
                        div()
                            .w_10()
                            .h_6()
                            .px_0p5()
                            .rounded_full()
                            .border_1()
                            .border_color(theme::module_row_stroke(is_dark))
                            .bg(theme::module_panel_stroke(is_dark))
                            .flex()
                            .items_center()
                            .justify_start()
                            .child(
                                div()
                                    .w_4()
                                    .h_4()
                                    .rounded_full()
                                    .bg(if is_dark { rgb(0xd1d5db) } else { rgb(0xf8fafc) }),
                            )
                    })
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            let ctx = this.execution_context();
                            let token = if enabled {
                                "python-host.extension-disable"
                            } else {
                                "python-host.extension-enable"
                            };
                            let _ = modules::execute_command(token, &[name.as_str()], &ctx);
                            this.reload_python_extensions();
                            cx.notify();
                        }),
                    ),
            )
    }

    fn python_reload_button(cx: &mut Context<Self>, is_dark: bool) -> impl IntoElement {
        div()
            .cursor_pointer()
            .px_4()
            .py_2()
            .rounded_lg()
            .bg(theme::module_button_enable_bg(is_dark))
            .text_sm()
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(theme::module_button_enable_text(is_dark))
            .child("Reload Extensions")
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    let ctx = this.execution_context();
                    let _ = modules::execute_command("python-host.reload", &[], &ctx);
                    this.reload_python_extensions();
                    cx.notify();
                }),
            )
    }
}
