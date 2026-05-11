use arcadia_core::config::shortcuts::{
    has_system_wide_consent, record_system_wide_consent, revoke_system_wide_consent,
};
use arcadia_core::shortcuts::{self, EffectiveMergedShortcut, ShortcutTrigger};
use openframe::{
    AnyElement, Context, FontWeight, InteractiveElement, IntoElement, MouseButton, ParentElement,
    Styled, Window, div, px,
};
use openframe::prelude::FluentBuilder as _;

use super::shortcuts::{self as app_shortcuts, global_shortcuts_for_settings};
use super::ArcadiaRoot;
use crate::gui::theme;

impl ArcadiaRoot {
    pub fn shortcuts_panel(
        &mut self,
        _window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> AnyElement {
        if self.active_page_id.as_str() != "global.shortcuts" {
            return div().into_any_element();
        }
        let p = theme::theme_palette(cx, is_dark);
        let g_snap = theme::glyph_snapshot(cx);
        let panel_radius = g_snap.map(|g| g.border_radius).unwrap_or(p.radius_md);

        let conflicts = shortcuts::chord_conflicts();
        let mut conflict_lines = Vec::new();
        for c in &conflicts {
            conflict_lines.push(format!(
                "{} → {}",
                c.fingerprint,
                c.shortcut_ids.join(", ")
            ));
        }

        let list = global_shortcuts_for_settings();
        let mut rows: Vec<AnyElement> = Vec::new();

        for sc in list {
            let id = sc.id.clone();
            let label = sc.label.clone();
            let trig = trigger_summary(&sc);
            let sw = sc.system_wide;
            let consented = has_system_wide_consent(&id);
            let id_consent = id.clone();
            rows.push(
                div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .py_3()
                    .border_b_1()
                    .border_color(p.border)
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(p.content_title)
                            .child(label),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(p.content_meta)
                            .child(format!("{id} · {trig}")),
                    )
                    .when(sw, |d| {
                        d.child(
                            div()
                                .mt_1()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(p.ui_subtext)
                                        .child("OS-global"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(if consented { p.accent } else { p.ui_subtext })
                                        .cursor_pointer()
                                        .child(if consented {
                                            "Consented — tap to revoke"
                                        } else {
                                            "Tap to consent and register"
                                        })
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(move |_this, _, _, cx| {
                                                let r = if has_system_wide_consent(&id_consent) {
                                                    revoke_system_wide_consent(&id_consent)
                                                } else {
                                                    record_system_wide_consent(&id_consent)
                                                };
                                                let _ = r;
                                                app_shortcuts::sync_os_global_hotkeys();
                                                cx.notify();
                                            }),
                                        ),
                                ),
                        )
                    })
                    .into_any_element(),
            );
        }

        div()
            .w_full()
            .max_w(px(theme::GLYPH_PANEL_CONTENT_MAX_W_PX))
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::BOLD)
                    .text_color(p.content_title)
                    .child("Shortcuts"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(p.content_meta)
                    .child("Registry-driven shortcuts from Arcadia core and extensions. Pointer hot-corners and edge swipes can be disabled in shortcuts.toml."),
            )
            .when(!conflict_lines.is_empty(), |d| {
                d.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .p_3()
                        .rounded(px(panel_radius))
                        .bg(p.badge_info_bg)
                        .border_1()
                        .border_color(p.border)
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(p.badge_info_fg)
                                .child("Chord conflicts"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(p.badge_info_fg)
                                .children(
                                    conflict_lines
                                        .into_iter()
                                        .map(|line| div().child(line).into_any_element()),
                                ),
                        ),
                )
            })
            .child(div().flex().flex_col().children(rows))
            .into_any_element()
    }
}

fn trigger_summary(sc: &EffectiveMergedShortcut) -> String {
    let mut parts = Vec::new();
    for t in &sc.triggers {
        match t {
            ShortcutTrigger::Chord(c) => parts.push(format!(
                "{}+{}",
                mod_summary(c),
                c.key
            )),
            ShortcutTrigger::Sequence(seq) => parts.push(format!("seq({} steps)", seq.len())),
            ShortcutTrigger::EdgeSwipe { edge, .. } => {
                parts.push(format!("edge-{edge:?}"))
            }
            ShortcutTrigger::HotCorner { quadrant, .. } => {
                parts.push(format!("corner-{quadrant:?}"))
            }
        }
    }
    if parts.is_empty() {
        "(none)".into()
    } else {
        parts.join(" | ")
    }
}

fn mod_summary(c: &arcadia_core::shortcuts::KeyChordSpec) -> String {
    let mut s = Vec::new();
    if c.platform {
        s.push("Cmd");
    }
    if c.control {
        s.push("Ctrl");
    }
    if c.alt {
        s.push("Alt");
    }
    if c.shift {
        s.push("Shift");
    }
    if c.function {
        s.push("Fn");
    }
    if s.is_empty() {
        "plain".into()
    } else {
        s.join("+")
    }
}
