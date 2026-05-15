use openframe::{div, px, rgb, Context, InteractiveElement, IntoElement, ParentElement, Styled};

use crate::gui::app::ArcadiaRoot;
use crate::gui::theme;
use arcadia_core::config::modules::ModulesConfig;
use arcadia_core::config::ConfigFile;

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
                            .rounded(px(theme::ui_radius(cx)))
                            .bg(theme::ui_surface(cx, is_dark))
                            .border_1()
                            .border_color(theme::ui_border(cx, is_dark))
                            .flex()
                            .flex_col()
                            .gap_3()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(openframe::FontWeight::BOLD)
                                    .text_color(theme::ui_text(cx, is_dark))
                                    .child("Enable with requirements?"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme::ui_subtext(cx, is_dark))
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
                                            .rounded(px(theme::ui_radius(cx)))
                                            .cursor_pointer()
                                            .bg(theme::ui_surface2(cx, is_dark))
                                            .text_color(theme::ui_text(cx, is_dark))
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
                                            .rounded(px(theme::ui_radius(cx)))
                                            .cursor_pointer()
                                            .bg(theme::ui_accent(cx))
                                            .text_color(theme::ui_accent_fg(cx))
                                            .child("Enable")
                                            .on_mouse_down(
                                                openframe::MouseButton::Left,
                                                cx.listener(|this, _, _, cx| {
                                                    if let Some((module_name, _)) =
                                                        this.pending_module_enable.clone()
                                                    {
                                                        if let Ok(mut cfg) =
                                                            ModulesConfig::load_or_create()
                                                        {
                                                            let _ = cfg.enable_with_requirements(
                                                                &module_name,
                                                            );
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
