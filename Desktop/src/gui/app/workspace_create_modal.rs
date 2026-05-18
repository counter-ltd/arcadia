use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use arcadia_core::config::workspace::{WorkspaceEntry, WorkspacesConfig};
use arcadia_core::config::ConfigFile;
use openframe::prelude::FluentBuilder as _;
use openframe::{
    div, px, rgb, text_input, AnyElement, Context, FontWeight, InteractiveElement, IntoElement,
    KeyDownEvent, MouseButton, ParentElement, PathPromptOptions, Styled, Window,
};

use super::ArcadiaRoot;
use crate::gui::theme;

impl ArcadiaRoot {
    pub fn workspace_create_modal(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> AnyElement {
        if self.workspace_create_draft.is_none() {
            return div().into_any_element();
        }

        let p = theme::theme_palette(cx, is_dark);
        let g = theme::glyph_snapshot(cx);
        let radius = g.map(|gg| gg.border_radius).unwrap_or(p.radius_md);

        let label_focused = self.workspace_create_label_focus.is_focused(window);
        let path_focused = self.workspace_create_path_focus.is_focused(window);

        let label_fh = self.workspace_create_label_focus.clone();
        let path_fh = self.workspace_create_path_focus.clone();

        let draft = self.workspace_create_draft.clone().unwrap();

        let modal_surface = g.map(|gg| gg.surface).unwrap_or(p.surface);
        let modal_border = g.map(|gg| gg.border).unwrap_or(p.border);
        let weak = cx.weak_entity();

        let label_field = text_input(
            "workspace-create-label",
            window,
            weak.clone(),
            &draft.label,
            "Workspace label…",
            &label_fh,
            p.content_title,
            p.ui_subtext,
            |this, new_text, cx| {
                if let Some(ref mut d) = this.workspace_create_draft { d.label = new_text; }
                cx.notify();
            },
        )
        .w_full()
        .px_3()
        .py_2()
        .rounded(px(radius.min(8.0)))
        .bg(p.surface_elevated)
        .border_1()
        .border_color(if label_focused { p.accent } else { p.border })
        .text_sm()
        .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
            if event.keystroke.key == "escape" {
                this.workspace_create_draft = None;
                cx.notify();
            }
        }))
        .into_any_element();

        let path_field = text_input(
            "workspace-create-path",
            window,
            weak.clone(),
            &draft.path,
            "/path/to/project…",
            &path_fh,
            p.content_title,
            p.ui_subtext,
            |this, new_text, cx| {
                if let Some(ref mut d) = this.workspace_create_draft { d.path = new_text; }
                cx.notify();
            },
        )
        .w_full()
        .px_3()
        .py_2()
        .rounded(px(radius.min(8.0)))
        .bg(p.surface_elevated)
        .border_1()
        .border_color(if path_focused { p.accent } else { p.border })
        .text_sm()
        .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
            if event.keystroke.key == "escape" {
                this.workspace_create_draft = None;
                cx.notify();
            }
        }))
        .into_any_element();

        let error_el = draft.error.as_ref().map(|e| {
            div()
                .text_xs()
                .text_color(p.danger)
                .child(e.clone())
                .into_any_element()
        });

        div()
            .absolute()
            .top_0()
            .left_0()
            .right_0()
            .bottom_0()
            .child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .bottom_0()
                    .bg(rgb(0x000000))
                    .opacity(0.3)
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                        this.workspace_create_draft = None;
                        cx.notify();
                    })),
            )
            .child(
                div()
                    .size_full()
                    .flex()
                    .justify_center()
                    .items_center()
                    .child(
                        div()
                            .w(px(480.0))
                            .p_5()
                            .rounded(px(radius.min(16.0)))
                            .bg(modal_surface)
                            .border_1()
                            .border_color(modal_border)
                            .flex()
                            .flex_col()
                            .gap_4()
                            .on_mouse_down(MouseButton::Left, cx.listener(|_, _, _, cx| {
                                cx.stop_propagation();
                            }))
                            // Title
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(p.content_title)
                                    .child("Add Workspace"),
                            )
                            // Label row
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(p.ui_subtext)
                                            .child("LABEL"),
                                    )
                                    .child(label_field),
                            )
                            // Path row
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(p.ui_subtext)
                                            .child("PATH"),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .child(div().flex_1().child(path_field))
                                            .child(
                                                div()
                                                    .px_3()
                                                    .py_2()
                                                    .rounded(px(radius.min(8.0)))
                                                    .bg(p.surface_elevated)
                                                    .border_1()
                                                    .border_color(p.border)
                                                    .text_sm()
                                                    .font_weight(FontWeight::SEMIBOLD)
                                                    .text_color(p.content_title)
                                                    .cursor_pointer()
                                                    .child("Browse…")
                                                    .on_mouse_down(MouseButton::Left, cx.listener(|_this, _, window, cx| {
                                                        let receiver = cx.prompt_for_paths(PathPromptOptions {
                                                            files: false,
                                                            directories: true,
                                                            multiple: false,
                                                            prompt: None,
                                                            allowed_extensions: Vec::new(),
                                                        });
                                                        cx.spawn_in(window, move |this: openframe::WeakEntity<ArcadiaRoot>, cx: &mut openframe::AsyncWindowContext| {
                                                            let mut cx = cx.clone();
                                                            async move {
                                                                if let Ok(Ok(Some(paths))) = receiver.await {
                                                                    if let Some(path) = paths.into_iter().next() {
                                                                        cx.update(|_, app| {
                                                                            this.update(app, |this, cx| {
                                                                                if let Some(ref mut d) = this.workspace_create_draft {
                                                                                    d.path = path.to_string_lossy().to_string();
                                                                                    d.error = None;
                                                                                }
                                                                                cx.notify();
                                                                            }).ok();
                                                                        }).ok();
                                                                    }
                                                                }
                                                            }
                                                        }).detach();
                                                    })),
                                            ),
                                    ),
                            )
                            // Error
                            .when_some(error_el, |d, e| d.child(e))
                            // Footer
                            .child(
                                div()
                                    .flex()
                                    .justify_end()
                                    .gap_2()
                                    .child(
                                        div()
                                            .px_3()
                                            .py_1()
                                            .rounded(px(radius.min(8.0)))
                                            .bg(p.surface_elevated)
                                            .border_1()
                                            .border_color(p.border)
                                            .text_xs()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(p.ui_subtext)
                                            .cursor_pointer()
                                            .child("Cancel")
                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                this.workspace_create_draft = None;
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        div()
                                            .px_3()
                                            .py_1()
                                            .rounded(px(radius.min(8.0)))
                                            .bg(p.accent)
                                            .text_xs()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(p.on_accent)
                                            .cursor_pointer()
                                            .child("Add")
                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                this.workspace_create_save(cx);
                                            })),
                                    ),
                            ),
                    ),
            )
            .into_any_element()
    }

    pub fn workspace_create_save(&mut self, cx: &mut Context<Self>) {
        let Some(draft) = self.workspace_create_draft.clone() else {
            return;
        };

        let label = draft.label.trim().to_string();
        if label.is_empty() {
            if let Some(ref mut d) = self.workspace_create_draft {
                d.error = Some("Label is required.".to_string());
            }
            cx.notify();
            return;
        }

        let path = draft.path.trim().to_string();
        if path.is_empty() {
            if let Some(ref mut d) = self.workspace_create_draft {
                d.error = Some("Path is required.".to_string());
            }
            cx.notify();
            return;
        }

        if !Path::new(&path).exists() {
            if let Some(ref mut d) = self.workspace_create_draft {
                d.error = Some(format!("Path does not exist: {path}"));
            }
            cx.notify();
            return;
        }

        let Ok(mut cfg) = WorkspacesConfig::load_or_create() else {
            if let Some(ref mut d) = self.workspace_create_draft {
                d.error = Some("Could not load workspace.toml".to_string());
            }
            cx.notify();
            return;
        };

        if cfg.workspaces.iter().any(|w| w.path == path) {
            if let Some(ref mut d) = self.workspace_create_draft {
                d.error = Some(format!("Workspace already exists for: {path}"));
            }
            cx.notify();
            return;
        }

        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let id = format!("ws_{ms}");

        cfg.workspaces.push(WorkspaceEntry {
            id,
            label,
            path,
            granted_permissions: vec!["workspace.read".to_string()],
        });

        if let Err(e) = cfg.save() {
            if let Some(ref mut d) = self.workspace_create_draft {
                d.error = Some(format!("Save failed: {e}"));
            }
            cx.notify();
            return;
        }

        self.workspace_create_draft = None;
        cx.notify();
    }
}
