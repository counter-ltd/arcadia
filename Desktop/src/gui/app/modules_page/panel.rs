use arcadia_core::config::modules::ModulesConfig;
use openframe::{AnyElement, Window, div, px};
use openframe::{Context, FontWeight, IntoElement, ParentElement, Styled};

use crate::gui::app::list_panel_search::{ListPanelSearchKind, list_panel_row_matches};
use crate::gui::app::ArcadiaRoot;
use crate::gui::theme::{self, GLYPH_PANEL_CONTENT_MAX_W_PX};

impl ArcadiaRoot {
    pub fn modules_panel(&mut self, window: &Window, cx: &mut Context<Self>, is_dark: bool) -> AnyElement {
        if self.active_page_id.as_str() != "global.modules" {
            return div().into_any_element();
        }
        let glyph_cfg = theme::glyph_snapshot(cx);
        let q = self.modules_search_query.trim().to_ascii_lowercase();
        let rows_src: Vec<_> = self
            .module_rows
            .iter()
            .filter(|(module_name, _)| {
                let manifest = ModulesConfig::manifest_for(module_name);
                let version = manifest.map(|m| m.version).unwrap_or("unknown");
                let description = manifest.map(|m| m.description).unwrap_or("");
                list_panel_row_matches(&q, module_name, &[version, description])
            })
            .collect();

        let search_bar = self.list_panel_search_bar(window, cx, is_dark, ListPanelSearchKind::Modules);

        let rows: Vec<_> = rows_src
            .into_iter()
            .map(|(module_name, enabled)| {
                Self::module_row_item(
                    cx,
                    module_name.clone(),
                    *enabled,
                    ModulesConfig::manifest_for(module_name),
                    is_dark,
                )
            })
            .collect();

        let empty_filtered = rows.is_empty()
            && !self.module_rows.is_empty()
            && !self.modules_search_query.trim().is_empty();

        let list_body = if empty_filtered {
            div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .gap_2()
                .py_10()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme::module_title_text(is_dark))
                        .child("No matching modules"),
                )
                .into_any_element()
        } else {
            div().flex().flex_col().gap_3().children(rows).into_any_element()
        };

        if let Some(ref g) = glyph_cfg {
            let radius = g.border_radius.min(12.0);
            div()
                .w_full()
                .flex()
                .justify_center()
                .child(
                    div()
                        .w_full()
                        .max_w(px(GLYPH_PANEL_CONTENT_MAX_W_PX))
                        .p_4()
                        .rounded(px(radius))
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
            div()
                .w_full()
                .p_4()
                .rounded(px(8.))
                .bg(theme::module_panel_bg(is_dark))
                .border_1()
                .border_color(theme::module_panel_stroke(is_dark))
                .flex()
                .flex_col()
                .gap_3()
                .child(search_bar)
                .child(list_body)
                .into_any_element()
        }
    }
}
