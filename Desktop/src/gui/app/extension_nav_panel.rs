use arcadia_core::modules::{self, python_registry};
use arcadia_core::modules::python_registry::NavActionStyle;
use openframe::{
    div, px, Context, FontWeight, InteractiveElement, IntoElement, MouseButton, ParentElement,
    Styled, Window,
};

use crate::gui::app::ArcadiaRoot;
use crate::gui::theme;

impl ArcadiaRoot {
    pub(crate) fn extension_nav_panel(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
        is_dark: bool,
        ext_id: &str,
    ) -> impl IntoElement {
        let p = theme::theme_palette(cx, is_dark);
        let ext_id = ext_id.to_string();

        let decl = python_registry::nav_page_for(&ext_id);

        if decl.is_none() {
            return div()
                .w_full()
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(p.content_title)
                        .child("Extension not loaded"),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(p.content_meta)
                        .child("Enable this extension in Extensions settings and reload."),
                );
        }

        let decl = decl.unwrap();

        // Query status if declared.
        let status: Option<(String, bool, Option<Vec<String>>)> = decl.status_command.as_ref().and_then(|cmd| {
            let ctx = self.execution_context();
            let raw = modules::execute_command(cmd, &[], &ctx).ok()??;
            let obj: serde_json::Value = serde_json::from_str(&raw).ok()?;
            let text = obj.get("text")?.as_str()?.to_string();
            let active = obj.get("active").and_then(|v| v.as_bool()).unwrap_or(false);
            let active_args = obj.get("active_args")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|x| x.as_str().map(str::to_string)).collect());
            Some((text, active, active_args))
        });

        let mut container = div().w_full().flex().flex_col().gap_4();

        // Header: title + description.
        container = container.child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .text_base()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(p.content_title)
                        .child(decl.title.clone()),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(p.content_meta)
                        .child(decl.description.clone()),
                ),
        );

        // Status badge.
        if let Some((text, active, _)) = status.as_ref().map(|(t, a, aa)| (t.clone(), *a, aa.clone())) {
            let badge_bg = if active { p.badge_info_bg } else { p.badge_muted_bg };
            let badge_text = if active { p.badge_info_fg } else { p.badge_muted_fg };
            container = container.child(
                div()
                    .px_3()
                    .py_1()
                    .rounded(px(p.radius_md))
                    .bg(badge_bg)
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(badge_text)
                    .max_w(px(400.))
                    .child(text),
            );
        }

        // Action buttons.
        if !decl.actions.is_empty() {
            let active_args: Option<Vec<String>> = status.as_ref().and_then(|(_, _, aa)| aa.clone());
            let mut row = div().flex().flex_row().flex_wrap().gap_2();
            for action in &decl.actions {
                let cmd = action.command.clone();
                let args: Vec<String> = action.args.clone();
                let style = action.style.clone();
                let is_active = active_args.as_ref().map(|aa| aa == &action.args).unwrap_or(false);
                let (btn_bg, btn_text) = if is_active {
                    (p.accent, p.on_accent)
                } else {
                    match style {
                        NavActionStyle::Primary => (p.accent, p.on_accent),
                        NavActionStyle::Destructive => (p.danger, p.on_accent),
                        NavActionStyle::Secondary => (p.row_bg, p.content_title),
                    }
                };
                let label = action.label.clone();
                row = row.child(
                    div()
                        .px_3()
                        .py(px(6.))
                        .rounded(px(p.radius_md))
                        .bg(btn_bg)
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(btn_text)
                        .cursor_pointer()
                        .child(label)
                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                            let ctx = this.execution_context();
                            let arg_strs: Vec<&str> = args.iter().map(String::as_str).collect();
                            let _ = modules::execute_command(&cmd, &arg_strs, &ctx);
                            cx.notify();
                        })),
                );
            }
            container = container.child(row);
        }

        container
    }
}
