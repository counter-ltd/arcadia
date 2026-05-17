use arcadia_core::config::ai::AiConfig;
use arcadia_core::config::ConfigFile;
use arcadia_core::modules::ai::is_ai_provider_available;
use openframe::{
    div, px, Context, FontWeight, InteractiveElement, IntoElement, KeyDownEvent, MouseButton,
    ParentElement, Styled, Window,
};

use openframe::prelude::FluentBuilder as _;

use crate::gui::app::ArcadiaRoot;
use crate::gui::theme::{self, GLYPH_PANEL_CONTENT_MAX_W_PX};

impl ArcadiaRoot {
    pub(crate) fn ai_settings_panel(
        &mut self,
        _window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> impl IntoElement {
        let p = theme::theme_palette(cx, is_dark);
        let g_snap = theme::glyph_snapshot(cx);
        let radius = g_snap.map(|g| g.border_radius).unwrap_or(p.radius_md);

        let provider_available = is_ai_provider_available(&self.module_rows);
        let system_prompt = self.ai.default_system_prompt.clone();

        let mut root = div()
            .w_full()
            .when(g_snap.is_some(), |d| {
                d.max_w(px(GLYPH_PANEL_CONTENT_MAX_W_PX))
            })
            .flex()
            .flex_col()
            .gap_6()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_2xl()
                            .font_weight(FontWeight::BOLD)
                            .text_color(p.content_title)
                            .child("AI"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(p.content_meta)
                            .child("AI chat preferences and provider configuration."),
                    ),
            );

        if !provider_available {
            root =
                root.child(
                    div()
                        .rounded(px(radius.min(12.0)))
                        .border_1()
                        .border_color(p.panel_border)
                        .bg(p.panel_bg)
                        .p_4()
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .gap_3()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(p.content_title)
                                        .child("No AI provider configured"),
                                )
                                .child(div().text_xs().text_color(p.content_meta).child(
                                    "Enable an AI provider module to use the chat interface.",
                                )),
                        )
                        .child(
                            div()
                                .px_3()
                                .py_1p5()
                                .rounded(px(radius.min(8.0)))
                                .border_1()
                                .border_color(p.panel_border)
                                .text_sm()
                                .text_color(p.content_body)
                                .cursor_pointer()
                                .child("Open Modules")
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|this, _, _, cx| {
                                        this.active_page_id = "global.modules".to_string();
                                        this.sync_settings_hub_expanded_from_active_page();
                                        cx.notify();
                                    }),
                                ),
                        ),
                );
        }

        root.child(
            div()
                .rounded(px(radius.min(12.0)))
                .border_1()
                .border_color(p.panel_border)
                .bg(p.panel_bg)
                .overflow_hidden()
                .child(
                    div()
                        .px_4()
                        .py_3()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(p.content_title)
                                .child("Default System Prompt"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(p.content_meta)
                                .child(
                                    "Sent to the AI at the start of every new conversation. Leave empty for no system prompt.",
                                ),
                        )
                        .child(
                            div()
                                .w_full()
                                .min_h(px(80.))
                                .p_2()
                                .rounded(px(radius.min(6.0)))
                                .border_1()
                                .border_color(p.panel_border)
                                .bg(if is_dark {
                                    openframe::rgb(0x0f1115)
                                } else {
                                    openframe::rgb(0xffffff)
                                })
                                .text_sm()
                                .text_color(p.content_body)
                                .child(if system_prompt.is_empty() {
                                    div()
                                        .text_color(p.content_meta)
                                        .child("No system prompt…")
                                } else {
                                    div().child(system_prompt.clone())
                                })
                                .on_key_down(cx.listener(|this, ev: &KeyDownEvent, _, cx| {
                                    let key = &ev.keystroke.key;
                                    if key == "backspace" {
                                        this.ai.default_system_prompt.pop();
                                    } else if key == "enter" {
                                        this.ai.default_system_prompt.push('\n');
                                    } else if ev.keystroke.key.len() == 1
                                        && !ev.keystroke.modifiers.platform
                                    {
                                        let ch = ev.keystroke.key.chars().next().unwrap();
                                        if !ch.is_control() {
                                            this.ai.default_system_prompt.push(ch);
                                        }
                                    }
                                    this.save_ai_settings();
                                    cx.notify();
                                })),
                        ),
                ),
        )
    }

    fn save_ai_settings(&self) {
        let mut cfg = AiConfig::load_or_create().unwrap_or_default();
        cfg.default_system_prompt = self.ai.default_system_prompt.clone();
        let _ = cfg.save();
    }
}
