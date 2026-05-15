use arcadia_core::config::ai_skills::{all_skills, AiSkillsConfig};
use arcadia_core::config::ConfigFile;
use openframe::prelude::FluentBuilder as _;
use openframe::{
    div, px, AnyElement, Context, FontWeight, InteractiveElement, IntoElement, MouseButton,
    ParentElement, Styled, Window,
};

use crate::gui::app::list_panel_search::{list_panel_row_matches, ListPanelSearchKind};
use crate::gui::app::ArcadiaRoot;
use crate::gui::theme::{self, render_icon, GLYPH_PANEL_CONTENT_MAX_W_PX};

impl ArcadiaRoot {
    pub fn ai_skills_panel(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> AnyElement {
        let p = theme::theme_palette(cx, is_dark);
        let g_snap = theme::glyph_snapshot(cx);
        let radius = g_snap.map(|g| g.border_radius).unwrap_or(p.radius_md);

        let active_ids = self.ai_active_skill_ids.clone();
        let cfg = AiSkillsConfig::load_or_create().unwrap_or_default();
        let all = all_skills(&cfg);
        let q = self.skills_search_query.trim().to_ascii_lowercase();

        let search_bar =
            self.list_panel_search_bar(window, cx, is_dark, ListPanelSearchKind::Skills);

        let filtered: Vec<_> = all
            .into_iter()
            .filter(|s| list_panel_row_matches(&q, &s.name, &[s.system_fragment.as_str()]))
            .collect();

        let rows: Vec<AnyElement> = filtered
            .into_iter()
            .map(|skill| {
                let skill_id = skill.id.clone();
                let skill_id_toggle = skill_id.clone();
                let skill_icon = skill.icon.clone();
                let is_active = active_ids.contains(&skill_id);

                let (badge_bg, badge_fg) = if is_active {
                    (p.accent, p.on_accent)
                } else {
                    (p.surface_elevated, p.ui_subtext)
                };

                let r_track = radius.min(8.0_f32).max(0.0);
                let is_glyph = g_snap.is_some();

                let row_inner = div()
                    .w_full()
                    .px_4()
                    .py_3()
                    .flex()
                    .justify_between()
                    .items_center()
                    .gap_4()
                    .child(
                        div()
                            .flex()
                            .flex_1()
                            .min_w_0()
                            .items_start()
                            .gap_3()
                            .child(
                                render_icon(&skill_icon)
                                    .size_8()
                                    .flex_shrink_0()
                                    .text_color(p.content_title),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_base()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(p.content_title)
                                            .child(skill.name.clone()),
                                    )
                                    .child(
                                        div().flex().items_center().gap_2().child(
                                            div()
                                                .px_2()
                                                .py_0p5()
                                                .when(!is_glyph, |d| d.rounded_full())
                                                .rounded(px(radius.min(12.0)))
                                                .text_xs()
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .bg(badge_bg)
                                                .text_color(badge_fg)
                                                .child(if is_active {
                                                    "Active"
                                                } else {
                                                    "Inactive"
                                                }),
                                        ),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(p.content_body)
                                            .child(skill.system_fragment.clone()),
                                    )
                                    .when(!skill.allowed_tools.is_empty(), |col| {
                                        col.child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .gap_1()
                                                .flex_wrap()
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(p.content_meta)
                                                        .child("Tools: "),
                                                )
                                                .children(skill.allowed_tools.iter().map(|t| {
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
                                    }),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .cursor_pointer()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(if is_active { p.accent } else { p.ui_subtext })
                                    .child(if is_active { "ON" } else { "OFF" }),
                            )
                            .child(if is_active {
                                div()
                                    .w_10()
                                    .h_6()
                                    .px_0p5()
                                    .when(!is_glyph, |d| d.rounded_full())
                                    .rounded(px(r_track))
                                    .border_1()
                                    .border_color(p.border)
                                    .bg(p.accent)
                                    .flex()
                                    .items_center()
                                    .justify_end()
                                    .child(
                                        div()
                                            .w_4()
                                            .h_4()
                                            .when(!is_glyph, |d| d.rounded_full())
                                            .rounded(px(r_track))
                                            .bg(p.on_accent),
                                    )
                            } else {
                                div()
                                    .w_10()
                                    .h_6()
                                    .px_0p5()
                                    .when(!is_glyph, |d| d.rounded_full())
                                    .rounded(px(r_track))
                                    .border_1()
                                    .border_color(p.border)
                                    .bg(p.surface_elevated)
                                    .flex()
                                    .items_center()
                                    .justify_start()
                                    .child(
                                        div()
                                            .w_4()
                                            .h_4()
                                            .when(!is_glyph, |d| d.rounded_full())
                                            .rounded(px(r_track))
                                            .bg(p.toggle_knob_off),
                                    )
                            })
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, _, cx| {
                                    if this.ai_active_skill_ids.contains(&skill_id_toggle) {
                                        this.ai_active_skill_ids
                                            .retain(|id| id != &skill_id_toggle);
                                    } else {
                                        this.ai_active_skill_ids.push(skill_id_toggle.clone());
                                    }
                                    cx.notify();
                                }),
                            ),
                    );

                let row = if let Some(ref g) = g_snap {
                    div()
                        .w_full()
                        .rounded(px(g.border_radius.min(12.0)))
                        .bg(g.surface2)
                        .border_1()
                        .border_color(if is_active { p.accent } else { g.border })
                        .child(row_inner)
                        .into_any_element()
                } else {
                    div()
                        .w_full()
                        .rounded(px(radius.min(12.0)))
                        .bg(p.row_bg)
                        .border_1()
                        .border_color(if is_active { p.accent } else { p.row_border })
                        .child(row_inner)
                        .into_any_element()
                };

                row
            })
            .collect();

        let list_body = div()
            .flex()
            .flex_col()
            .gap_3()
            .children(rows)
            .into_any_element();

        if let Some(ref g) = g_snap {
            let r = g.border_radius.min(12.0);
            div()
                .w_full()
                .flex()
                .justify_center()
                .child(
                    div()
                        .w_full()
                        .max_w(px(GLYPH_PANEL_CONTENT_MAX_W_PX))
                        .p_4()
                        .rounded(px(r))
                        .bg(g.surface)
                        .border_1()
                        .border_color(g.border)
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(search_bar)
                        .child(list_body),
                )
                .into_any_element()
        } else {
            let panel_bg = theme::module_panel_bg(is_dark);
            let panel_border = theme::module_panel_stroke(is_dark);
            div()
                .w_full()
                .p_4()
                .rounded(px(8.0))
                .bg(panel_bg)
                .border_1()
                .border_color(panel_border)
                .flex()
                .flex_col()
                .gap_3()
                .child(search_bar)
                .child(list_body)
                .into_any_element()
        }
    }
}
