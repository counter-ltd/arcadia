use arcadia_core::config::modules::supports_runtime_platform_owned;
use arcadia_core::modules;
use arcadia_core::modules::python_registry;
use openframe::prelude::FluentBuilder as _;
use openframe::{
    div, px, AnyElement, InteractiveElement, IntoElement, ParentElement, Styled, Window,
};
use openframe::{Context, FontWeight, MouseButton};

use crate::gui::app::list_panel_search::{list_panel_row_matches, ListPanelSearchKind};
use crate::gui::app::ArcadiaRoot;
use crate::gui::theme::{self, render_icon, GLYPH_PANEL_CONTENT_MAX_W_PX};

impl ArcadiaRoot {
    pub fn python_settings_panel(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> AnyElement {
        let p = theme::theme_palette(cx, is_dark);
        let g_snap = theme::glyph_snapshot(cx);
        let panel_radius = g_snap.map(|g| g.border_radius).unwrap_or(p.radius_md);

        let all_rows = self.python_extension_rows.clone();
        let none_loaded = all_rows.is_empty();
        let q = self.extensions_search_query.trim().to_ascii_lowercase();

        let search_bar =
            self.list_panel_search_bar(window, cx, is_dark, ListPanelSearchKind::Extensions);

        let error_banner: Option<openframe::AnyElement> =
            self.python_extension_action_error.clone().map(|msg| {
                div()
                    .w_full()
                    .px_3()
                    .py_2()
                    .rounded(px(panel_radius.min(8.0)))
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

        let filtered: Vec<_> = all_rows
            .into_iter()
            .filter(|(name, version, description, _, _, _, tags)| {
                let mut extras = vec![version.as_str(), description.as_str()];
                extras.extend(tags.iter().map(String::as_str));
                list_panel_row_matches(&q, name, &extras)
            })
            .collect();

        let content = if none_loaded {
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_2()
                .py_10()
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
        } else if filtered.is_empty() && !self.extensions_search_query.trim().is_empty() {
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_2()
                .py_10()
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
                .children(filtered.into_iter().map(
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
                            panel_radius,
                            runtime_supported,
                            tags,
                        )
                    },
                ))
                .into_any_element()
        };

        if let Some(g) = theme::active_glyph(cx) {
            let inner = div()
                .w_full()
                .max_w(px(GLYPH_PANEL_CONTENT_MAX_W_PX))
                .p_4()
                .rounded(px(panel_radius.min(12.0)))
                .bg(g.surface)
                .border_1()
                .border_color(g.border)
                .flex()
                .flex_col()
                .gap_3()
                .child(search_bar);
            let inner = match error_banner {
                Some(banner) => inner.child(banner),
                None => inner,
            };
            div()
                .w_full()
                .flex()
                .justify_center()
                .child(inner.child(content))
                .into_any_element()
        } else {
            let inner = div()
                .w_full()
                .p_4()
                .rounded(px(panel_radius.min(12.0)))
                .bg(p.panel_bg)
                .border_1()
                .border_color(p.panel_border)
                .flex()
                .flex_col()
                .gap_3()
                .child(search_bar);
            let inner = match error_banner {
                Some(banner) => inner.child(banner),
                None => inner,
            };
            inner.child(content).into_any_element()
        }
    }

    pub(crate) fn python_extension_row(
        cx: &mut Context<Self>,
        name: String,
        version: String,
        description: String,
        enabled: bool,
        is_dark: bool,
        border_radius: f32,
        runtime_supported: bool,
        tags: Vec<String>,
    ) -> AnyElement {
        let p = theme::theme_palette(cx, is_dark);
        let is_glyph = theme::glyph_snapshot(cx).is_some();
        let r_track = border_radius.min(8.0_f32).max(0.0);

        let row_inner = div()
            .w_full()
            .px_4()
            .py_3()
            .flex()
            .justify_between()
            .items_center()
            .gap_4()
            .child(
                div()
                    .flex()
                    .items_start()
                    .gap_3()
                    .child({
                        let has_icon =
                            arcadia_core::modules::python_registry::resolve_extension_asset_path(
                                &name, "icon.svg",
                            )
                            .map(|p| p.exists())
                            .unwrap_or(false);
                        let icon_key = if has_icon {
                            format!("extension-icon/{name}")
                        } else {
                            "extensions".to_string()
                        };
                        render_icon(&icon_key)
                            .size_8()
                            .flex_shrink_0()
                            .text_color(p.content_title)
                    })
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(p.content_title)
                                    .child(name.clone()),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(p.content_meta)
                                            .child(format!("v{version}")),
                                    )
                                    .children(tags.iter().map(|tag| {
                                        div()
                                            .px_2()
                                            .py_0p5()
                                            .when(!is_glyph, |d| d.rounded_full())
                                            .rounded(px(border_radius.min(12.0)))
                                            .text_xs()
                                            .bg(p.surface_elevated)
                                            .text_color(p.ui_subtext)
                                            .child(tag.clone())
                                    })),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(p.content_body)
                                    .child(description),
                            )
                            .when(!runtime_supported, |col| {
                                col.child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(p.ui_subtext)
                                        .child("Platform Not Supported"),
                                )
                            }),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .when(runtime_supported || enabled, |d| d.cursor_pointer())
                    .when(!runtime_supported && !enabled, |d| d.cursor_default())
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(if enabled { p.accent } else { p.ui_subtext })
                            .child(if enabled { "Enabled" } else { "Disabled" }),
                    )
                    .child(if enabled {
                        div()
                            .w_10()
                            .h_6()
                            .px_0p5()
                            .when(!is_glyph, |d| d.rounded_full())
                            .rounded(px(r_track))
                            .border_1()
                            .border_color(p.border)
                            .bg(p.accent)
                            .flex()
                            .items_center()
                            .justify_end()
                            .child(
                                div()
                                    .w_4()
                                    .h_4()
                                    .when(!is_glyph, |d| d.rounded_full())
                                    .rounded(px(r_track))
                                    .bg(p.on_accent),
                            )
                    } else {
                        div()
                            .w_10()
                            .h_6()
                            .px_0p5()
                            .when(!is_glyph, |d| d.rounded_full())
                            .rounded(px(r_track))
                            .border_1()
                            .border_color(p.border)
                            .bg(p.surface_elevated)
                            .flex()
                            .items_center()
                            .justify_start()
                            .child(
                                div()
                                    .w_4()
                                    .h_4()
                                    .when(!is_glyph, |d| d.rounded_full())
                                    .rounded(px(r_track))
                                    .bg(p.toggle_knob_off),
                            )
                    })
                    .when(runtime_supported || enabled, |d| {
                        d.on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, _, cx| {
                                use crate::gui::app::PendingPermissionGrant;
                                use arcadia_core::config::modules::{
                                    ModulesConfig, PYTHON_HOST_MODULE_NAME,
                                };
                                use arcadia_core::config::permissions::{
                                    PermissionSubject, PermissionsConfig,
                                };
                                use arcadia_core::config::ConfigFile;

                                let enabled_next = !enabled;
                                if !runtime_supported {
                                    if !enabled {
                                        return;
                                    }
                                    if enabled_next {
                                        return;
                                    }
                                }

                                this.python_extension_action_error = None;
                                let ctx = this.execution_context();
                                let token = if enabled {
                                    "python-host.extension-disable"
                                } else {
                                    "python-host.extension-enable"
                                };

                                // Auto-heal: if the python-host module is enabled but its own
                                // required permissions were never granted (older builds, CLI
                                // toggles, manual file edits, etc.) the dispatcher rejects every
                                // extension-enable / disable call. Enabling the module is itself
                                // the user's consent for the module's declared perms, so fill in
                                // any missing grants on the `module:python-host` subject before
                                // routing the command. Only adds — never overrides explicit values.
                                if let Ok(host_modules) = ModulesConfig::load_or_create() {
                                    if host_modules
                                        .modules
                                        .get(PYTHON_HOST_MODULE_NAME)
                                        .copied()
                                        .unwrap_or(false)
                                    {
                                        if let (Some(manifest), Ok(mut pc)) = (
                                            ModulesConfig::manifest_for(PYTHON_HOST_MODULE_NAME),
                                            PermissionsConfig::load_or_create(),
                                        ) {
                                            let host_subj = PermissionSubject::module(
                                                PYTHON_HOST_MODULE_NAME.to_string(),
                                            );
                                            let needed: Vec<String> = manifest
                                                .required_permissions
                                                .iter()
                                                .map(|s| s.to_string())
                                                .collect();
                                            let missing = pc.missing_grants_for_declared(
                                                &host_subj,
                                                &manifest.required_permissions,
                                            );
                                            if !missing.is_empty() {
                                                let _ =
                                                    pc.ensure_effective_grants(&host_subj, &needed);
                                                let _ = pc.save();
                                            }
                                        }
                                    }
                                }

                                if !enabled {
                                    let decl =
                                        python_registry::extension_declared_permissions(&name);
                                    let rf: Vec<&str> = decl.iter().map(String::as_str).collect();
                                    if let Ok(pc) = PermissionsConfig::load_or_create() {
                                        let subj = PermissionSubject::python(name.clone());
                                        let miss = pc.missing_grants_for_declared(&subj, &rf);
                                        if !miss.is_empty() {
                                            this.pending_permission_grant =
                                                Some(PendingPermissionGrant::PythonExtension {
                                                    extension: name.clone(),
                                                    missing: miss,
                                                });
                                            cx.notify();
                                            return;
                                        }
                                    }
                                }
                                match modules::execute_command(token, &[name.as_str()], &ctx) {
                                    Ok(Some(msg)) => {
                                        // Command handlers report load failures via the response
                                        // string (`extension-enable` returns "...failed to load:
                                        // ..."). Treat any response that doesn't match the happy
                                        // path as an error so the panel can surface it.
                                        if msg.contains("failed") || msg.contains("Failed") {
                                            this.python_extension_action_error = Some(msg);
                                        }
                                    }
                                    Ok(None) => {
                                        this.python_extension_action_error = Some(format!(
                                            "Command {token} not found — is python-host loaded?"
                                        ));
                                    }
                                    Err(e) => {
                                        this.python_extension_action_error = Some(e);
                                    }
                                }
                                this.reload_extension_state(cx);
                                cx.notify();
                            }),
                        )
                    }),
            );

        if let Some(g) = theme::active_glyph(cx) {
            div()
                .w_full()
                .rounded(px(g.border_radius.min(12.0)))
                .bg(g.surface2)
                .border_1()
                .border_color(if enabled { p.accent } else { g.border })
                .child(row_inner)
                .into_any_element()
        } else {
            div()
                .w_full()
                .rounded(px(border_radius.min(12.0)))
                .bg(p.row_bg)
                .border_1()
                .border_color(if enabled { p.accent } else { p.row_border })
                .child(row_inner)
                .into_any_element()
        }
    }
}
