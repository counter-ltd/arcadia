use openframe::{div, rgb};
use openframe::{Context, InteractiveElement, IntoElement, ParentElement, Styled};

use arcadia_core::config::ConfigFile;
use arcadia_core::config::modules::ModulesConfig;
use crate::gui::app::ArcadiaRoot;

impl ArcadiaRoot {
    pub fn requirements_modal(&self, cx: &mut Context<Self>, is_dark: bool) -> impl IntoElement {
        let Some((module_name, missing)) = &self.pending_module_enable else {
            return div();
        };
        let requirements = missing.join(", ");

        div()
            .absolute()
            .top_0()
            .left_0()
            .right_0()
            .bottom_0()
            .child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .bottom_0()
                    .bg(rgb(0x000000))
                    .opacity(0.35)
                    .on_mouse_down(
                        openframe::MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.pending_module_enable = None;
                            cx.notify();
                        }),
                    ),
            )
            .child(
                div()
                    .size_full()
                    .flex()
                    .justify_center()
                    .items_center()
                    .child(
                        div()
                            .w_128()
                            .p_5()
                            .rounded_lg()
                            .bg(if is_dark { rgb(0x111827) } else { rgb(0xffffff) })
                            .border_1()
                            .border_color(if is_dark { rgb(0x374151) } else { rgb(0xe2e8f0) })
                            .flex()
                            .flex_col()
                            .gap_3()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(openframe::FontWeight::BOLD)
                                    .text_color(if is_dark { rgb(0xf9fafb) } else { rgb(0x111827) })
                                    .child("Enable with requirements?"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(if is_dark { rgb(0xd1d5db) } else { rgb(0x374151) })
                                    .child(format!(
                                        "To enable {module_name}, Arcadia needs to enable: {requirements}."
                                    )),
                            )
                            .child(
                                div()
                                    .flex()
                                    .gap_2()
                                    .justify_end()
                                    .child(
                                        div()
                                            .px_3()
                                            .py_2()
                                            .rounded_md()
                                            .cursor_pointer()
                                            .bg(if is_dark { rgb(0x374151) } else { rgb(0xe5e7eb) })
                                            .text_color(if is_dark { rgb(0xf3f4f6) } else { rgb(0x1f2937) })
                                            .child("Cancel")
                                            .on_mouse_down(
                                                openframe::MouseButton::Left,
                                                cx.listener(|this, _, _, cx| {
                                                    this.pending_module_enable = None;
                                                    cx.notify();
                                                }),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .px_3()
                                            .py_2()
                                            .rounded_md()
                                            .cursor_pointer()
                                            .bg(rgb(0xdbeafe))
                                            .text_color(rgb(0x1d4ed8))
                                            .child("Enable")
                                            .on_mouse_down(
                                                openframe::MouseButton::Left,
                                                cx.listener(|this, _, _, cx| {
                                                    if let Some((module_name, _)) =
                                                        this.pending_module_enable.clone()
                                                    {
                                                        if let Ok(mut cfg) = ModulesConfig::load_or_create() {
                                                            let _ = cfg.enable_with_requirements(&module_name);
                                                            let _ = cfg.save();
                                                        }
                                                        this.reload_modules();
                                                    }
                                                    this.pending_module_enable = None;
                                                    cx.notify();
                                                }),
                                            ),
                                    ),
                            ),
                    ),
            )
    }
}
