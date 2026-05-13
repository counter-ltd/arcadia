use arcadia_core::config::ai_rules::{all_rules, AiRulesConfig};
use arcadia_core::config::ConfigFile;
use openframe::{
    div, px, Context, FontWeight, InteractiveElement, IntoElement, MouseButton,
    ParentElement, Styled, Window,
};
use openframe::prelude::FluentBuilder as _;

use crate::gui::app::ArcadiaRoot;
use crate::gui::theme;

const GLYPH_PANEL_CONTENT_MAX_W_PX: f32 = 680.0;

impl ArcadiaRoot {
    pub fn ai_rules_panel(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> impl IntoElement {
        let p = theme::theme_palette(cx, is_dark);
        let g_snap = theme::glyph_snapshot(cx);
        let is_glyph = g_snap.is_some();
        let radius = g_snap.map(|g| g.border_radius).unwrap_or(p.radius_md);
        let accent = theme::ui_accent(cx);
        let accent_fg = theme::ui_accent_fg(cx);

        let active_ids = self.ai_active_rule_ids.clone();
        let cfg = AiRulesConfig::load_or_create().unwrap_or_default();
        let all = all_rules(&cfg);

        let mut root = div()
            .w_full()
            .when(is_glyph, |d| d.max_w(px(GLYPH_PANEL_CONTENT_MAX_W_PX)))
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
                            .child("Rules"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(p.content_meta)
                            .child("Per-chat constraints injected into the system prompt. Active rules show as chips in the chat header."),
                    ),
            );

        for rule in &all {
            let rule_id = rule.id.clone();
            let rule_id_toggle = rule_id.clone();
            let is_active = active_ids.contains(&rule_id);
            let (badge_bg, badge_fg) = if is_active {
                (accent, accent_fg)
            } else {
                (p.badge_muted_bg, p.badge_muted_fg)
            };

            let card = div()
                .rounded(px(radius.min(12.0)))
                .border_1()
                .border_color(if is_active { accent } else { p.panel_border })
                .bg(p.panel_bg)
                .p_4()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_0p5()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(p.content_title)
                                        .child(rule.name.clone()),
                                )
                                .child(
                                    div()
                                        .px_2()
                                        .py_0p5()
                                        .rounded(px(4.0))
                                        .bg(badge_bg)
                                        .text_xs()
                                        .text_color(badge_fg)
                                        .child(if is_active { "Active" } else { "Inactive" }),
                                ),
                        )
                        .child(
                            div()
                                .px_3()
                                .py_1()
                                .rounded(px(radius.min(8.0)))
                                .bg(if is_active { p.panel_bg } else { accent })
                                .border_1()
                                .border_color(if is_active { p.panel_border } else { accent })
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(if is_active { p.content_meta } else { accent_fg })
                                .cursor_pointer()
                                .child(if is_active { "Deactivate" } else { "Activate" })
                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                    if this.ai_active_rule_ids.contains(&rule_id_toggle) {
                                        this.ai_active_rule_ids.retain(|id| id != &rule_id_toggle);
                                    } else {
                                        this.ai_active_rule_ids.push(rule_id_toggle.clone());
                                    }
                                    cx.notify();
                                })),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(p.content_meta)
                        .child(rule.system_fragment.clone()),
                )
                .when(!rule.forbidden_tools.is_empty(), |d| {
                    d.child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_1()
                            .flex_wrap()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(p.content_meta)
                                    .child("Blocks: "),
                            )
                            .children(rule.forbidden_tools.iter().map(|t| {
                                div()
                                    .px_1p5()
                                    .py_0p5()
                                    .rounded(px(3.0))
                                    .bg(p.badge_muted_bg)
                                    .text_xs()
                                    .text_color(p.badge_muted_fg)
                                    .child(t.clone())
                            })),
                    )
                });

            root = root.child(card);
        }

        root
    }
}
