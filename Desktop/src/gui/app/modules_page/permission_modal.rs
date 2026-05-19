use openframe::{
    div, px, rgb, Context, InteractiveElement, IntoElement, MouseButton, ParentElement, Styled,
};

use crate::gui::app::{ArcadiaRoot, PendingPermissionGrant};
use crate::gui::theme;
use arcadia_core::config::modules::ModulesConfig;
use arcadia_core::config::permissions::{PermissionSubject, PermissionsConfig};
use arcadia_core::config::ConfigFile;
use arcadia_core::modules;

impl ArcadiaRoot {
    pub fn permission_grant_modal(
        &self,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> impl IntoElement {
        let Some(grant) = &self.pending_permission_grant else {
            return div();
        };
        let (title, requirements) = match grant {
            PendingPermissionGrant::NativeModule { module, missing } => (
                format!("Grant permissions for {module}?"),
                missing.join(", "),
            ),
            PendingPermissionGrant::PythonExtension { extension, missing } => (
                format!("Grant permissions for extension '{extension}'?"),
                missing.join(", "),
            ),
            PendingPermissionGrant::WasmModule { module, missing } => (
                format!("Grant permissions for WASM module '{module}'?"),
                missing.join(", "),
            ),
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
                    .opacity(0.35)
                    .on_mouse_down(
                        openframe::MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.pending_permission_grant = None;
                            cx.notify();
                        }),
                    ),
            )
            .child(
                div()
                    .size_full()
                    .flex()
                    .justify_center()
                    .items_center()
                    .child(
                        div()
                            .w_128()
                            .p_5()
                            .rounded(px(theme::ui_radius(cx)))
                            .bg(theme::ui_surface(cx, is_dark))
                            .border_1()
                            .border_color(theme::ui_border(cx, is_dark))
                            .flex()
                            .flex_col()
                            .gap_3()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(openframe::FontWeight::BOLD)
                                    .text_color(theme::ui_text(cx, is_dark))
                                    .child(title),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme::ui_subtext(cx, is_dark))
                                    .child(format!(
                                        "Arcadia needs these permissions: {requirements}."
                                    )),
                            )
                            .child(
                                div()
                                    .flex()
                                    .gap_2()
                                    .justify_end()
                                    .child(
                                        div()
                                            .px_3()
                                            .py_2()
                                            .rounded(px(theme::ui_radius(cx)))
                                            .cursor_pointer()
                                            .bg(theme::ui_surface2(cx, is_dark))
                                            .text_color(theme::ui_text(cx, is_dark))
                                            .child("Cancel")
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(|this, _, _, cx| {
                                                    this.pending_permission_grant = None;
                                                    cx.notify();
                                                }),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .px_3()
                                            .py_2()
                                            .rounded(px(theme::ui_radius(cx)))
                                            .cursor_pointer()
                                            .bg(theme::ui_accent(cx))
                                            .text_color(theme::ui_accent_fg(cx))
                                            .child("Grant")
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(move |this, _, _, cx| {
                                                    let Some(g) =
                                                        this.pending_permission_grant.clone()
                                                    else {
                                                        return;
                                                    };
                                                    match g {
                                                    PendingPermissionGrant::NativeModule {
                                                        module,
                                                        missing,
                                                    } => {
                                                        if let Ok(mut pc) =
                                                            PermissionsConfig::load_or_create()
                                                        {
                                                            let subj = PermissionSubject::module(
                                                                module.clone(),
                                                            );
                                                            let _ = pc.ensure_effective_grants(
                                                                &subj, &missing,
                                                            );
                                                            let _ = pc.save();
                                                        }
                                                        if let Ok(mut cfg) =
                                                            ModulesConfig::load_or_create()
                                                        {
                                                            let _ = cfg
                                                                .enable_with_requirements(&module);
                                                            let _ = cfg.save();
                                                        }
                                                        this.reload_modules();
                                                    }
                                                    PendingPermissionGrant::PythonExtension {
                                                        extension,
                                                        missing,
                                                    } => {
                                                        if let Ok(mut pc) =
                                                            PermissionsConfig::load_or_create()
                                                        {
                                                            let subj = PermissionSubject::python(
                                                                extension.clone(),
                                                            );
                                                            let _ = pc.ensure_effective_grants(
                                                                &subj, &missing,
                                                            );
                                                            let _ = pc.save();
                                                        }
                                                        let ctx = this.execution_context();
                                                        let _ = modules::execute_command(
                                                            "python-host.extension-enable",
                                                            &[extension.as_str()],
                                                            &ctx,
                                                        );
                                                        this.reload_extension_state(cx);
                                                    }
                                                    PendingPermissionGrant::WasmModule {
                                                        module,
                                                        missing,
                                                    } => {
                                                        if let Ok(mut pc) =
                                                            PermissionsConfig::load_or_create()
                                                        {
                                                            let subj = PermissionSubject::wasm(
                                                                module.clone(),
                                                            );
                                                            let _ = pc.ensure_effective_grants(
                                                                &subj, &missing,
                                                            );
                                                            let _ = pc.save();
                                                        }
                                                        let ctx = this.execution_context();
                                                        let _ = modules::execute_command(
                                                            "wasm-host.module-enable",
                                                            &[module.as_str()],
                                                            &ctx,
                                                        );
                                                        this.reload_wasm_state(cx);
                                                    }
                                                }
                                                    this.pending_permission_grant = None;
                                                    cx.notify();
                                                }),
                                            ),
                                    ),
                            ),
                    ),
            )
    }
}
