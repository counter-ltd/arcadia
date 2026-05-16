mod dashboard;
mod edit;
mod line_render;
mod segments;
mod text;
mod view;

pub use text::detect_language;

use openframe::{AnyElement, Context, Window};

use crate::gui::app::ArcadiaRoot;

impl ArcadiaRoot {
    pub fn code_editor_panel(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> AnyElement {
        if self.code_editor_tabs.is_empty() || self.code_editor_show_dashboard {
            self.code_editor_dashboard(window, cx, is_dark)
        } else {
            self.code_editor_view(window, cx, is_dark)
        }
    }
}
