use std::time::{SystemTime, UNIX_EPOCH};

use arcadia_core::config::llama_cpp::{LlamaCppConfig, LlamaCppModel, LlamaCppModelKind};
use arcadia_core::config::ConfigFile;
use openframe::prelude::FluentBuilder as _;
use openframe::{
    AnyElement, Context, FontWeight, InteractiveElement, IntoElement, KeyDownEvent, MouseButton,
    ParentElement, PathPromptOptions, Styled, Window, div, px, rgb,
};

use super::ArcadiaRoot;
use crate::gui::app::text_input_caret::text_with_trailing_caret;
use crate::gui::theme;
use crate::gui::theme::palette::ThemePalette;

fn text_field(
    value: &str,
    placeholder: &'static str,
    focused: bool,
    blink: bool,
    focus_handle: openframe::FocusHandle,
    p: ThemePalette,
    radius: f32,
    on_key_down: impl Fn(
            &mut ArcadiaRoot,
            &KeyDownEvent,
            &mut openframe::Window,
            &mut Context<ArcadiaRoot>,
        ) + 'static,
    cx: &mut Context<ArcadiaRoot>,
) -> AnyElement {
    let fh = focus_handle.clone();
    let content: String = if value.is_empty() && !focused {
        placeholder.to_string()
    } else {
        text_with_trailing_caret(value, focused, blink)
    };
    let text_color = if value.is_empty() && !focused {
        p.ui_subtext
    } else {
        p.content_title
    };
    div()
        .w_full()
        .px_3()
        .py_2()
        .rounded(px(radius.min(8.0)))
        .bg(p.surface_elevated)
        .border_1()
        .border_color(if focused { p.accent } else { p.border })
        .text_sm()
        .text_color(text_color)
        .track_focus(&focus_handle)
        .on_mouse_down(MouseButton::Left, cx.listener(move |_, _, window, _| {
            fh.focus(window);
        }))
        .on_key_down(cx.listener(on_key_down))
        .child(content)
        .into_any_element()
}

impl ArcadiaRoot {
    pub fn llama_cpp_create_model_modal(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> AnyElement {
        if self.llama_cpp_create_draft.is_none() {
            return div().into_any_element();
        }

        let p = theme::theme_palette(cx, is_dark);
        let g = theme::glyph_snapshot(cx);
        let radius = g.map(|gg| gg.border_radius).unwrap_or(p.radius_md);

        let name_focused = self.llama_cpp_create_name_focus.is_focused(window);
        let path_focused = self.llama_cpp_create_path_focus.is_focused(window);
        let mmproj_focused = self.llama_cpp_create_mmproj_focus.is_focused(window);
        let blink = self.text_caret_blink_visible;

        let name_fh = self.llama_cpp_create_name_focus.clone();
        let path_fh = self.llama_cpp_create_path_focus.clone();
        let mmproj_fh = self.llama_cpp_create_mmproj_focus.clone();

        let draft = self.llama_cpp_create_draft.clone().unwrap();
        let is_vision = draft.model_kind == LlamaCppModelKind::Vision;

        let modal_surface = g.map(|gg| gg.surface).unwrap_or(p.surface);
        let modal_border = g.map(|gg| gg.border).unwrap_or(p.border);

        let name_field = text_field(
            &draft.name,
            "Model name…",
            name_focused,
            blink,
            name_fh,
            p,
            radius,
            |this, event, _, cx| {
                let key = event.keystroke.key.as_str();
                let mods = event.keystroke.modifiers;
                if key == "escape" {
                    this.llama_cpp_create_draft = None;
                    cx.notify();
                    return;
                }
                if let Some(ref mut d) = this.llama_cpp_create_draft {
                    if key == "backspace" {
                        d.name.pop();
                        cx.notify();
                    } else if !mods.control && !mods.alt && !mods.platform && !mods.function {
                        if let Some(kc) = &event.keystroke.key_char {
                            d.name.push_str(kc);
                            cx.notify();
                        }
                    }
                }
            },
            cx,
        );

        let path_field = text_field(
            &draft.path,
            "/path/to/model.gguf…",
            path_focused,
            blink,
            path_fh,
            p,
            radius,
            |this, event, _, cx| {
                let key = event.keystroke.key.as_str();
                let mods = event.keystroke.modifiers;
                if key == "escape" {
                    this.llama_cpp_create_draft = None;
                    cx.notify();
                    return;
                }
                if let Some(ref mut d) = this.llama_cpp_create_draft {
                    if key == "backspace" {
                        d.path.pop();
                        cx.notify();
                    } else if !mods.control && !mods.alt && !mods.platform && !mods.function {
                        if let Some(kc) = &event.keystroke.key_char {
                            d.path.push_str(kc);
                            cx.notify();
                        }
                    }
                }
            },
            cx,
        );

        let mmproj_field = text_field(
            &draft.mmproj_path,
            "/path/to/mmproj.gguf…",
            mmproj_focused,
            blink,
            mmproj_fh,
            p,
            radius,
            |this, event, _, cx| {
                let key = event.keystroke.key.as_str();
                let mods = event.keystroke.modifiers;
                if key == "escape" {
                    this.llama_cpp_create_draft = None;
                    cx.notify();
                    return;
                }
                if let Some(ref mut d) = this.llama_cpp_create_draft {
                    if key == "backspace" {
                        d.mmproj_path.pop();
                        cx.notify();
                    } else if !mods.control && !mods.alt && !mods.platform && !mods.function {
                        if let Some(kc) = &event.keystroke.key_char {
                            d.mmproj_path.push_str(kc);
                            cx.notify();
                        }
                    }
                }
            },
            cx,
        );

        let error_el = draft.error.as_ref().map(|e| {
            div()
                .text_xs()
                .text_color(p.danger)
                .child(e.clone())
                .into_any_element()
        });

        // Model type selector row
        let type_row = {
            let current_type = draft.model_kind.clone();
            let mut row = div().flex().gap_2();
            for mt in LlamaCppModelKind::all() {
                let mt_clone = mt.clone();
                let is_selected = *mt == current_type;
                let bg = if is_selected { p.accent } else { p.surface_elevated };
                let fg = if is_selected { p.on_accent } else { p.content_title };
                let border = if is_selected { p.accent } else { p.border };
                row = row.child(
                    div()
                        .px_3()
                        .py_1()
                        .rounded(px(radius.min(8.0)))
                        .border_1()
                        .border_color(border)
                        .bg(bg)
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(fg)
                        .cursor_pointer()
                        .child(mt.label())
                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                            if let Some(ref mut d) = this.llama_cpp_create_draft {
                                d.model_kind = mt_clone.clone();
                            }
                            cx.notify();
                        })),
                );
            }
            row
        };

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
                        this.llama_cpp_create_draft = None;
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
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(p.content_title)
                                    .child("Create Model"),
                            )
                            // Name
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
                                            .child("NAME"),
                                    )
                                    .child(name_field),
                            )
                            // Path
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
                                            .child("MODEL FILE"),
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
                                                            files: true,
                                                            directories: false,
                                                            multiple: false,
                                                            prompt: None,
                                                        });
                                                        cx.spawn_in(window, move |this: openframe::WeakEntity<ArcadiaRoot>, cx: &mut openframe::AsyncWindowContext| {
                                                            let mut cx = cx.clone();
                                                            async move {
                                                                if let Ok(Ok(Some(paths))) = receiver.await {
                                                                    if let Some(path) = paths.into_iter().next() {
                                                                        cx.update(|_, app| {
                                                                            this.update(app, |this, cx| {
                                                                                if let Some(ref mut d) = this.llama_cpp_create_draft {
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
                            // Type
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
                                            .child("TYPE"),
                                    )
                                    .child(type_row),
                            )
                            // mmproj path — Vision only
                            .when(is_vision, |d| d.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(p.ui_subtext)
                                            .child("MMPROJ FILE"),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .child(div().flex_1().child(mmproj_field))
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
                                                            files: true,
                                                            directories: false,
                                                            multiple: false,
                                                            prompt: None,
                                                        });
                                                        cx.spawn_in(window, move |this: openframe::WeakEntity<ArcadiaRoot>, cx: &mut openframe::AsyncWindowContext| {
                                                            let mut cx = cx.clone();
                                                            async move {
                                                                if let Ok(Ok(Some(paths))) = receiver.await {
                                                                    if let Some(path) = paths.into_iter().next() {
                                                                        cx.update(|_, app| {
                                                                            this.update(app, |this, cx| {
                                                                                if let Some(ref mut d) = this.llama_cpp_create_draft {
                                                                                    d.mmproj_path = path.to_string_lossy().to_string();
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
                            ))
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
                                                this.llama_cpp_create_draft = None;
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
                                            .child("Create")
                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                this.llama_cpp_create_model_save(cx);
                                            })),
                                    ),
                            ),
                    ),
            )
            .into_any_element()
    }

    pub fn llama_cpp_create_model_save(&mut self, cx: &mut Context<Self>) {
        let Some(draft) = self.llama_cpp_create_draft.clone() else {
            return;
        };

        let name = draft.name.trim().to_string();
        if name.is_empty() {
            if let Some(ref mut d) = self.llama_cpp_create_draft {
                d.error = Some("Name is required.".to_string());
            }
            cx.notify();
            return;
        }

        let path = draft.path.trim().to_string();
        if path.is_empty() {
            if let Some(ref mut d) = self.llama_cpp_create_draft {
                d.error = Some("Model file path is required.".to_string());
            }
            cx.notify();
            return;
        }
        if !std::path::Path::new(&path).exists() {
            if let Some(ref mut d) = self.llama_cpp_create_draft {
                d.error = Some("Model file not found at that path.".to_string());
            }
            cx.notify();
            return;
        }

        let mmproj_path = if draft.model_kind == arcadia_core::config::llama_cpp::LlamaCppModelKind::Vision {
            let p = draft.mmproj_path.trim().to_string();
            if p.is_empty() {
                if let Some(ref mut d) = self.llama_cpp_create_draft {
                    d.error = Some("mmproj file path is required for Vision models.".to_string());
                }
                cx.notify();
                return;
            }
            if !std::path::Path::new(&p).exists() {
                if let Some(ref mut d) = self.llama_cpp_create_draft {
                    d.error = Some("mmproj file not found at that path.".to_string());
                }
                cx.notify();
                return;
            }
            Some(p)
        } else {
            None
        };

        let Ok(mut cfg) = LlamaCppConfig::load_or_create() else {
            if let Some(ref mut d) = self.llama_cpp_create_draft {
                d.error = Some("Could not load llama-cpp.toml".to_string());
            }
            cx.notify();
            return;
        };

        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let id = format!("m_{ms}");

        let model = LlamaCppModel {
            id: id.clone(),
            name,
            model_kind: draft.model_kind,
            path,
            mmproj_path,
        };

        cfg.models.push(model.clone());

        if let Err(e) = cfg.save() {
            if let Some(ref mut d) = self.llama_cpp_create_draft {
                d.error = Some(format!("Save failed: {e}"));
            }
            cx.notify();
            return;
        }

        self.llama_cpp_models.push(model);
        self.active_llama_cpp_model_id = Some(id);
        self.active_ai_provider_module =
            arcadia_core::config::modules::AI_LLAMA_CPP_MODULE_NAME.to_string();
        self.active_page_id = "ai.models".to_string();
        self.llama_cpp_create_draft = None;
        cx.notify();
    }
}
