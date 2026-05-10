use arcadia_core::config::modules::{ModuleManifest, ModulesConfig};
use arcadia_core::modules;
use arcadia_core::config::ConfigFile;
use openframe::{div, rgb};
use openframe::{Context, InteractiveElement, IntoElement, ParentElement, Styled};

use crate::gui::app::ArcadiaRoot;
use crate::gui::theme;

impl ArcadiaRoot {
    pub fn module_row_item(
        cx: &mut Context<Self>,
        module_name: String,
        enabled: bool,
        manifest: Option<&'static ModuleManifest>,
        is_dark: bool,
    ) -> impl IntoElement {
        let version = manifest.map(|m| m.version).unwrap_or("unknown");
        let description = manifest
            .map(|m| m.description)
            .unwrap_or("No manifest description.");
        let state = if enabled { "Enabled" } else { "Disabled" };
        div()
            .w_full()
            .px_4()
            .py_3()
            .rounded_lg()
            .bg(theme::module_row_bg(is_dark))
            .border_1()
            .border_color(theme::module_row_stroke(is_dark))
            .flex()
            .justify_between()
            .items_center()
            .gap_4()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_base()
                            .font_weight(openframe::FontWeight::BOLD)
                            .text_color(theme::module_title_text(is_dark))
                            .child(module_name.clone()),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::module_meta_text(is_dark))
                                    .child(format!("v{version}")),
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py_0p5()
                                    .rounded_full()
                                    .text_xs()
                                    .font_weight(openframe::FontWeight::SEMIBOLD)
                                    .bg(if enabled {
                                        theme::module_state_enabled_bg(is_dark)
                                    } else {
                                        theme::module_state_disabled_bg(is_dark)
                                    })
                                    .text_color(if enabled {
                                        theme::module_state_enabled_text(is_dark)
                                    } else {
                                        theme::module_state_disabled_text(is_dark)
                                    })
                                    .child(state),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::module_description_text(is_dark))
                            .child(description),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_2()
                    .py_1()
                    .rounded_full()
                    .cursor_pointer()
                    .bg(if enabled {
                        theme::module_state_enabled_bg(is_dark)
                    } else {
                        theme::module_state_disabled_bg(is_dark)
                    })
                    .child(
                        div()
                            .text_xs()
                            .font_weight(openframe::FontWeight::SEMIBOLD)
                            .text_color(if enabled {
                                theme::module_state_enabled_text(is_dark)
                            } else {
                                theme::module_state_disabled_text(is_dark)
                            })
                            .child(if enabled { "ON" } else { "OFF" }),
                    )
                    .child(if enabled {
                        div()
                            .w_10()
                            .h_6()
                            .px_0p5()
                            .rounded_full()
                            .border_1()
                            .border_color(theme::module_row_stroke(is_dark))
                            .bg(theme::module_button_enable_bg(is_dark))
                            .flex()
                            .items_center()
                            .justify_end()
                            .child(
                                div()
                                    .w_4()
                                    .h_4()
                                    .rounded_full()
                                    .bg(theme::module_button_enable_text(is_dark)),
                            )
                    } else {
                        div()
                            .w_10()
                            .h_6()
                            .px_0p5()
                            .rounded_full()
                            .border_1()
                            .border_color(theme::module_row_stroke(is_dark))
                            .bg(theme::module_panel_stroke(is_dark))
                            .flex()
                            .items_center()
                            .justify_start()
                            .child(div().w_4().h_4().rounded_full().bg(if is_dark {
                                rgb(0xd1d5db)
                            } else {
                                rgb(0xf8fafc)
                            }))
                    })
                    .on_mouse_down(
                        openframe::MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            if this.remote_route.is_some() {
                                let enabled_next = !enabled;
                                let ctx = this.execution_context();
                                let name = module_name.clone();
                                let payload =
                                    arcadia_core::modules::surface::patch_json_modules_set(
                                        &name,
                                        enabled_next,
                                        Some(this.surface_client_id.as_str()),
                                    );
                                match modules::execute_command("surface.patch", &[payload.as_str()], &ctx)
                                {
                                    Err(err) => eprintln!("{err}"),
                                    Ok(Some(msg)) => eprintln!("{msg}"),
                                    Ok(None) => {}
                                }
                                this.pending_module_enable = None;
                                this.reload_modules();
                                cx.notify();
                                return;
                            }
                            if enabled {
                                if let Ok(mut cfg) = ModulesConfig::load_or_create() {
                                    let _ = cfg.set_module_state(&module_name, false);
                                    let _ = cfg.save();
                                }
                                this.pending_module_enable = None;
                                this.reload_modules();
                                cx.notify();
                                return;
                            }
                            match ModulesConfig::load_or_create() {
                                Ok(cfg) => match cfg.missing_requirements_for(&module_name) {
                                    Ok(missing) if !missing.is_empty() => {
                                        this.pending_module_enable =
                                            Some((module_name.clone(), missing));
                                    }
                                    Ok(_) => {
                                        if let Ok(mut cfg) = ModulesConfig::load_or_create() {
                                            let _ = cfg.enable_with_requirements(&module_name);
                                            let _ = cfg.save();
                                        }
                                        this.pending_module_enable = None;
                                        this.reload_modules();
                                    }
                                    Err(err) => eprintln!("{err}"),
                                },
                                Err(err) => eprintln!("{err}"),
                            }
                            cx.notify();
                        }),
                    ),
            )
    }
}
