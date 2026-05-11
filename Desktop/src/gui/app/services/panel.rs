//! Services panel — registry-driven. Each service row reports availability and exposes the
//! controls it advertised in [`arcadia_core::services::ServiceControls`]. No service-id
//! switching here: a new service appears as soon as it lands in `SERVICE_DEFINITIONS`.

use arcadia_core::services::{
    is_port_collision_error, services_for_page, ServiceDefinition, ServiceRuntimeStatus,
};
use openframe::{
    div, Context, FontWeight, InteractiveElement, IntoElement, MouseButton, ParentElement, Styled,
};

use crate::gui::app::{ArcadiaRoot, PendingPortKill};
use crate::gui::theme;

impl ArcadiaRoot {
    pub fn services_panel(&self, cx: &mut Context<Self>, is_dark: bool) -> impl IntoElement {
        let services = services_for_page("utility.services");
        let p = theme::theme_palette(cx, is_dark);
        let meta_c = p.content_meta;
        let desc_c = p.content_body;
        let text_c = p.content_title;
        let bg_c = p.panel_bg;
        let border_c = p.panel_border;
        let row_bg = p.row_bg;
        let row_str = p.row_border;
        let radius = p.radius_md;

        let mut container = div().w_full().flex().flex_col().gap_3().child(
            div()
                .text_sm()
                .text_color(meta_c)
                .child(
                    "Modules that expose long-running services register here. Disable the \
                    underlying module to hide a service.",
                ),
        );

        if services.is_empty() {
            container = container.child(
                div()
                    .text_sm()
                    .text_color(desc_c)
                    .child("No services registered."),
            );
            return container;
        }

        for service in services {
            container =
                container.child(self.service_row(cx, service, is_dark, text_c, meta_c, desc_c, bg_c, border_c, row_bg, row_str, radius));
        }
        container.child(self.services_feedback_row(cx, is_dark))
    }

    #[allow(clippy::too_many_arguments)]
    fn service_row(
        &self,
        cx: &mut Context<Self>,
        service: &'static ServiceDefinition,
        is_dark: bool,
        text_c: openframe::Rgba,
        meta_c: openframe::Rgba,
        desc_c: openframe::Rgba,
        bg_c: openframe::Rgba,
        border_c: openframe::Rgba,
        row_bg: openframe::Rgba,
        row_str: openframe::Rgba,
        radius: f32,
    ) -> impl IntoElement {
        let module_enabled = self.is_module_enabled(service.required_module);
        let runtime = service.controls.status_detail.map(|f| f());
        let badge = ServiceBadge::resolve(module_enabled, runtime.as_ref());
        let detail_line = match (module_enabled, runtime.as_ref()) {
            (false, _) => format!("module: {} · disabled", service.required_module),
            (true, Some(r)) => format!("module: {} · {}", service.required_module, r.detail),
            (true, None) => format!("module: {} · available", service.required_module),
        };

        div()
            .w_full()
            .px_4()
            .py_3()
            .rounded(openframe::px(radius.min(12.0)))
            .bg(bg_c)
            .border_1()
            .border_color(border_c)
            .flex()
            .items_center()
            .justify_between()
            .gap_3()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .flex_1()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(text_c)
                            .child(service.title),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(meta_c)
                            .child(service.description),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(desc_c)
                            .child(detail_line),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(badge.into_view(cx, is_dark))
                    .child(self.service_controls_row(
                        cx,
                        service,
                        module_enabled,
                        is_dark,
                        text_c,
                        row_bg,
                        row_str,
                        radius,
                    )),
            )
    }

    fn service_controls_row(
        &self,
        cx: &mut Context<Self>,
        service: &'static ServiceDefinition,
        module_enabled: bool,
        is_dark: bool,
        text_c: openframe::Rgba,
        row_bg: openframe::Rgba,
        row_str: openframe::Rgba,
        radius: f32,
    ) -> impl IntoElement {
        let mut row = div().flex().items_center().gap_2();
        if !module_enabled {
            return row;
        }
        let running = service.controls.status_detail.map(|f| f().running).flatten();
        if let Some(stop) = service.controls.stop {
            if running == Some(true) {
                row = row.child(self.service_button(cx, "Stop", is_dark, text_c, row_bg, row_str, radius, move |this, cx| {
                    stop();
                    this.lan_service_feedback = format!("{} stopped.", service.title);
                    cx.notify();
                }));
            }
        }
        if let Some(start) = service.controls.start {
            if running != Some(true) {
                row = row.child(self.service_button(cx, "Start", is_dark, text_c, row_bg, row_str, radius, move |this, cx| {
                    match start() {
                        Ok(()) => {
                            this.lan_service_feedback = format!("{} started.", service.title);
                            this.pending_port_kill_prompt = None;
                        }
                        Err(err) => {
                            this.lan_service_feedback =
                                format!("Failed to start {}: {err}", service.title);
                            // Generic port-collision recovery: services that advertise a port
                            // (`controls.port_for_collision`) get the Kill Existing modal
                            // automatically — no per-service GUI code.
                            if is_port_collision_error(&err) {
                                if let Some(port_fn) = service.controls.port_for_collision {
                                    this.pending_port_kill_prompt = Some(PendingPortKill {
                                        service_id: service.id,
                                        service_title: service.title,
                                        port: port_fn(),
                                        error: err,
                                    });
                                }
                            }
                        }
                    }
                    cx.notify();
                }));
            }
        }
        if service.controls.status_detail.is_some() {
            row = row.child(self.service_button(cx, "Refresh", is_dark, text_c, row_bg, row_str, radius, move |this, cx| {
                this.lan_service_feedback = format!("{} status refreshed.", service.title);
                cx.notify();
            }));
        }
        row
    }

    fn service_button<F>(
        &self,
        cx: &mut Context<Self>,
        label: &'static str,
        _is_dark: bool,
        text_c: openframe::Rgba,
        row_bg: openframe::Rgba,
        row_str: openframe::Rgba,
        radius: f32,
        on_click: F,
    ) -> impl IntoElement
    where
        F: Fn(&mut ArcadiaRoot, &mut Context<ArcadiaRoot>) + 'static,
    {
        div()
            .cursor_pointer()
            .px_3()
            .py_1()
            .rounded(openframe::px(radius.min(6.0)))
            .bg(row_bg)
            .border_1()
            .border_color(row_str)
            .text_sm()
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(text_c)
            .child(label)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| on_click(this, cx)),
            )
    }

    fn services_feedback_row(&self, cx: &mut Context<Self>, is_dark: bool) -> impl IntoElement {
        div()
            .text_sm()
            .text_color(theme::content_subdued_text(cx, is_dark))
            .child(if self.lan_service_feedback.is_empty() {
                "Service action output appears here.".to_string()
            } else {
                self.lan_service_feedback.clone()
            })
    }
}

#[derive(Clone, Copy)]
enum ServiceBadge {
    Running,
    Available,
    Unavailable,
}

impl ServiceBadge {
    fn resolve(module_enabled: bool, runtime: Option<&ServiceRuntimeStatus>) -> Self {
        if !module_enabled {
            return ServiceBadge::Unavailable;
        }
        match runtime.and_then(|r| r.running) {
            Some(true) => ServiceBadge::Running,
            Some(false) => ServiceBadge::Available,
            None => ServiceBadge::Available,
        }
    }

    fn into_view(self, cx: &openframe::App, is_dark: bool) -> impl IntoElement {
        let p = theme::theme_palette(cx, is_dark);
        let (label, bg, fg) = match self {
            ServiceBadge::Running => ("running", p.accent, p.on_accent),
            ServiceBadge::Available => ("available", p.badge_info_bg, p.badge_info_fg),
            ServiceBadge::Unavailable => ("unavailable", p.badge_muted_bg, p.badge_muted_fg),
        };
        div()
            .px_2()
            .py_0p5()
            .rounded_full()
            .text_xs()
            .font_weight(FontWeight::SEMIBOLD)
            .bg(bg)
            .text_color(fg)
            .child(label)
    }
}
