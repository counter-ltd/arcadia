use arcadia_core::modules::lan::{discover_lan_peers, list_known_lan_peers};
use arcadia_core::modules::{execute_command, ExecutionContext};
use openframe::{
    div, rgb, Context, InteractiveElement, IntoElement, MouseButton, ParentElement, Styled,
};

use crate::gui::app::ArcadiaRoot;
use crate::gui::theme;

impl ArcadiaRoot {
    pub(crate) fn lan_execute_feedback(&mut self, token: &str, args: Vec<String>) {
        let slices: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        self.lan_command_feedback =
            match execute_command(token, &slices, &ExecutionContext::default()) {
                Ok(Some(message)) => message,
                Ok(None) => format!("Unknown command: {token}"),
                Err(err) => err,
            };
    }

    pub fn lan_nodes_panel(&self, cx: &mut Context<Self>, is_dark: bool) -> impl IntoElement {
        let known = list_known_lan_peers();
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_4()
            .child(self.lan_nodes_toolbar(cx, is_dark))
            .child(
                div()
                    .text_sm()
                    .text_color(theme::module_meta_text(is_dark))
                    .child("Broadcast scan matches lan.scan with no args. For CIDR/IP-specific discovery use the shell: lan.scan --range …"),
            )
            .child(self.lan_section_title("Discovered (scan)", is_dark))
            .child(if self.lan_discovered_peers.is_empty() {
                div()
                    .text_sm()
                    .text_color(theme::module_description_text(is_dark))
                    .child("No scan results yet — run Scan.")
            } else {
                div().flex().flex_col().gap_2().children(
                    self.lan_discovered_peers
                        .iter()
                        .map(|(ip, hostname)| self.lan_discovered_row(cx, ip, hostname, is_dark)),
                )
            })
            .child(self.lan_section_title("Known nodes", is_dark))
            .child(if known.is_empty() {
                div()
                    .text_sm()
                    .text_color(theme::module_description_text(is_dark))
                    .child("No peers in node state yet — pair from discovery or wait for inbound.")
            } else {
                div().flex().flex_col().gap_2().children(
                    known
                        .into_iter()
                        .map(|peer| self.lan_known_row(cx, peer.ip, peer.hostname, peer.status, is_dark)),
                )
            })
            .child(
                div()
                    .w_full()
                    .mt_2()
                    .p_3()
                    .rounded_lg()
                    .bg(theme::module_panel_bg(is_dark))
                    .border_1()
                    .border_color(theme::module_panel_stroke(is_dark))
                    .text_sm()
                    .text_color(theme::module_description_text(is_dark))
                    .child(if self.lan_command_feedback.is_empty() {
                        "Command output appears here.".to_string()
                    } else {
                        self.lan_command_feedback.clone()
                    }),
            )
    }

    fn lan_section_title(&self, label: &'static str, is_dark: bool) -> impl IntoElement {
        div()
            .text_base()
            .font_weight(openframe::FontWeight::SEMIBOLD)
            .text_color(theme::module_title_text(is_dark))
            .child(label)
    }

    fn lan_nodes_toolbar(&self, cx: &mut Context<Self>, is_dark: bool) -> impl IntoElement {
        div()
            .flex()
            .gap_2()
            .child(self.lan_primary_button(cx, "Refresh", is_dark, |this, cx| {
                this.lan_command_feedback = "LAN nodes status refreshed.".to_string();
                cx.notify();
            }))
            .child(self.lan_primary_button(cx, "Scan", is_dark, |this, cx| {
                match discover_lan_peers(None) {
                    Ok(peers) => {
                        let n = peers.len();
                        this.lan_discovered_peers = peers;
                        this.lan_command_feedback = format!("Scan finished — {n} peer(s).");
                    }
                    Err(err) => {
                        this.lan_discovered_peers.clear();
                        this.lan_command_feedback = err;
                    }
                }
                cx.notify();
            }))
            .child(
                self.lan_primary_button(cx, "Save connected (all)", is_dark, |this, cx| {
                    this.lan_execute_feedback("lan.node", vec!["save".into()]);
                    cx.notify();
                }),
            )
    }

    fn lan_primary_button(
        &self,
        cx: &mut Context<Self>,
        label: &'static str,
        is_dark: bool,
        on_click: fn(&mut ArcadiaRoot, &mut Context<Self>),
    ) -> impl IntoElement {
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
            .child(label)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    on_click(this, cx);
                }),
            )
    }

    fn lan_discovered_row(
        &self,
        cx: &mut Context<Self>,
        ip: &str,
        hostname: &str,
        is_dark: bool,
    ) -> impl IntoElement {
        let ip_btn = ip.to_string();
        div()
            .w_full()
            .px_3()
            .py_2()
            .rounded_lg()
            .bg(theme::module_panel_bg(is_dark))
            .border_1()
            .border_color(theme::module_panel_stroke(is_dark))
            .flex()
            .items_center()
            .justify_between()
            .gap_3()
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
                            .child(hostname.to_string()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::module_meta_text(is_dark))
                            .child(ip.to_string()),
                    ),
            )
            .child(
                div().flex().gap_2().child(
                    div()
                        .cursor_pointer()
                        .px_2()
                        .py_1()
                        .rounded_md()
                        .bg(if is_dark {
                            rgb(0x1f2937)
                        } else {
                            rgb(0xeef2ff)
                        })
                        .text_xs()
                        .font_weight(openframe::FontWeight::SEMIBOLD)
                        .text_color(theme::module_title_text(is_dark))
                        .child("Pair")
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, _, cx| {
                                this.lan_execute_feedback(
                                    "lan.node",
                                    vec!["pair".into(), ip_btn.clone()],
                                );
                                cx.notify();
                            }),
                        ),
                ),
            )
    }

    fn lan_known_row(
        &self,
        cx: &mut Context<Self>,
        ip: String,
        hostname: String,
        status: &'static str,
        is_dark: bool,
    ) -> impl IntoElement {
        let actions = match status {
            "pending-inbound" => {
                let ip_a = ip.clone();
                let ip_r = ip.clone();
                div()
                    .flex()
                    .gap_2()
                    .child(
                        self.lan_small_button(cx, "Accept", is_dark, ip_a, |this, ip, cx| {
                            this.lan_execute_feedback("lan.node", vec!["accept".into(), ip]);
                            cx.notify();
                        }),
                    )
                    .child(
                        self.lan_small_button(cx, "Reject", is_dark, ip_r, |this, ip, cx| {
                            this.lan_execute_feedback("lan.node", vec!["reject".into(), ip]);
                            cx.notify();
                        }),
                    )
            }
            "pending-outbound" => {
                let ip_c = ip.clone();
                let ip_a = ip.clone();
                let ip_r = ip.clone();
                div()
                    .flex()
                    .gap_2()
                    .child(
                        self.lan_small_button(cx, "Connect", is_dark, ip_c, |this, ip, cx| {
                            this.lan_execute_feedback("lan.node", vec!["connect".into(), ip]);
                            cx.notify();
                        }),
                    )
                    .child(
                        self.lan_small_button(cx, "Accept", is_dark, ip_a, |this, ip, cx| {
                            this.lan_execute_feedback("lan.node", vec!["accept".into(), ip]);
                            cx.notify();
                        }),
                    )
                    .child(
                        self.lan_small_button(cx, "Reject", is_dark, ip_r, |this, ip, cx| {
                            this.lan_execute_feedback("lan.node", vec!["reject".into(), ip]);
                            cx.notify();
                        }),
                    )
            }
            "connected" => {
                let ip_s = ip.clone();
                div().flex().gap_2().child(self.lan_small_button(
                    cx,
                    "Save",
                    is_dark,
                    ip_s,
                    |this, ip, cx| {
                        this.lan_execute_feedback("lan.node", vec!["save".into(), ip]);
                        cx.notify();
                    },
                ))
            }
            _ => {
                let ip_p = ip.clone();
                div().flex().gap_2().child(self.lan_small_button(
                    cx,
                    "Pair again",
                    is_dark,
                    ip_p,
                    |this, ip, cx| {
                        this.lan_execute_feedback("lan.node", vec!["pair".into(), ip]);
                        cx.notify();
                    },
                ))
            }
        };

        div()
            .w_full()
            .px_3()
            .py_2()
            .rounded_lg()
            .bg(theme::module_panel_bg(is_dark))
            .border_1()
            .border_color(theme::module_panel_stroke(is_dark))
            .flex()
            .items_center()
            .justify_between()
            .gap_3()
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
                            .child(hostname),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::module_meta_text(is_dark))
                            .child(format!("{ip} · {status}")),
                    ),
            )
            .child(actions)
    }

    fn lan_small_button(
        &self,
        cx: &mut Context<Self>,
        label: &'static str,
        is_dark: bool,
        ip: String,
        on_click: fn(&mut ArcadiaRoot, String, &mut Context<Self>),
    ) -> impl IntoElement {
        div()
            .cursor_pointer()
            .px_2()
            .py_1()
            .rounded_md()
            .bg(if is_dark {
                rgb(0x1f2937)
            } else {
                rgb(0xeef2ff)
            })
            .text_xs()
            .font_weight(openframe::FontWeight::SEMIBOLD)
            .text_color(theme::module_title_text(is_dark))
            .child(label)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    on_click(this, ip.clone(), cx);
                }),
            )
    }
}
