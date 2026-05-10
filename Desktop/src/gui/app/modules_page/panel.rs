use arcadia_core::config::modules::ModulesConfig;
use openframe::{AnyElement, IntoElement as _, div, px};
use openframe::{Context, IntoElement, ParentElement, Styled, glyph_border};

use crate::gui::app::ArcadiaRoot;
use crate::gui::theme::{self, apply_glyph_border_typography, GLYPH_PANEL_CONTENT_MAX_W_PX};

impl ArcadiaRoot {
    pub fn modules_panel(&self, cx: &mut Context<Self>, is_dark: bool) -> AnyElement {
        if self.active_page_id.as_str() != "global.modules" {
            return div().into_any_element();
        }
        let glyph_cfg = theme::glyph_snapshot(cx);
        let rows: Vec<_> = self
            .module_rows
            .iter()
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
        if let Some(ref g) = glyph_cfg {
            div()
                .w_full()
                .flex()
                .justify_center()
                .child(
                    div()
                        .w_full()
                        .max_w(px(GLYPH_PANEL_CONTENT_MAX_W_PX))
                        .child(
                            apply_glyph_border_typography(
                                &*cx,
                                glyph_border()
                                    .border_color(g.accent)
                                    .bg(g.surface)
                                    .border_chars(g.border_chars),
                            )
                            .child(div().w_full().flex().flex_col().gap_3().children(rows.into_iter())),
                        ),
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
                .children(rows)
                .into_any_element()
        }
    }
}
