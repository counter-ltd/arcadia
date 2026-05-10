//! Generic port-collision recovery: when any service's `start` fails with an
//! "address already in use"-class error, the Services panel populates
//! `pending_port_kill_prompt` and this modal offers to terminate the prior Arcadia process
//! holding the port, then retry `service.controls.start`.
//!
//! No service-specific code lives here — the service tells us which port it tried to bind
//! via [`arcadia_core::services::ServiceControls::port_for_collision`].

use arcadia_core::services::{is_port_collision_error, service_by_id};
use openframe::{
    div, rgb, Context, FontWeight, InteractiveElement, IntoElement, MouseButton, ParentElement,
    Styled,
};

use crate::gui::app::{ArcadiaRoot, PendingPortKill};

impl ArcadiaRoot {
    /// Generic "kill any existing Arcadia owner of `port`" then re-invoke the service's
    /// `start`. macOS/Linux only — uses `lsof`/`ps`/`kill`. Returns `Ok` if a process was
    /// killed and the retry was attempted (the retry result is surfaced via `lan_service_feedback`).
    pub(crate) fn kill_existing_port_owner_and_retry(
        &mut self,
        service_id: &'static str,
        port: u16,
    ) -> Result<(), String> {
        use std::process::Command;
        let Some(service) = service_by_id(service_id) else {
            return Err(format!("Unknown service: {service_id}"));
        };
        let current_pid = std::process::id();
        let output = Command::new("lsof")
            .args(["-nP", "-t", &format!("-iUDP:{port}"), &format!("-iTCP:{port}")])
            .output()
            .map_err(|err| format!("Failed to inspect port {port} usage: {err}"))?;

        if !output.status.success() && output.stdout.is_empty() {
            return Err(format!("No process currently owns port {port}."));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut killed_any = false;
        for line in stdout.lines() {
            let pid = match line.trim().parse::<u32>() {
                Ok(pid) if pid != current_pid => pid,
                _ => continue,
            };

            let command_output = Command::new("ps")
                .args(["-p", &pid.to_string(), "-o", "command="])
                .output()
                .map_err(|err| format!("Failed to inspect process {pid}: {err}"))?;
            let command_text =
                String::from_utf8_lossy(&command_output.stdout).to_ascii_lowercase();
            if !command_text.contains("arcadia") {
                continue;
            }

            let status = Command::new("kill")
                .arg(pid.to_string())
                .status()
                .map_err(|err| format!("Failed to signal process {pid}: {err}"))?;
            if !status.success() {
                return Err(format!("Failed to terminate process {pid}."));
            }
            killed_any = true;
        }

        if !killed_any {
            return Err(format!(
                "No existing Arcadia process found owning port {port}."
            ));
        }

        // Background processes need a moment to release the socket before retry.
        std::thread::sleep(std::time::Duration::from_millis(400));

        if let Some(start) = service.controls.start {
            match start() {
                Ok(()) => {
                    self.lan_service_feedback = format!("{} started.", service.title);
                }
                Err(err) => {
                    self.lan_service_feedback =
                        format!("Failed to start {}: {err}", service.title);
                    if is_port_collision_error(&err) {
                        self.pending_port_kill_prompt = Some(PendingPortKill {
                            service_id: service.id,
                            service_title: service.title,
                            port,
                            error: err,
                        });
                    }
                }
            }
        }
        Ok(())
    }

    pub fn kill_existing_port_modal(
        &self,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> impl IntoElement {
        let Some(pending) = self.pending_port_kill_prompt.clone() else {
            return div();
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
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.pending_port_kill_prompt = None;
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
                            .rounded_lg()
                            .bg(if is_dark { rgb(0x111827) } else { rgb(0xffffff) })
                            .border_1()
                            .border_color(if is_dark { rgb(0x374151) } else { rgb(0xe2e8f0) })
                            .flex()
                            .flex_col()
                            .gap_3()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(if is_dark { rgb(0xf9fafb) } else { rgb(0x111827) })
                                    .child("Kill Existing?"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(if is_dark { rgb(0xd1d5db) } else { rgb(0x374151) })
                                    .child(format!(
                                        "{} failed to bind port {}: {}",
                                        pending.service_title, pending.port, pending.error
                                    )),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(if is_dark { rgb(0x9ca3af) } else { rgb(0x6b7280) })
                                    .child("Arcadia will terminate any older Arcadia process holding this port and retry Start."),
                            )
                            .child(self.kill_modal_actions(cx, is_dark, pending)),
                    ),
            )
    }

    fn kill_modal_actions(
        &self,
        cx: &mut Context<Self>,
        is_dark: bool,
        pending: PendingPortKill,
    ) -> impl IntoElement {
        let service_id = pending.service_id;
        let port = pending.port;
        div()
            .flex()
            .gap_2()
            .justify_end()
            .child(
                div()
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .cursor_pointer()
                    .bg(if is_dark { rgb(0x374151) } else { rgb(0xe5e7eb) })
                    .text_color(if is_dark { rgb(0xf3f4f6) } else { rgb(0x1f2937) })
                    .child("Cancel")
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.pending_port_kill_prompt = None;
                            cx.notify();
                        }),
                    ),
            )
            .child(
                div()
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .cursor_pointer()
                    .bg(rgb(0xdbeafe))
                    .text_color(rgb(0x1d4ed8))
                    .child("Kill Existing")
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.pending_port_kill_prompt = None;
                            if let Err(err) = this.kill_existing_port_owner_and_retry(service_id, port) {
                                this.lan_service_feedback = err;
                            }
                            cx.notify();
                        }),
                    ),
            )
    }
}
