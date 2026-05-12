use arcadia_core::config::modules::MODULE_REGISTRY;
use arcadia_core::config::workspace::WorkspaceEntry;
use arcadia_core::modules;
use openframe::{
    AnyElement, Context, FontWeight, InteractiveElement, IntoElement, MouseButton, ParentElement,
    Styled, div, px,
};

use crate::gui::app::ArcadiaRoot;
use crate::gui::theme::{self, GLYPH_PANEL_CONTENT_MAX_W_PX};

impl ArcadiaRoot {
    pub fn workspace_row(
        &self,
        ws: WorkspaceEntry,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> AnyElement {
        let p = theme::theme_palette(cx, is_dark);
        let g_snap = theme::glyph_snapshot(cx);
        let panel_radius = g_snap.map(|g| g.border_radius).unwrap_or(p.radius_md);

        let ws_id = ws.id.clone();
        let ws_id_remove = ws.id.clone();

        // Collect workspace permission defs from all enabled modules.
        let enabled_modules: Vec<&str> = self
            .module_rows
            .iter()
            .filter(|(_, enabled)| *enabled)
            .map(|(name, _)| name.as_str())
            .collect();

        let mut perm_rows: Vec<AnyElement> = Vec::new();
        for manifest in MODULE_REGISTRY {
            if manifest.workspace_permissions.is_empty() {
                continue;
            }
            if !enabled_modules.contains(&manifest.name) {
                continue;
            }
            for def in manifest.workspace_permissions {
                let granted = ws.granted_permissions.iter().any(|p| p == def.id);
                let perm_id = def.id.to_string();
                let ws_id_c = ws_id.clone();
                let verb = if granted { "workspace.revoke" } else { "workspace.grant" };
                perm_rows.push(
                    div()
                        .w_full()
                        .flex()
                        .justify_between()
                        .items_center()
                        .py_2()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(p.content_title)
                                        .child(def.title.to_string()),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(p.content_meta)
                                        .child(perm_id.clone()),
                                ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(if granted { p.accent } else { p.ui_subtext })
                                .cursor_pointer()
                                .child(if granted { "ON" } else { "OFF" })
                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                    let ctx = this.execution_context();
                                    let _ = modules::execute_command(
                                        verb,
                                        &[ws_id_c.as_str(), perm_id.as_str()],
                                        &ctx,
                                    );
                                    cx.notify();
                                })),
                        )
                        .into_any_element(),
                );
            }
        }

        div()
            .w_full()
            .max_w(px(GLYPH_PANEL_CONTENT_MAX_W_PX))
            .rounded(px(panel_radius))
            .border_1()
            .border_color(p.border)
            .bg(p.surface_elevated)
            .overflow_hidden()
            // Header
            .child(
                div()
                    .w_full()
                    .px_4()
                    .py_3()
                    .flex()
                    .justify_between()
                    .items_center()
                    .border_b_1()
                    .border_color(p.border)
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
                                    .child(ws.label.clone()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(p.content_meta)
                                    .child(ws.path.clone()),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(p.ui_subtext)
                            .cursor_pointer()
                            .px_2()
                            .py_1()
                            .rounded(px(panel_radius.min(6.0)))
                            .child("Remove")
                            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                let ctx = this.execution_context();
                                let _ = modules::execute_command(
                                    "workspace.remove",
                                    &[ws_id_remove.as_str()],
                                    &ctx,
                                );
                                cx.notify();
                            })),
                    ),
            )
            // Permission rows
            .child(
                div()
                    .w_full()
                    .px_4()
                    .flex()
                    .flex_col()
                    .children(perm_rows),
            )
            .into_any_element()
    }
}
