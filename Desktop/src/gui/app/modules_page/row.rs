use arcadia_core::config::modules::{ModuleManifest, ModulesConfig};
use arcadia_core::config::ConfigFile;
use arcadia_core::modules;
use openframe::prelude::FluentBuilder as _;
use openframe::{AnyElement, div, px};
use openframe::{Context, FontWeight, InteractiveElement, IntoElement, MouseButton, ParentElement, Styled};

use crate::gui::app::ArcadiaRoot;
use crate::gui::theme;

impl ArcadiaRoot {
    pub fn module_row_item(
        cx: &mut Context<Self>,
        module_name: String,
        enabled: bool,
        manifest: Option<&'static ModuleManifest>,
        is_dark: bool,
        runtime_supported: bool,
    ) -> AnyElement {
        let version = manifest.map(|m| m.version).unwrap_or("unknown");
        let description = manifest
            .map(|m| m.description)
            .unwrap_or("No manifest description.");
        let state = if enabled { "Enabled" } else { "Disabled" };

        let p = theme::theme_palette(cx, is_dark);
        let g_snap = theme::glyph_snapshot(cx);
        let is_glyph = g_snap.is_some();
        let radius = g_snap
            .map(|g| g.border_radius)
            .unwrap_or(p.radius_md);


        let title_c = p.content_title;
        let meta_c = p.content_meta;
        let desc_c = p.content_body;

        let (badge_bg, badge_fg) = if enabled {
            (p.accent, p.on_accent)
        } else {
            (p.surface_elevated, p.ui_subtext)
        };

        let row = div()
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
                    .gap_2()
                    .child(
                        div()
                            .text_base()
                            .font_weight(FontWeight::BOLD)
                            .text_color(title_c)
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
                                    .text_color(meta_c)
                                    .child(format!("v{version}")),
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py_0p5()
                                    .when(!is_glyph, |d| d.rounded_full())
                                    .rounded(px(radius.min(12.0)))
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .bg(badge_bg)
                                    .text_color(badge_fg)
                                    .child(state),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(desc_c)
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
            )
            .child(Self::row_toggle(
                cx,
                module_name,
                enabled,
                is_dark,
                is_glyph,
                radius,
                runtime_supported,
            ));

        if let Some(g) = theme::active_glyph(cx) {
            // Flat border + glyph colors: a full glyph border per row made scrolling layout-thrash
            // (thousands of repeated border glyphs × N rows each frame).
            div()
                .w_full()
                .rounded(px(g.border_radius.min(12.0)))
                .bg(g.surface2)
                .border_1()
                .border_color(g.border)
                .child(row)
                .into_any_element()
        } else {
            div()
                .w_full()
                .rounded(px(p.radius_md.min(12.0)))
                .bg(p.row_bg)
                .border_1()
                .border_color(p.row_border)
                .child(row)
                .into_any_element()
        }
    }

    fn row_toggle(
        cx: &mut Context<Self>,
        module_name: String,
        enabled: bool,
        is_dark: bool,
        is_glyph: bool,
        radius: f32,
        allow_enable: bool,
    ) -> impl IntoElement {
        let p = theme::theme_palette(cx, is_dark);
        let r_track = radius.min(8.0_f32).max(0.0);

        div()
            .flex()
            .items_center()
            .gap_2()
            .when(allow_enable || enabled, |d| d.cursor_pointer())
            .when(!allow_enable && !enabled, |d| d.cursor_default())
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(if enabled {
                        p.accent
                    } else {
                        p.ui_subtext
                    })
                    .child(if enabled { "ON" } else { "OFF" }),
            )
            .child(
                if enabled {
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
                },
            )
            .when(allow_enable || enabled, |d| {
                d.on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                let enabled_next = !enabled;
                if !allow_enable {
                    if !enabled {
                        return;
                    }
                    if enabled_next {
                        return;
                    }
                }
                if this.remote_route.is_some() {
                    let ctx = this.execution_context();
                    let name = module_name.clone();
                    let payload = arcadia_core::modules::surface::patch_json_modules_set(
                        &name,
                        enabled_next,
                        Some(this.surface_client_id.as_str()),
                    );
                    match modules::execute_command("surface.patch", &[payload.as_str()], &ctx) {
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
                            this.pending_module_enable = Some((module_name.clone(), missing));
                        }
                        Ok(_) => {
                            use arcadia_core::config::permissions::PermissionsConfig;
                            use crate::gui::app::PendingPermissionGrant;
                            if let Ok(pc) = PermissionsConfig::load_or_create() {
                                let miss = pc.missing_grants_for_module_enable(&module_name);
                                if !miss.is_empty() {
                                    this.pending_permission_grant = Some(
                                        PendingPermissionGrant::NativeModule {
                                            module: module_name.clone(),
                                            missing: miss,
                                        },
                                    );
                                    cx.notify();
                                    return;
                                }
                            }
                            if let Ok(mut cfg) = ModulesConfig::load_or_create() {
                                let _ = cfg.enable_with_requirements(&module_name);
                                let _ = cfg.save();
                            }
                            this.pending_module_enable = None;
                            this.pending_permission_grant = None;
                            this.reload_modules();
                        }
                        Err(err) => eprintln!("{err}"),
                    },
                    Err(err) => eprintln!("{err}"),
                }
                cx.notify();
            }))
            })
    }
}
