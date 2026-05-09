use arcadia_core::config::modules::{LAN_MODULE_NAME, NET_MODULE_NAME};
use arcadia_core::modules::lan::{lan_service_info, start_service, stop_service};
use openframe::{
    div, rgb, Context, InteractiveElement, IntoElement, MouseButton, ParentElement, Styled,
};

use crate::gui::app::ArcadiaRoot;
use crate::gui::theme;

impl ArcadiaRoot {
    fn try_start_lan_service(&mut self) {
        match start_service() {
            Ok(()) => {
                self.lan_service_feedback = "LAN discovery service started.".to_string();
            }
            Err(err) => {
                self.lan_service_feedback =
                    format!("Failed to start LAN discovery service: {err}");
                let lower = err.to_ascii_lowercase();
                if lower.contains("address already in use") || lower.contains("port") {
                    self.pending_lan_port_kill_prompt = Some(err);
                }
            }
        }
    }

    fn kill_existing_lan_port_owner_and_retry(&mut self) -> Result<(), String> {
        use std::process::Command;
        let info = lan_service_info();
        let current_pid = std::process::id();
        let output = Command::new("lsof")
            .args(["-nP", "-t", &format!("-iUDP:{}", info.port)])
            .output()
            .map_err(|err| format!("Failed to inspect UDP {} usage: {err}", info.port))?;

        if !output.status.success() && output.stdout.is_empty() {
            return Err(format!("No process currently owns UDP {}.", info.port));
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
            let command_text = String::from_utf8_lossy(&command_output.stdout).to_ascii_lowercase();
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
                "No existing Arcadia process found owning UDP {}.",
                info.port
            ));
        }

        // Give the killed process time to release the socket before retrying.
        std::thread::sleep(std::time::Duration::from_millis(400));
        self.try_start_lan_service();
        Ok(())
    }

    pub fn network_overview_panel(
        &self,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> impl IntoElement {
        let net_enabled = self.is_module_enabled(NET_MODULE_NAME);
        let lan_enabled = self.is_module_enabled(LAN_MODULE_NAME);

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_4()
            .child(self.net_status_row(net_enabled, is_dark))
            .child(if lan_enabled {
                self.lan_service_row(cx, is_dark).into_any_element()
            } else {
                div()
                    .text_sm()
                    .text_color(theme::module_description_text(is_dark))
                    .child("Enable the LAN module to manage the discovery service.")
                    .into_any_element()
            })
            .child(
                div()
                    .text_sm()
                    .text_color(theme::module_description_text(is_dark))
                    .child(self.lan_service_feedback.clone()),
            )
    }

    fn net_status_row(&self, net_enabled: bool, is_dark: bool) -> impl IntoElement {
        div()
            .w_full()
            .px_4()
            .py_3()
            .rounded_lg()
            .bg(theme::module_panel_bg(is_dark))
            .border_1()
            .border_color(theme::module_panel_stroke(is_dark))
            .flex()
            .items_center()
            .justify_between()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(openframe::FontWeight::SEMIBOLD)
                            .text_color(theme::module_title_text(is_dark))
                            .child("Network Module"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::module_meta_text(is_dark))
                            .child("Networking foundation — required by LAN and remote session"),
                    ),
            )
            .child(self.status_badge(net_enabled, is_dark))
    }

    fn lan_service_row(&self, cx: &mut Context<Self>, is_dark: bool) -> impl IntoElement {
        let info = lan_service_info();
        let running = info.running;
        div()
            .w_full()
            .px_4()
            .py_3()
            .rounded_lg()
            .bg(theme::module_panel_bg(is_dark))
            .border_1()
            .border_color(theme::module_panel_stroke(is_dark))
            .flex()
            .items_center()
            .justify_between()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(openframe::FontWeight::SEMIBOLD)
                            .text_color(theme::module_title_text(is_dark))
                            .child("LAN Discovery Service"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::module_meta_text(is_dark))
                            .child(format!("UDP :{} · {}", info.port, info.hostname)),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(self.service_running_badge(running, is_dark))
                    .child(
                        div()
                            .cursor_pointer()
                            .px_3()
                            .py_1()
                            .rounded_md()
                            .bg(theme::module_row_bg(is_dark))
                            .border_1()
                            .border_color(theme::module_row_stroke(is_dark))
                            .text_sm()
                            .font_weight(openframe::FontWeight::SEMIBOLD)
                            .text_color(theme::module_title_text(is_dark))
                            .child("Refresh")
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, _, cx| {
                                    this.lan_service_feedback =
                                        "LAN discovery status refreshed.".to_string();
                                    cx.notify();
                                }),
                            ),
                    )
                    .child(
                        div()
                            .cursor_pointer()
                            .px_3()
                            .py_1()
                            .rounded_md()
                            .bg(theme::module_row_bg(is_dark))
                            .border_1()
                            .border_color(theme::module_row_stroke(is_dark))
                            .text_sm()
                            .font_weight(openframe::FontWeight::SEMIBOLD)
                            .text_color(theme::module_title_text(is_dark))
                            .child(if running { "Stop" } else { "Start" })
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, _, cx| {
                                    if running {
                                        stop_service();
                                        this.pending_lan_port_kill_prompt = None;
                                        this.lan_service_feedback =
                                            "LAN discovery service stopped.".to_string();
                                    } else {
                                        this.try_start_lan_service();
                                    }
                                    cx.notify();
                                }),
                            ),
                    ),
            )
    }

    pub fn kill_existing_lan_modal(&self, cx: &mut Context<Self>, is_dark: bool) -> impl IntoElement {
        let Some(error_text) = &self.pending_lan_port_kill_prompt else {
            return div();
        };
        let info = lan_service_info();

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
                            this.pending_lan_port_kill_prompt = None;
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
                                    .font_weight(openframe::FontWeight::BOLD)
                                    .text_color(if is_dark { rgb(0xf9fafb) } else { rgb(0x111827) })
                                    .child("Kill Existing?"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(if is_dark { rgb(0xd1d5db) } else { rgb(0x374151) })
                                    .child(format!(
                                        "LAN start failed on UDP {}: {}",
                                        info.port, error_text
                                    )),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(if is_dark { rgb(0x9ca3af) } else { rgb(0x6b7280) })
                                    .child("Arcadia will terminate older Arcadia process using this port, then retry Start."),
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
                                            .rounded_md()
                                            .cursor_pointer()
                                            .bg(if is_dark { rgb(0x374151) } else { rgb(0xe5e7eb) })
                                            .text_color(if is_dark { rgb(0xf3f4f6) } else { rgb(0x1f2937) })
                                            .child("Cancel")
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(|this, _, _, cx| {
                                                    this.pending_lan_port_kill_prompt = None;
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
                                                cx.listener(|this, _, _, cx| {
                                                    this.pending_lan_port_kill_prompt = None;
                                                    match this.kill_existing_lan_port_owner_and_retry() {
                                                        Ok(()) => {}
                                                        Err(err) => {
                                                            this.lan_service_feedback = err;
                                                        }
                                                    }
                                                    cx.notify();
                                                }),
                                            ),
                                    ),
                            ),
                    ),
            )
    }

    fn status_badge(&self, enabled: bool, is_dark: bool) -> impl IntoElement {
        let _ = is_dark;
        div()
            .px_2()
            .py_0p5()
            .rounded_full()
            .text_xs()
            .font_weight(openframe::FontWeight::SEMIBOLD)
            .bg(if enabled {
                rgb(0x166534)
            } else {
                rgb(0x374151)
            })
            .text_color(if enabled {
                rgb(0x86efac)
            } else {
                rgb(0x9ca3af)
            })
            .child(if enabled { "enabled" } else { "disabled" })
    }

    fn service_running_badge(&self, running: bool, is_dark: bool) -> impl IntoElement {
        let _ = is_dark;
        div()
            .px_2()
            .py_0p5()
            .rounded_full()
            .text_xs()
            .font_weight(openframe::FontWeight::SEMIBOLD)
            .bg(if running {
                rgb(0x166534)
            } else {
                rgb(0x7f1d1d)
            })
            .text_color(if running {
                rgb(0x86efac)
            } else {
                rgb(0xfca5a5)
            })
            .child(if running { "running" } else { "stopped" })
    }
}
