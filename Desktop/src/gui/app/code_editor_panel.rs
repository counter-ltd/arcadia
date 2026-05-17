mod commands;
mod dashboard;
mod edit;
mod line_render;
mod segments;
mod text;
mod view;

pub use commands::EditorUndoMap;
pub(crate) use line_render::{code_line_content, LineStyle};
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
        if self.code_editor.tabs.is_empty() || self.code_editor.show_dashboard {
            self.code_editor_dashboard(window, cx, is_dark)
        } else {
            self.code_editor_view(window, cx, is_dark)
        }
    }
}
