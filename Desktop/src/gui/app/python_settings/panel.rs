use arcadia_core::modules;
use openframe::{
    AnyElement, IntoElement, InteractiveElement, ParentElement, Styled, Window, div, px,
};
use openframe::{Context, FontWeight, MouseButton};
use openframe::prelude::FluentBuilder as _;

use crate::gui::app::list_panel_search::{ListPanelSearchKind, list_panel_row_matches};
use crate::gui::app::ArcadiaRoot;
use crate::gui::theme::{self, GLYPH_PANEL_CONTENT_MAX_W_PX};

impl ArcadiaRoot {
    pub fn python_settings_panel(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> AnyElement {
        let p = theme::theme_palette(cx, is_dark);
        let g_snap = theme::glyph_snapshot(cx);
        let panel_radius = g_snap.map(|g| g.border_radius).unwrap_or(p.radius_md);

        let all_rows = self.python_extension_rows.clone();
        let none_loaded = all_rows.is_empty();
        let q = self.extensions_search_query.trim().to_ascii_lowercase();

        let search_bar = self.list_panel_search_bar(window, cx, is_dark, ListPanelSearchKind::Extensions);

        let filtered: Vec<_> = all_rows
            .into_iter()
            .filter(|(name, version, description, _)| {
                list_panel_row_matches(&q, name, &[version.as_str(), description.as_str()])
            })
            .collect();

        let content = if none_loaded {
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
                        .text_color(p.content_title)
                        .child("No extensions loaded"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(p.content_meta)
                        .child("Drop a .py file or a folder with main.py into ~/Arcadia/Extensions/ and reload."),
                )
                .into_any_element()
        } else if filtered.is_empty() && !self.extensions_search_query.trim().is_empty() {
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
                        .text_color(p.content_title)
                        .child("No matching extensions"),
                )
                .into_any_element()
        } else {
            div()
                .flex()
                .flex_col()
                .gap_3()
                .children(filtered.into_iter().map(|(name, version, description, enabled)| {
                    Self::python_extension_row(
                        cx,
                        name,
                        version,
                        description,
                        enabled,
                        is_dark,
                        panel_radius,
                    )
                }))
                .into_any_element()
        };

        if let Some(g) = theme::active_glyph(cx) {
            div()
                .w_full()
                .flex()
                .justify_center()
                .child(
                    div()
                        .w_full()
                        .max_w(px(GLYPH_PANEL_CONTENT_MAX_W_PX))
                        .p_4()
                        .rounded(px(panel_radius.min(12.0)))
                        .bg(g.surface)
                        .border_1()
                        .border_color(g.border)
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(search_bar)
                        .child(content),
                )
                .into_any_element()
        } else {
            div()
                .w_full()
                .p_4()
                .rounded(px(panel_radius.min(12.0)))
                .bg(p.panel_bg)
                .border_1()
                .border_color(p.panel_border)
                .flex()
                .flex_col()
                .gap_3()
                .child(search_bar)
                .child(content)
                .into_any_element()
        }
    }

    fn python_extension_row(
        cx: &mut Context<Self>,
        name: String,
        version: String,
        description: String,
        enabled: bool,
        is_dark: bool,
        border_radius: f32,
    ) -> AnyElement {
        let p = theme::theme_palette(cx, is_dark);
        let is_glyph = theme::glyph_snapshot(cx).is_some();
        let state_label = if enabled { "Enabled" } else { "Disabled" };
        let r_track = border_radius.min(8.0_f32).max(0.0);

        let (badge_bg, badge_fg) = if enabled {
            (p.accent, p.on_accent)
        } else {
            (p.surface_elevated, p.ui_subtext)
        };

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
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_base()
                            .font_weight(FontWeight::BOLD)
                            .text_color(p.content_title)
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
                                    .text_color(p.content_meta)
                                    .child(format!("v{version}")),
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py_0p5()
                                    .when(!is_glyph, |d| d.rounded_full())
                                    .rounded(px(border_radius.min(12.0)))
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .bg(badge_bg)
                                    .text_color(badge_fg)
                                    .child(state_label),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(p.content_body)
                            .child(description),
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
                            .text_color(if enabled {
                                p.accent
                            } else {
                                p.ui_subtext
                            })
                            .child(if enabled { "ON" } else { "OFF" }),
                    )
                    .child(if enabled {
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
                            let ctx = this.execution_context();
                            let token = if enabled {
                                "python-host.extension-disable"
                            } else {
                                "python-host.extension-enable"
                            };
                            let _ = modules::execute_command(token, &[name.as_str()], &ctx);
                            this.reload_python_extensions(cx);
                            cx.notify();
                        }),
                    ),
            );

        if let Some(g) = theme::active_glyph(cx) {
            div()
                .w_full()
                .rounded(px(g.border_radius.min(12.0)))
                .bg(g.surface2)
                .border_1()
                .border_color(g.border)
                .child(row_inner)
                .into_any_element()
        } else {
            div()
                .w_full()
                .rounded(px(border_radius.min(12.0)))
                .bg(p.row_bg)
                .border_1()
                .border_color(p.row_border)
                .child(row_inner)
                .into_any_element()
        }
    }
}
