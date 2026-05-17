use arcadia_core::modules::python_registry::{self, extension_enabled, list_style_tokens};
use arcadia_core::modules;
use openframe::prelude::FluentBuilder as _;
use openframe::{
    div, px, Context, FontWeight, InteractiveElement, IntoElement, MouseButton, ParentElement,
    Styled, Window,
};

use crate::gui::app::ArcadiaRoot;
use crate::gui::theme;

impl ArcadiaRoot {
    pub(crate) fn extension_token_settings_panel_for_module(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
        module_id: &str,
    ) -> impl IntoElement {
        let p = theme::theme_palette(cx, is_dark);
        let g_snap = theme::glyph_snapshot(cx);
        let panel_radius = g_snap.map(|g| g.border_radius).unwrap_or(p.radius_md);

        let token_specs = list_style_tokens()
            .into_iter()
            .find(|(m, _)| m == module_id)
            .map(|(_, specs)| specs)
            .unwrap_or_default();

        let wrong_surface = token_specs.is_empty()
            || python_registry::extension_tokens_editable_under_appearance(module_id);

        let is_ext_enabled = extension_enabled(module_id);
        let module_id_for_btn = module_id.to_string();

        let title_row = {
            let title_text = self
                .page_ref(self.active_page_id.as_str())
                .map(|pg| pg.title().to_string())
                .unwrap_or_else(|| "Extension tokens".into());

            let mut row = div()
                .w_full()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .gap_4()
                .child(
                    div()
                        .text_2xl()
                        .font_weight(FontWeight::BOLD)
                        .text_color(p.content_title)
                        .child(title_text),
                );

            if is_ext_enabled {
                row = row.child(
                    div()
                        .px_3()
                        .py_1()
                        .rounded(px(p.radius_md))
                        .bg(p.danger)
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(p.on_accent)
                        .cursor_pointer()
                        .child("Disable Extension")
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, _, cx| {
                                let ctx = this.execution_context();
                                let _ = modules::execute_command(
                                    "python-host.extension-disable",
                                    &[module_id_for_btn.as_str()],
                                    &ctx,
                                );
                                this.reload_extension_state(cx);
                                this.active_page_id = "extensions.settings".into();
                                this.sync_settings_hub_expanded_from_active_page();
                                cx.notify();
                            }),
                        ),
                );
            }

            row
        };

        let header = div()
            .flex()
            .flex_col()
            .gap_1()
            .child(title_row)
            .child(
                div().text_sm().text_color(p.content_body).child(
                    self.page_ref(self.active_page_id.as_str())
                        .map(|pg| pg.description().to_string())
                        .unwrap_or_default(),
                ),
            );

        let body: openframe::Div = if wrong_surface {
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
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(p.content_title)
                        .child("Nothing to edit here"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(p.content_meta)
                        .child(
                            "Either this extension is not loaded, has no register_tokens, or its tokens are edited under Appearance with its render style.",
                        ),
                )
                .child(
                    div()
                        .px_3()
                        .py_2()
                        .rounded(px(8.))
                        .cursor_pointer()
                        .bg(p.accent)
                        .text_color(p.on_accent)
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("Open Extensions")
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                this.active_page_id = "extensions.settings".into();
                                this.sync_settings_hub_expanded_from_active_page();
                                cx.notify();
                            }),
                        ),
                )
                .when(python_registry::extension_tokens_editable_under_appearance(module_id), |d| {
                    d.child(
                        div()
                            .px_3()
                            .py_2()
                            .rounded(px(8.))
                            .cursor_pointer()
                            .bg(p.surface_elevated)
                            .border_1()
                            .border_color(p.border)
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(p.content_title)
                            .child("Open Appearance")
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _, cx| {
                                    this.active_page_id = "global.appearance".into();
                                    this.sync_settings_hub_expanded_from_active_page();
                                    cx.notify();
                                }),
                            ),
                    )
                })
        } else {
            self.extension_tokens_settings_card(
                window,
                cx,
                is_dark,
                module_id,
                &token_specs,
                "Token overrides".to_string(),
                "Saved under ~/Arcadia/Configuration/extension_tokens/. Press Enter to save."
                    .to_string(),
                None,
            )
        };

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_6()
            .child(header)
            .child(body)
    }
}
