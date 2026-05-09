use arcadia_core::config::modules::ModulesConfig;
use openframe::div;
use openframe::{Context, IntoElement, ParentElement, Styled};

use crate::gui::app::ArcadiaRoot;
use crate::gui::theme;

impl ArcadiaRoot {
    pub fn modules_panel(&self, cx: &mut Context<Self>, is_dark: bool) -> impl IntoElement {
        if self.active_page_id.as_str() != "global.modules" {
            return div();
        }
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
            .children(self.module_rows.iter().map(|(module_name, enabled)| {
                Self::module_row_item(
                    cx,
                    module_name.clone(),
                    *enabled,
                    ModulesConfig::manifest_for(module_name),
                    is_dark,
                )
            }))
    }
}
