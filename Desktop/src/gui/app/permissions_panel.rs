use arcadia_core::config::modules::MODULE_REGISTRY;
use arcadia_core::config::permissions::{
    permission_definition, PermissionSubject, PermissionsConfig, PERMISSION_REGISTRY,
};
use arcadia_core::config::ConfigFile;
use arcadia_core::modules;
use openframe::prelude::FluentBuilder as _;
use openframe::{
    div, px, AnyElement, Context, FontWeight, InteractiveElement, IntoElement, MouseButton,
    ParentElement, Styled, Window,
};

use crate::gui::app::list_panel_search::{list_panel_row_matches, ListPanelSearchKind};
use crate::gui::app::ArcadiaRoot;
use crate::gui::theme::{self, GLYPH_PANEL_CONTENT_MAX_W_PX};

fn perm_toggle_switch(
    on: bool,
    is_glyph: bool,
    r_track: f32,
    p: &theme::palette::ThemePalette,
) -> AnyElement {
    if on {
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
            .into_any_element()
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
            .into_any_element()
    }
}

impl ArcadiaRoot {
    pub fn permissions_panel(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> AnyElement {
        if self.active_page_id.as_str() != "global.permissions" {
            return div().into_any_element();
        }
        let p = theme::theme_palette(cx, is_dark);
        let g_snap = theme::glyph_snapshot(cx);
        let panel_radius = g_snap.map(|g| g.border_radius).unwrap_or(p.radius_md);
        let is_glyph = g_snap.is_some();
        let r_track = panel_radius.min(8.0_f32).max(0.0);
        let q = self.permissions_search_query.trim().to_ascii_lowercase();
        let search_bar =
            self.list_panel_search_bar(window, cx, is_dark, ListPanelSearchKind::Permissions);

        let Ok(pc) = PermissionsConfig::load_or_create() else {
            return div()
                .text_sm()
                .text_color(p.content_title)
                .child("Could not load permissions.toml")
                .into_any_element();
        };

        // ── Global permissions ──────────────────────────────────────────────
        let mut global_children: Vec<AnyElement> = Vec::new();
        for def in PERMISSION_REGISTRY {
            if !list_panel_row_matches(&q, def.id, &[def.title, def.description]) {
                continue;
            }
            let id = def.id.to_string();
            let title = def.title.to_string();
            let desc = def.description.to_string();
            let on = pc.global_allowed(def.id);
            let id_c = id.clone();
            let toggle = perm_toggle_switch(on, is_glyph, r_track, &p);

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
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(p.content_title)
                                .child(title),
                        )
                        .child(div().text_xs().text_color(p.content_meta).child(id.clone()))
                        .when(!desc.is_empty(), |d| {
                            d.child(div().text_xs().text_color(p.content_body).child(desc))
                        }),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .cursor_pointer()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(if on { p.accent } else { p.ui_subtext })
                                .child(if on { "ON" } else { "OFF" }),
                        )
                        .child(toggle)
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, _, cx| {
                                let ctx = this.execution_context();
                                let next = if on { "false" } else { "true" };
                                let _ = modules::execute_command(
                                    "permissions.global-set",
                                    &[id_c.as_str(), next],
                                    &ctx,
                                );
                                cx.notify();
                            }),
                        ),
                );

            global_children.push(
                if let Some(g) = g_snap {
                    div()
                        .w_full()
                        .rounded(px(g.border_radius.min(12.0)))
                        .bg(g.surface2)
                        .border_1()
                        .border_color(if on { p.accent } else { g.border })
                        .child(row_inner)
                } else {
                    div()
                        .w_full()
                        .rounded(px(panel_radius.min(12.0)))
                        .bg(p.row_bg)
                        .border_1()
                        .border_color(if on { p.accent } else { p.row_border })
                        .child(row_inner)
                }
                .into_any_element(),
            );
        }

        // ── Native module permissions ───────────────────────────────────────
        let native_catalog_nonempty = MODULE_REGISTRY
            .iter()
            .any(|m| !m.required_permissions.is_empty());

        let mut module_children: Vec<AnyElement> = Vec::new();
        for m in MODULE_REGISTRY.iter() {
            if m.required_permissions.is_empty() {
                continue;
            }
            let mod_name = m.name;
            let header_matches = list_panel_row_matches(&q, mod_name, &[]);
            let pids_to_show: Vec<&'static str> = if q.is_empty() || header_matches {
                m.required_permissions.iter().copied().collect()
            } else {
                m.required_permissions
                    .iter()
                    .copied()
                    .filter(|pid| {
                        if let Some(d) = permission_definition(pid) {
                            list_panel_row_matches(&q, pid, &[mod_name, d.title, d.description])
                        } else {
                            list_panel_row_matches(&q, pid, &[mod_name])
                        }
                    })
                    .collect()
            };
            if pids_to_show.is_empty() {
                continue;
            }

            let mod_name_owned = mod_name.to_string();
            let subj = PermissionSubject::module(mod_name_owned.clone());
            let sk = subj.storage_key();

            module_children.push(
                div()
                    .w_full()
                    .pt_2()
                    .pb_1()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(p.content_title)
                            .child(mod_name_owned.clone()),
                    )
                    .into_any_element(),
            );

            for pid in pids_to_show {
                let pid_s = pid.to_string();
                let sk_c = sk.clone();
                let mod_for_grant = mod_name_owned.clone();
                let granted =
                    pc.subject_grants(&PermissionSubject::module(mod_for_grant.clone()), pid);
                let def = permission_definition(pid);
                let row_title = def.map(|d| d.title).unwrap_or(pid);
                let row_desc = def.map(|d| d.description).unwrap_or("");
                let toggle = perm_toggle_switch(granted, is_glyph, r_track, &p);

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
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(p.content_title)
                                    .child(row_title.to_string()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(p.content_meta)
                                    .child(pid_s.clone()),
                            )
                            .when(!row_desc.is_empty(), |d| {
                                d.child(
                                    div()
                                        .text_xs()
                                        .text_color(p.content_body)
                                        .child(row_desc.to_string()),
                                )
                            }),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .cursor_pointer()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(if granted { p.accent } else { p.ui_subtext })
                                    .child(if granted { "ON" } else { "OFF" }),
                            )
                            .child(toggle)
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, _, cx| {
                                    let ctx = this.execution_context();
                                    let next = if granted { "false" } else { "true" };
                                    let _ = modules::execute_command(
                                        "permissions.grant-set",
                                        &[sk_c.as_str(), pid_s.as_str(), next],
                                        &ctx,
                                    );
                                    cx.notify();
                                }),
                            ),
                    );

                module_children.push(
                    if let Some(g) = g_snap {
                        div()
                            .w_full()
                            .rounded(px(g.border_radius.min(12.0)))
                            .bg(g.surface2)
                            .border_1()
                            .border_color(if granted { p.accent } else { g.border })
                            .child(row_inner)
                    } else {
                        div()
                            .w_full()
                            .rounded(px(panel_radius.min(12.0)))
                            .bg(p.row_bg)
                            .border_1()
                            .border_color(if granted { p.accent } else { p.row_border })
                            .child(row_inner)
                    }
                    .into_any_element(),
                );
            }
        }

        // ── Python extension permissions ────────────────────────────────────
        let py_catalog_nonempty = self
            .python_extension_rows
            .iter()
            .any(|(_, _, _, enabled, perms, _)| *enabled && !perms.is_empty());

        let mut py_children: Vec<AnyElement> = Vec::new();
        for (name, _ver, _desc, enabled, perms, _platforms) in &self.python_extension_rows {
            if !enabled || perms.is_empty() {
                continue;
            }
            let ext = name.clone();
            let py_prefix = format!("python:{ext}");
            let header_matches = list_panel_row_matches(&q, ext.as_str(), &[])
                || list_panel_row_matches(&q, py_prefix.as_str(), &[]);
            let pids_to_show: Vec<&String> = if q.is_empty() || header_matches {
                perms.iter().collect()
            } else {
                perms
                    .iter()
                    .filter(|pid| {
                        let pid_s = pid.as_str();
                        if let Some(d) = permission_definition(pid_s) {
                            list_panel_row_matches(
                                &q,
                                pid_s,
                                &[ext.as_str(), py_prefix.as_str(), d.title, d.description],
                            )
                        } else {
                            list_panel_row_matches(&q, pid_s, &[ext.as_str(), py_prefix.as_str()])
                        }
                    })
                    .collect()
            };
            if pids_to_show.is_empty() {
                continue;
            }

            let sk = PermissionSubject::python(ext.clone()).storage_key();
            py_children.push(
                div()
                    .w_full()
                    .pt_2()
                    .pb_1()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(p.content_title)
                            .child(py_prefix.clone()),
                    )
                    .into_any_element(),
            );

            for pid in pids_to_show {
                let pid_s = pid.clone();
                let sk_c = sk.clone();
                let ext_c = ext.clone();
                let granted =
                    pc.subject_grants(&PermissionSubject::python(ext_c.clone()), pid.as_str());
                let def = permission_definition(pid.as_str());
                let row_title = def.map(|d| d.title).unwrap_or(pid.as_str());
                let row_desc = def.map(|d| d.description).unwrap_or("");
                let toggle = perm_toggle_switch(granted, is_glyph, r_track, &p);

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
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(p.content_title)
                                    .child(row_title.to_string()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(p.content_meta)
                                    .child(pid_s.clone()),
                            )
                            .when(!row_desc.is_empty(), |d| {
                                d.child(
                                    div()
                                        .text_xs()
                                        .text_color(p.content_body)
                                        .child(row_desc.to_string()),
                                )
                            }),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .cursor_pointer()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(if granted { p.accent } else { p.ui_subtext })
                                    .child(if granted { "ON" } else { "OFF" }),
                            )
                            .child(toggle)
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, _, cx| {
                                    let ctx = this.execution_context();
                                    let next = if granted { "false" } else { "true" };
                                    let _ = modules::execute_command(
                                        "permissions.grant-set",
                                        &[sk_c.as_str(), pid_s.as_str(), next],
                                        &ctx,
                                    );
                                    cx.notify();
                                }),
                            ),
                    );

                py_children.push(
                    if let Some(g) = g_snap {
                        div()
                            .w_full()
                            .rounded(px(g.border_radius.min(12.0)))
                            .bg(g.surface2)
                            .border_1()
                            .border_color(if granted { p.accent } else { g.border })
                            .child(row_inner)
                    } else {
                        div()
                            .w_full()
                            .rounded(px(panel_radius.min(12.0)))
                            .bg(p.row_bg)
                            .border_1()
                            .border_color(if granted { p.accent } else { p.row_border })
                            .child(row_inner)
                    }
                    .into_any_element(),
                );
            }
        }

        // ── Assemble sections ───────────────────────────────────────────────
        let global_block =
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .text_lg()
                        .font_weight(FontWeight::BOLD)
                        .text_color(p.content_title)
                        .child("Global"),
                )
                .child(div().text_xs().text_color(p.content_meta).child(
                    "Kill switches for capability classes. Per-module grants still required.",
                ))
                .child(if global_children.is_empty() && !q.is_empty() {
                    div()
                        .text_xs()
                        .text_color(p.content_meta)
                        .child("No matching global permissions.")
                        .into_any_element()
                } else {
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .children(global_children)
                        .into_any_element()
                });

        let native_empty_msg = if !native_catalog_nonempty || module_children.is_empty() {
            if !q.is_empty() {
                "No matching native module permissions."
            } else {
                "No declared module permissions."
            }
        } else {
            ""
        };

        let native_block = div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::BOLD)
                    .text_color(p.content_title)
                    .child("Native modules"),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(p.content_meta)
                    .child("Per-module grants (module:key in permissions.toml)."),
            )
            .child(if module_children.is_empty() {
                div()
                    .text_xs()
                    .text_color(p.content_meta)
                    .child(native_empty_msg)
                    .into_any_element()
            } else {
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .children(module_children)
                    .into_any_element()
            });

        let py_empty_msg = if !py_catalog_nonempty || py_children.is_empty() {
            if !q.is_empty() {
                "No matching Python extension permissions."
            } else {
                "No extensions declare permissions, or python-host is off."
            }
        } else {
            ""
        };

        let py_block = div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::BOLD)
                    .text_color(p.content_title)
                    .child("Python extensions"),
            )
            .child(if py_children.is_empty() {
                div()
                    .text_xs()
                    .text_color(p.content_meta)
                    .child(py_empty_msg)
                    .into_any_element()
            } else {
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .children(py_children)
                    .into_any_element()
            });

        let body = div()
            .w_full()
            .flex()
            .flex_col()
            .gap_4()
            .child(search_bar)
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .gap_6()
                    .child(global_block)
                    .child(native_block)
                    .child(py_block),
            );

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
