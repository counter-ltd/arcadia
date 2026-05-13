use arcadia_core::config::workspace::WorkspacesConfig;
use arcadia_core::config::ConfigFile;
use openframe::{
    AnyElement, Context, FontWeight, InteractiveElement, IntoElement, MouseButton, ParentElement,
    Styled, Window, div, px,
};

use crate::gui::app::list_panel_search::{ListPanelSearchKind, list_panel_row_matches};
use crate::gui::app::{ArcadiaRoot, WorkspaceCreateDraft};
use crate::gui::theme::{self, GLYPH_PANEL_CONTENT_MAX_W_PX};

impl ArcadiaRoot {
    pub fn workspace_panel(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> AnyElement {
        let p = theme::theme_palette(cx, is_dark);
        let g_snap = theme::glyph_snapshot(cx);
        let panel_radius = g_snap.map(|g| g.border_radius).unwrap_or(p.radius_md);

        let q = self.workspace_search_query.trim().to_ascii_lowercase();
        let search_bar = self.list_panel_search_bar(window, cx, is_dark, ListPanelSearchKind::Workspaces);

        let Ok(cfg) = WorkspacesConfig::load_or_create() else {
            return div()
                .text_sm()
                .text_color(p.content_title)
                .child("Could not load workspace.toml")
                .into_any_element();
        };

        // Filter workspaces by search query (label or path).
        let workspace_rows: Vec<AnyElement> = cfg
            .workspaces
            .into_iter()
            .filter(|ws| list_panel_row_matches(&q, &ws.label, &[ws.path.as_str()]))
            .map(|ws| self.workspace_row(ws, cx, is_dark))
            .collect();

        let empty_message: &'static str = if q.is_empty() {
            "No workspaces added yet."
        } else {
            "No matching workspaces."
        };

        let add_btn = div()
            .px_3()
            .py_1()
            .rounded(px(panel_radius.min(8.0)))
            .bg(p.accent)
            .text_xs()
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(p.on_accent)
            .cursor_pointer()
            .child("+ Add Workspace")
            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                this.workspace_create_draft = Some(WorkspaceCreateDraft {
                    label: String::new(),
                    path: String::new(),
                    error: None,
                });
                cx.notify();
            }));

        let body = div()
            .w_full()
            .flex()
            .flex_col()
            .gap_4()
            // Top bar: search + add button
            .child(
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(div().flex_1().child(search_bar))
                    .child(add_btn),
            )
            // Section header
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::BOLD)
                            .text_color(p.content_title)
                            .child("Workspaces"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(p.content_meta)
                            .child("Registered directories and their scoped permissions. Enabled modules contribute additional permission rows."),
                    ),
            )
            // Workspace list
            .child(if workspace_rows.is_empty() {
                div()
                    .text_xs()
                    .text_color(p.content_meta)
                    .child(empty_message)
                    .into_any_element()
            } else {
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .children(workspace_rows)
                    .into_any_element()
            });

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
                        .child(body),
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
                .child(body)
                .into_any_element()
        }
    }
}
