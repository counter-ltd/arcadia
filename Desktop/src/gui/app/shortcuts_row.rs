use arcadia_core::config::shortcuts::{
    ShortcutsConfig, has_system_wide_consent, record_system_wide_consent,
    revoke_system_wide_consent,
};
use arcadia_core::config::ConfigFile;
use arcadia_core::shortcuts::{EffectiveMergedShortcut, KeyChordSpec, ShortcutTrigger};
use openframe::prelude::FluentBuilder as _;
use openframe::{
    AnyElement, Context, FocusHandle, FontWeight, InteractiveElement, IntoElement, MouseButton,
    ParentElement, Styled, div, px,
};

use crate::gui::app::ArcadiaRoot;
use crate::gui::app::shortcuts::sync_os_global_hotkeys;
use crate::gui::theme;
use crate::gui::theme::palette::ThemePalette;

impl ArcadiaRoot {
    pub fn shortcut_row_item(
        cx: &mut Context<Self>,
        sc: &EffectiveMergedShortcut,
        is_dark: bool,
        override_chord: Option<KeyChordSpec>,
        override_sequence: Option<Vec<KeyChordSpec>>,
        // Some((steps_captured, total_steps)) when recording a sequence for this shortcut.
        seq_listening: Option<(usize, usize)>,
        listening: bool,
        listen_focus: FocusHandle,
    ) -> AnyElement {
        let p = theme::theme_palette(cx, is_dark);
        let g_snap = theme::glyph_snapshot(cx);
        let radius = g_snap.map(|g| g.border_radius).unwrap_or(p.radius_md);

        let id = sc.id.clone();
        let label = sc.label.clone();
        let owner = sc.owner.clone();
        let sw = sc.system_wide;
        let triggers = sc.triggers.clone();

        let has_chord_trigger = triggers.iter().any(|t| matches!(t, ShortcutTrigger::Chord(_)));
        let has_seq_trigger = triggers.iter().any(|t| matches!(t, ShortcutTrigger::Sequence(_)));
        let has_chord_override = override_chord.is_some();
        let has_seq_override = override_sequence.is_some();
        let consented = has_system_wide_consent(&id);

        let is_custom = owner == "user";
        let id_delete = id.clone();
        let id_listen = id.clone();
        let id_seq_start = id.clone();
        let id_reset_chord = id.clone();
        let id_reset_seq = id.clone();
        let id_consent = id.clone();

        let chip_radius = radius.min(6.0);
        let badge_radius = radius.min(8.0);

        // Build keybind control area based on trigger type and listening state.
        let keybind_area: AnyElement = if listening {
            // Chord capture in progress.
            div()
                .flex()
                .items_center()
                .px_3()
                .py_1()
                .rounded(px(badge_radius))
                .bg(p.accent)
                .track_focus(&listen_focus)
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(p.on_accent)
                        .child("Press a key…"),
                )
                .into_any_element()
        } else if let Some((captured, total)) = seq_listening {
            // Sequence capture in progress — show step counter.
            div()
                .flex()
                .items_center()
                .px_3()
                .py_1()
                .rounded(px(badge_radius))
                .bg(p.accent)
                .track_focus(&listen_focus)
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(p.on_accent)
                        .child(format!("Step {} of {} — Press a key…", captured + 1, total)),
                )
                .into_any_element()
        } else if has_chord_trigger {
            let chord_to_show: Option<KeyChordSpec> = override_chord.clone().or_else(|| {
                triggers.iter().find_map(|t| {
                    if let ShortcutTrigger::Chord(c) = t {
                        Some(c.clone())
                    } else {
                        None
                    }
                })
            });
            let chips = chord_to_show.map(|c| chord_chips(&c)).unwrap_or_default();

            div()
                .flex()
                .items_center()
                .gap_1()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .cursor_pointer()
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, window, cx| {
                                this.shortcut_listening_id = Some(id_listen.clone());
                                this.shortcut_listen_focus.focus(window);
                                cx.notify();
                            }),
                        )
                        .children(chips.into_iter().map(|chip| {
                            div()
                                .px_2()
                                .py_0p5()
                                .rounded(px(chip_radius))
                                .bg(p.surface_elevated)
                                .border_1()
                                .border_color(p.border)
                                .text_xs()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(p.content_title)
                                .child(chip)
                                .into_any_element()
                        })),
                )
                .when(has_chord_override, |d| {
                    d.child(
                        div()
                            .px_2()
                            .py_0p5()
                            .rounded(px(chip_radius))
                            .text_xs()
                            .text_color(p.ui_subtext)
                            .cursor_pointer()
                            .child("×")
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |_, _, _, cx| {
                                    if let Ok(mut cfg) = ShortcutsConfig::load_or_create() {
                                        if let Some(entry) =
                                            cfg.overrides.get_mut(&id_reset_chord)
                                        {
                                            entry.chord = None;
                                        }
                                        let _ = cfg.save();
                                    }
                                    sync_os_global_hotkeys();
                                    cx.notify();
                                }),
                            ),
                    )
                })
                .into_any_element()
        } else if has_seq_trigger {
            // Sequence trigger — show step chips with → separators + click to re-record.
            let seq_to_show: Option<Vec<KeyChordSpec>> = override_sequence.clone().or_else(|| {
                triggers.iter().find_map(|t| {
                    if let ShortcutTrigger::Sequence(s) = t {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
            });
            let total_steps = seq_to_show.as_ref().map(|s| s.len()).unwrap_or(0);

            div()
                .flex()
                .items_center()
                .gap_1()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .cursor_pointer()
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, window, cx| {
                                this.shortcut_listening_sequence =
                                    Some((id_seq_start.clone(), Vec::new(), total_steps));
                                this.shortcut_listen_focus.focus(window);
                                cx.notify();
                            }),
                        )
                        .children(
                            sequence_step_elements(
                                seq_to_show.as_deref().unwrap_or(&[]),
                                chip_radius,
                                p,
                            ),
                        ),
                )
                .when(has_seq_override, |d| {
                    d.child(
                        div()
                            .px_2()
                            .py_0p5()
                            .rounded(px(chip_radius))
                            .text_xs()
                            .text_color(p.ui_subtext)
                            .cursor_pointer()
                            .child("×")
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |_, _, _, cx| {
                                    if let Ok(mut cfg) = ShortcutsConfig::load_or_create() {
                                        if let Some(entry) =
                                            cfg.overrides.get_mut(&id_reset_seq)
                                        {
                                            entry.sequence = None;
                                        }
                                        let _ = cfg.save();
                                    }
                                    sync_os_global_hotkeys();
                                    cx.notify();
                                }),
                            ),
                    )
                })
                .into_any_element()
        } else {
            // Edge/HotCorner or unknown — non-editable badge.
            div()
                .px_2()
                .py_0p5()
                .rounded(px(badge_radius))
                .bg(p.surface_elevated)
                .border_1()
                .border_color(p.border)
                .text_xs()
                .text_color(p.ui_subtext)
                .child(trigger_type_label(&triggers))
                .into_any_element()
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
                    .gap_1()
                    .child(
                        div()
                            .text_base()
                            .font_weight(FontWeight::BOLD)
                            .text_color(p.content_title)
                            .child(label),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(p.content_meta)
                            .child(format!("{id} · {owner}")),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_end()
                    .gap_2()
                    .child(keybind_area)
                    .when(is_custom, |d| {
                        d.child(
                            div()
                                .px_2()
                                .py_0p5()
                                .rounded(px(chip_radius))
                                .text_xs()
                                .text_color(p.danger)
                                .cursor_pointer()
                                .child("Delete")
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |_, _, _, cx| {
                                        if let Ok(mut cfg) = ShortcutsConfig::load_or_create() {
                                            cfg.custom.retain(|c| c.id != id_delete);
                                            let _ = cfg.save();
                                        }
                                        sync_os_global_hotkeys();
                                        cx.notify();
                                    }),
                                ),
                        )
                    })
                    .when(sw, |d| {
                        d.child(
                            div()
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
                                        .text_color(if consented {
                                            p.accent
                                        } else {
                                            p.ui_subtext
                                        })
                                        .cursor_pointer()
                                        .child(if consented {
                                            "Consented — tap to revoke"
                                        } else {
                                            "Tap to consent"
                                        })
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(move |_, _, _, cx| {
                                                let r = if has_system_wide_consent(&id_consent) {
                                                    revoke_system_wide_consent(&id_consent)
                                                } else {
                                                    record_system_wide_consent(&id_consent)
                                                };
                                                let _ = r;
                                                sync_os_global_hotkeys();
                                                cx.notify();
                                            }),
                                        ),
                                ),
                        )
                    }),
            );

        if let Some(g) = theme::active_glyph(cx) {
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
}

/// Renders sequence steps as chip groups separated by → arrows.
fn sequence_step_elements(
    steps: &[KeyChordSpec],
    chip_radius: f32,
    p: ThemePalette,
) -> Vec<AnyElement> {
    let mut els: Vec<AnyElement> = Vec::new();
    for (i, step) in steps.iter().enumerate() {
        if i > 0 {
            els.push(
                div()
                    .text_xs()
                    .text_color(p.ui_subtext)
                    .child("→")
                    .into_any_element(),
            );
        }
        for chip in chord_chips(step) {
            els.push(
                div()
                    .px_2()
                    .py_0p5()
                    .rounded(px(chip_radius))
                    .bg(p.surface_elevated)
                    .border_1()
                    .border_color(p.border)
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(p.content_title)
                    .child(chip)
                    .into_any_element(),
            );
        }
    }
    els
}

fn chord_chips(chord: &KeyChordSpec) -> Vec<String> {
    let mut chips = Vec::new();
    if chord.platform {
        chips.push("Cmd".to_string());
    }
    if chord.control {
        chips.push("Ctrl".to_string());
    }
    if chord.alt {
        chips.push("Alt".to_string());
    }
    if chord.shift {
        chips.push("Shift".to_string());
    }
    if chord.function {
        chips.push("Fn".to_string());
    }
    chips.push(key_display(&chord.key));
    chips
}

fn key_display(key: &str) -> String {
    match key {
        "escape" => "Esc".to_string(),
        "tab" => "Tab".to_string(),
        "space" => "Space".to_string(),
        "enter" => "Return".to_string(),
        "backspace" => "⌫".to_string(),
        "delete" => "Del".to_string(),
        "up" => "↑".to_string(),
        "down" => "↓".to_string(),
        "left" => "←".to_string(),
        "right" => "→".to_string(),
        k if k.len() == 1 => k.to_ascii_uppercase(),
        k => {
            let mut s = k.to_string();
            if let Some(first) = s.get_mut(0..1) {
                first.make_ascii_uppercase();
            }
            s
        }
    }
}

fn trigger_type_label(triggers: &[ShortcutTrigger]) -> String {
    for t in triggers {
        match t {
            ShortcutTrigger::Sequence(seq) => return format!("seq({} steps)", seq.len()),
            ShortcutTrigger::EdgeSwipe { edge, .. } => return format!("edge-{edge:?}"),
            ShortcutTrigger::HotCorner { quadrant, .. } => return format!("corner-{quadrant:?}"),
            _ => {}
        }
    }
    "(none)".to_string()
}
