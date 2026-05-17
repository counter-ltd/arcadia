mod block_render;
mod block_shape;
mod canvas;
mod dashboard;
mod inspector;
mod palette;

use openframe::{AnyElement, Context, Window};

use crate::gui::app::ArcadiaRoot;

impl ArcadiaRoot {
    pub fn visual_editor_panel(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> AnyElement {
        if self.visual_editor_tabs.is_empty() || self.visual_editor_show_dashboard {
            self.visual_editor_dashboard(window, cx, is_dark)
        } else {
            self.visual_editor_canvas(window, cx, is_dark)
        }
    }
}
