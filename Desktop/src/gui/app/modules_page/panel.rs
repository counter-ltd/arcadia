use arcadia_core::config::modules::{
    supports_runtime_platform, supports_runtime_platform_owned, ModulesConfig,
    PYTHON_HOST_MODULE_NAME, WASM_HOST_MODULE_NAME,
};
use arcadia_core::modules;
use openframe::prelude::FluentBuilder as _;
use openframe::{div, px, AnyElement, Window};
use openframe::{Context, FontWeight, InteractiveElement, IntoElement, MouseButton, ParentElement, Styled};

use crate::gui::app::list_panel_search::{list_panel_row_matches, ListPanelSearchKind};
use crate::gui::app::ArcadiaRoot;
use crate::gui::theme::{self, GLYPH_PANEL_CONTENT_MAX_W_PX};

impl ArcadiaRoot {
    pub fn modules_panel(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> AnyElement {
        if !matches!(
            self.active_page_id.as_str(),
            "global.modules" | "extensions.settings" | "wasm-modules.settings"
        ) {
            return div().into_any_element();
        }

        let glyph_cfg = theme::glyph_snapshot(cx);
        let p = theme::theme_palette(cx, is_dark);
        let radius = glyph_cfg.map(|g| g.border_radius).unwrap_or(p.radius_md);

        let q = self.modules_search_query.trim().to_ascii_lowercase();

        let python_host_enabled = self
            .module_rows
            .iter()
            .any(|(n, en)| n == PYTHON_HOST_MODULE_NAME && *en);
        let wasm_host_enabled = self
            .module_rows
            .iter()
            .any(|(n, en)| n == WASM_HOST_MODULE_NAME && *en);

        let search_bar =
            self.list_panel_search_bar(window, cx, is_dark, ListPanelSearchKind::Modules);

        // ── Section 1: Built-in ──────────────────────────────────────────────
        let builtin_rows_src: Vec<_> = self
            .module_rows
            .iter()
            .filter(|(module_name, _)| {
                let manifest = ModulesConfig::manifest_for(module_name);
                let version = manifest.map(|m| m.version).unwrap_or("unknown");
                let description = manifest.map(|m| m.description).unwrap_or("");
                list_panel_row_matches(&q, module_name, &[version, description])
            })
            .collect();

        let builtin_rows: Vec<_> = builtin_rows_src
            .into_iter()
            .map(|(module_name, enabled)| {
                let manifest = ModulesConfig::manifest_for(module_name);
                let runtime_supported = manifest
                    .map(|m| supports_runtime_platform(m.supported_platforms))
                    .unwrap_or(true);
                Self::module_row_item(
                    cx,
                    module_name.clone(),
                    *enabled,
                    manifest,
                    is_dark,
                    runtime_supported,
                    &["built-in".to_string()],
                )
            })
            .collect();

        let builtin_empty_filtered = builtin_rows.is_empty()
            && !self.module_rows.is_empty()
            && !self.modules_search_query.trim().is_empty();

        let builtin_header = div()
            .w_full()
            .flex()
            .items_center()
            .gap_2()
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(p.content_meta)
                    .child("BUILT-IN"),
            );

        let builtin_body: AnyElement = if builtin_empty_filtered {
            div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .gap_2()
                .py_6()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme::module_title_text(is_dark))
                        .child("No matching modules"),
                )
                .into_any_element()
        } else {
            div()
                .flex()
                .flex_col()
                .gap_3()
                .children(builtin_rows)
                .into_any_element()
        };

        // ── Section 2: Extensions ────────────────────────────────────────────
        let ext_header = div()
            .w_full()
            .flex()
            .items_center()
            .justify_between()
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(p.content_meta)
                    .child("EXTENSIONS"),
            )
            .when(python_host_enabled, |d| {
                d.child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded(px(radius.min(8.0)))
                        .cursor_pointer()
                        .text_xs()
                        .bg(p.surface_elevated)
                        .text_color(p.ui_subtext)
                        .hover(move |s| s.text_color(p.content_title))
                        .child("Reload")
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                let ctx = this.execution_context();
                                let _ = modules::execute_command("python-host.reload", &[], &ctx);
                                this.reload_extension_state(cx);
                                cx.notify();
                            }),
                        ),
                )
            });

        let ext_error_banner: Option<AnyElement> =
            self.python_extension_action_error.clone().map(|msg| {
                div()
                    .w_full()
                    .px_3()
                    .py_2()
                    .rounded(px(radius.min(8.0)))
                    .bg(p.row_bg)
                    .border_1()
                    .border_color(p.accent)
                    .child(
                        div()
                            .text_xs()
                            .text_color(p.content_body)
                            .child(format!("Last action failed: {msg}")),
                    )
                    .into_any_element()
            });

        let ext_body: AnyElement = if !python_host_enabled {
            div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .gap_2()
                .py_6()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(p.content_meta)
                        .child("Enable the python-host module to use extensions."),
                )
                .into_any_element()
        } else {
            let all_ext_rows = self.python_extension_rows.clone();
            let ext_none_loaded = all_ext_rows.is_empty();
            let ext_filtered: Vec<_> = all_ext_rows
                .into_iter()
                .filter(|(name, version, description, _, _, _, tags)| {
                    let mut extras = vec![version.as_str(), description.as_str()];
                    extras.extend(tags.iter().map(String::as_str));
                    list_panel_row_matches(&q, name, &extras)
                })
                .collect();

            if ext_none_loaded {
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_2()
                    .py_6()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(p.content_title)
                            .child("No extensions loaded"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(p.content_meta)
                            .child("Drop a folder with main.py into ~/Arcadia/Extensions/ (static files in that folder's Assets/). Or a single .py file. Then reload."),
                    )
                    .into_any_element()
            } else if ext_filtered.is_empty() && !self.extensions_search_query.trim().is_empty() {
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_2()
                    .py_6()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(p.content_title)
                            .child("No matching extensions"),
                    )
                    .into_any_element()
            } else {
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .children(ext_filtered.into_iter().map(
                        |(name, version, description, enabled, _, platforms, tags)| {
                            let runtime_supported =
                                supports_runtime_platform_owned(platforms.as_slice());
                            Self::python_extension_row(
                                cx,
                                name,
                                version,
                                description,
                                enabled,
                                is_dark,
                                radius,
                                runtime_supported,
                                tags,
                            )
                        },
                    ))
                    .into_any_element()
            }
        };

        // ── Section 3: WASM Modules ──────────────────────────────────────────
        let wasm_header = div()
            .w_full()
            .flex()
            .items_center()
            .justify_between()
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(p.content_meta)
                    .child("WASM MODULES"),
            )
            .when(wasm_host_enabled, |d| {
                d.child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded(px(radius.min(8.0)))
                        .cursor_pointer()
                        .text_xs()
                        .bg(p.surface_elevated)
                        .text_color(p.ui_subtext)
                        .hover(move |s| s.text_color(p.content_title))
                        .child("Reload")
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                let ctx = this.execution_context();
                                let _ = modules::execute_command("wasm-host.reload", &[], &ctx);
                                this.reload_wasm_state(cx);
                                cx.notify();
                            }),
                        ),
                )
            });

        let wasm_error_banner: Option<AnyElement> =
            self.wasm_action_error.clone().map(|msg| {
                div()
                    .w_full()
                    .px_3()
                    .py_2()
                    .rounded(px(radius.min(8.0)))
                    .bg(p.row_bg)
                    .border_1()
                    .border_color(p.accent)
                    .child(
                        div()
                            .text_xs()
                            .text_color(p.content_body)
                            .child(format!("Last action failed: {msg}")),
                    )
                    .into_any_element()
            });

        let wasm_body: AnyElement = if !wasm_host_enabled {
            div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .gap_2()
                .py_6()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(p.content_meta)
                        .child("Enable the wasm-host module to use WASM modules."),
                )
                .into_any_element()
        } else {
            let all_wasm_rows = self.wasm_module_rows.clone();
            let wasm_none_found = all_wasm_rows.is_empty();
            let wasm_filtered: Vec<_> = all_wasm_rows
                .into_iter()
                .filter(|(name, version, description, _, _, _, tags, _)| {
                    let mut extras = vec![version.as_str(), description.as_str()];
                    extras.extend(tags.iter().map(String::as_str));
                    list_panel_row_matches(&q, name, &extras)
                })
                .collect();

            if wasm_none_found {
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_2()
                    .py_6()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(p.content_title)
                            .child("No WASM modules found"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(p.content_meta)
                            .child("Drop a .wasm file (or a folder with module.wasm + Assets/) into ~/Arcadia/Modules/, then reload."),
                    )
                    .into_any_element()
            } else if wasm_filtered.is_empty() && !self.wasm_search_query.trim().is_empty() {
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_2()
                    .py_6()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(p.content_title)
                            .child("No matching WASM modules"),
                    )
                    .into_any_element()
            } else {
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .children(wasm_filtered.into_iter().map(
                        |(name, version, description, enabled, perms, platforms, tags, _loaded)| {
                            let runtime_supported =
                                supports_runtime_platform_owned(platforms.as_slice());
                            Self::wasm_module_row(
                                cx,
                                name,
                                version,
                                description,
                                enabled,
                                is_dark,
                                radius,
                                runtime_supported,
                                tags,
                                perms,
                            )
                        },
                    ))
                    .into_any_element()
            }
        };

        // ── Assemble body ────────────────────────────────────────────────────
        let mut full_body = div().flex().flex_col().gap_4();
        // Built-in section
        full_body = full_body
            .child(builtin_header)
            .child(builtin_body);
        // Extensions section
        full_body = full_body.child(ext_header);
        if let Some(banner) = ext_error_banner {
            full_body = full_body.child(banner);
        }
        full_body = full_body.child(ext_body);
        // WASM section
        full_body = full_body.child(wasm_header);
        if let Some(banner) = wasm_error_banner {
            full_body = full_body.child(banner);
        }
        full_body = full_body.child(wasm_body);

        if let Some(ref g) = glyph_cfg {
            let r = g.border_radius.min(12.0);
            div()
                .w_full()
                .flex()
                .justify_center()
                .child(
                    div()
                        .w_full()
                        .max_w(px(GLYPH_PANEL_CONTENT_MAX_W_PX))
                        .p_4()
                        .rounded(px(r))
                        .bg(g.surface)
                        .border_1()
                        .border_color(g.border)
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(search_bar)
                        .child(full_body),
                )
                .into_any_element()
        } else {
            div()
                .w_full()
                .p_4()
                .rounded(px(8.))
                .bg(theme::module_panel_bg(is_dark))
                .border_1()
                .border_color(theme::module_panel_stroke(is_dark))
                .flex()
                .flex_col()
                .gap_3()
                .child(search_bar)
                .child(full_body)
                .into_any_element()
        }
    }
}
