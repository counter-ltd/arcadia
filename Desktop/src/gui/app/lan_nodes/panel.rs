use arcadia_core::modules::lan::{discover_lan_peers, list_known_lan_peers};
use arcadia_core::modules::{execute_command, ExecutionContext};
use openframe::{
    div, Context, InteractiveElement, IntoElement, MouseButton, ParentElement, Styled,
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
        let glyph = theme::glyph_snapshot(cx);
        let text_c    = glyph.as_ref().map(|g| g.text).unwrap_or_else(|| theme::module_title_text(is_dark));
        let subtext_c = glyph.as_ref().map(|g| g.dim).unwrap_or_else(|| theme::module_meta_text(is_dark));
        let desc_c    = glyph.as_ref().map(|g| g.dim).unwrap_or_else(|| theme::module_description_text(is_dark));
        let bg_c      = glyph.as_ref().map(|g| g.surface).unwrap_or_else(|| theme::module_panel_bg(is_dark));
        let border_c  = glyph.as_ref().map(|g| g.border).unwrap_or_else(|| theme::module_panel_stroke(is_dark));
        let row_bg    = glyph.as_ref().map(|g| g.surface2).unwrap_or_else(|| theme::module_row_bg(is_dark));
        let row_str   = glyph.as_ref().map(|g| g.border).unwrap_or_else(|| theme::module_row_stroke(is_dark));
        let radius    = glyph.as_ref().map(|g| g.border_radius).unwrap_or(8.0);
        // Keep feedback tuple for the output box
        let glyph_feedback = glyph.as_ref().map(|g| (g.surface, g.border, g.dim));
        let known = list_known_lan_peers();
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_4()
            .child(self.lan_nodes_toolbar(cx, is_dark, text_c, row_bg, row_str, radius))
            .child(
                div()
                    .text_sm()
                    .text_color(subtext_c)
                    .child("Broadcast scan matches lan.scan with no args. For CIDR/IP-specific discovery use the shell: lan.scan --range …"),
            )
            .child(self.lan_section_title("Discovered (scan)", text_c))
            .child(if self.lan_discovered_peers.is_empty() {
                div()
                    .text_sm()
                    .text_color(desc_c)
                    .child("No scan results yet — run Scan.")
            } else {
                div().flex().flex_col().gap_2().children(
                    self.lan_discovered_peers
                        .iter()
                        .map(|(ip, hostname)| self.lan_discovered_row(cx, ip, hostname, is_dark, text_c, subtext_c, bg_c, border_c, row_bg, row_str, radius)),
                )
            })
            .child(self.lan_section_title("Known nodes", text_c))
            .child(if known.is_empty() {
                div()
                    .text_sm()
                    .text_color(desc_c)
                    .child("No peers in node state yet — pair from discovery or wait for inbound.")
            } else {
                div().flex().flex_col().gap_2().children(
                    known
                        .into_iter()
                        .map(|peer| self.lan_known_row(cx, peer.ip, peer.hostname, peer.status, is_dark, text_c, subtext_c, bg_c, border_c, row_bg, row_str, radius)),
                )
            })
            .child(
                div()
                    .w_full()
                    .mt_2()
                    .p_3()
                    .rounded(if glyph_feedback.is_some() { openframe::px(0.) } else { openframe::px(radius) })
                    .bg(bg_c)
                    .border_1()
                    .border_color(border_c)
                    .text_sm()
                    .text_color(desc_c)
                    .child(if self.lan_command_feedback.is_empty() {
                        "Command output appears here.".to_string()
                    } else {
                        self.lan_command_feedback.clone()
                    }),
            )
    }

    fn lan_section_title(&self, label: &'static str, text_c: openframe::Rgba) -> impl IntoElement {
        div()
            .text_base()
            .font_weight(openframe::FontWeight::SEMIBOLD)
            .text_color(text_c)
            .child(label)
    }

    fn lan_nodes_toolbar(&self, cx: &mut Context<Self>, is_dark: bool, text_c: openframe::Rgba, row_bg: openframe::Rgba, row_str: openframe::Rgba, radius: f32) -> impl IntoElement {
        div()
            .flex()
            .gap_2()
            .child(self.lan_primary_button(cx, "Refresh", is_dark, text_c, row_bg, row_str, radius, |this, cx| {
                this.lan_command_feedback = "LAN nodes status refreshed.".to_string();
                cx.notify();
            }))
            .child(self.lan_primary_button(cx, "Scan", is_dark, text_c, row_bg, row_str, radius, |this, cx| {
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
                self.lan_primary_button(cx, "Save connected (all)", is_dark, text_c, row_bg, row_str, radius, |this, cx| {
                    this.lan_execute_feedback("lan.node", vec!["save".into()]);
                    cx.notify();
                }),
            )
    }

    #[allow(clippy::too_many_arguments)]
    fn lan_primary_button(
        &self,
        cx: &mut Context<Self>,
        label: &'static str,
        _is_dark: bool,
        text_c: openframe::Rgba,
        row_bg: openframe::Rgba,
        row_str: openframe::Rgba,
        radius: f32,
        on_click: fn(&mut ArcadiaRoot, &mut Context<Self>),
    ) -> impl IntoElement {
        div()
            .cursor_pointer()
            .px_3()
            .py_1()
            .rounded(openframe::px(radius.min(6.0)))
            .bg(row_bg)
            .border_1()
            .border_color(row_str)
            .text_sm()
            .font_weight(openframe::FontWeight::SEMIBOLD)
            .text_color(text_c)
            .child(label)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    on_click(this, cx);
                }),
            )
    }

    #[allow(clippy::too_many_arguments)]
    fn lan_discovered_row(
        &self,
        cx: &mut Context<Self>,
        ip: &str,
        hostname: &str,
        _is_dark: bool,
        text_c: openframe::Rgba,
        subtext_c: openframe::Rgba,
        bg_c: openframe::Rgba,
        border_c: openframe::Rgba,
        row_bg: openframe::Rgba,
        row_str: openframe::Rgba,
        radius: f32,
    ) -> impl IntoElement {
        let ip_btn = ip.to_string();
        div()
            .w_full()
            .px_3()
            .py_2()
            .rounded(openframe::px(radius))
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
                    .child(
                        div()
                            .text_sm()
                            .font_weight(openframe::FontWeight::SEMIBOLD)
                            .text_color(text_c)
                            .child(hostname.to_string()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(subtext_c)
                            .child(ip.to_string()),
                    ),
            )
            .child(
                div().flex().gap_2().child(
                    div()
                        .cursor_pointer()
                        .px_2()
                        .py_1()
                        .rounded(openframe::px(radius.min(6.0)))
                        .bg(row_bg)
                        .border_1()
                        .border_color(row_str)
                        .text_xs()
                        .font_weight(openframe::FontWeight::SEMIBOLD)
                        .text_color(text_c)
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

    #[allow(clippy::too_many_arguments)]
    fn lan_known_row(
        &self,
        cx: &mut Context<Self>,
        ip: String,
        hostname: String,
        status: &'static str,
        _is_dark: bool,
        text_c: openframe::Rgba,
        subtext_c: openframe::Rgba,
        bg_c: openframe::Rgba,
        border_c: openframe::Rgba,
        row_bg: openframe::Rgba,
        row_str: openframe::Rgba,
        radius: f32,
    ) -> impl IntoElement {
        let actions = match status {
            "pending-inbound" => {
                let ip_a = ip.clone();
                let ip_r = ip.clone();
                div()
                    .flex()
                    .gap_2()
                    .child(
                        self.lan_small_button(cx, "Accept", text_c, row_bg, row_str, radius, ip_a, |this, ip, cx| {
                            this.lan_execute_feedback("lan.node", vec!["accept".into(), ip]);
                            cx.notify();
                        }),
                    )
                    .child(
                        self.lan_small_button(cx, "Reject", text_c, row_bg, row_str, radius, ip_r, |this, ip, cx| {
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
                        self.lan_small_button(cx, "Connect", text_c, row_bg, row_str, radius, ip_c, |this, ip, cx| {
                            this.lan_execute_feedback("lan.node", vec!["connect".into(), ip]);
                            cx.notify();
                        }),
                    )
                    .child(
                        self.lan_small_button(cx, "Accept", text_c, row_bg, row_str, radius, ip_a, |this, ip, cx| {
                            this.lan_execute_feedback("lan.node", vec!["accept".into(), ip]);
                            cx.notify();
                        }),
                    )
                    .child(
                        self.lan_small_button(cx, "Reject", text_c, row_bg, row_str, radius, ip_r, |this, ip, cx| {
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
                    text_c,
                    row_bg,
                    row_str,
                    radius,
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
                    text_c,
                    row_bg,
                    row_str,
                    radius,
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
            .rounded(openframe::px(radius))
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
                    .child(
                        div()
                            .text_sm()
                            .font_weight(openframe::FontWeight::SEMIBOLD)
                            .text_color(text_c)
                            .child(hostname),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(subtext_c)
                            .child(format!("{ip} · {status}")),
                    ),
            )
            .child(actions)
    }

    #[allow(clippy::too_many_arguments)]
    fn lan_small_button(
        &self,
        cx: &mut Context<Self>,
        label: &'static str,
        text_c: openframe::Rgba,
        row_bg: openframe::Rgba,
        row_str: openframe::Rgba,
        radius: f32,
        ip: String,
        on_click: fn(&mut ArcadiaRoot, String, &mut Context<Self>),
    ) -> impl IntoElement {
        div()
            .cursor_pointer()
            .px_2()
            .py_1()
            .rounded(openframe::px(radius.min(6.0)))
            .bg(row_bg)
            .border_1()
            .border_color(row_str)
            .text_xs()
            .font_weight(openframe::FontWeight::SEMIBOLD)
            .text_color(text_c)
            .child(label)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    on_click(this, ip.clone(), cx);
                }),
            )
    }
}
