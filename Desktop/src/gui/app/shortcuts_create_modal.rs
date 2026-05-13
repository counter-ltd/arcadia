use std::time::{SystemTime, UNIX_EPOCH};

use arcadia_core::config::shortcuts::{CustomShortcut, ShortcutsConfig};
use arcadia_core::config::ConfigFile;
use arcadia_core::navigation;
use arcadia_core::shortcuts::{KeyChordSpec, ShortcutAction, ShortcutTrigger};
use openframe::prelude::FluentBuilder as _;
use openframe::{
    AnyElement, Context, FontWeight, InteractiveElement, IntoElement, KeyDownEvent, MouseButton,
    ParentElement, Styled, Window, div, px, rgb,
};

use super::shortcuts::sync_os_global_hotkeys;
use super::{ArcadiaRoot, ShortcutCreateActionKind, ShortcutCreateTriggerKind};
use crate::gui::app::text_input_caret::text_with_trailing_caret;
use crate::gui::theme;
use crate::gui::theme::palette::ThemePalette;

fn chord_chips_modal(chord: &KeyChordSpec) -> Vec<String> {
    let mut chips = Vec::new();
    if chord.platform { chips.push("Cmd".to_string()); }
    if chord.control  { chips.push("Ctrl".to_string()); }
    if chord.alt      { chips.push("Alt".to_string()); }
    if chord.shift    { chips.push("Shift".to_string()); }
    if chord.function { chips.push("Fn".to_string()); }
    chips.push(key_display_modal(&chord.key));
    chips
}

fn key_display_modal(key: &str) -> String {
    match key {
        "escape"    => "Esc".to_string(),
        "tab"       => "Tab".to_string(),
        "space"     => "Space".to_string(),
        "enter"     => "Return".to_string(),
        "backspace" => "⌫".to_string(),
        "delete"    => "Del".to_string(),
        "up"        => "↑".to_string(),
        "down"      => "↓".to_string(),
        "left"      => "←".to_string(),
        "right"     => "→".to_string(),
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

fn chip_el(label: impl Into<String>, chip_radius: f32, p: ThemePalette) -> AnyElement {
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
        .child(label.into())
        .into_any_element()
}

fn toggle_btn(
    label: &'static str,
    selected: bool,
    p: ThemePalette,
    radius: f32,
    on_click: impl Fn(&mut ArcadiaRoot, &openframe::MouseDownEvent, &mut openframe::Window, &mut Context<ArcadiaRoot>) + 'static,
    cx: &mut Context<ArcadiaRoot>,
) -> AnyElement {
    div()
        .px_3()
        .py_1()
        .rounded(px(radius.min(8.0)))
        .bg(if selected { p.accent } else { p.surface_elevated })
        .border_1()
        .border_color(if selected { p.accent } else { p.border })
        .text_xs()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(if selected { p.on_accent } else { p.ui_subtext })
        .cursor_pointer()
        .child(label)
        .on_mouse_down(MouseButton::Left, cx.listener(on_click))
        .into_any_element()
}

fn text_field(
    value: &str,
    placeholder: &'static str,
    focused: bool,
    blink: bool,
    focus_handle: openframe::FocusHandle,
    p: ThemePalette,
    radius: f32,
    on_key_down: impl Fn(&mut ArcadiaRoot, &KeyDownEvent, &mut openframe::Window, &mut Context<ArcadiaRoot>) + 'static,
    cx: &mut Context<ArcadiaRoot>,
) -> AnyElement {
    let fh = focus_handle.clone();
    let content: String = if value.is_empty() && !focused {
        placeholder.to_string()
    } else {
        text_with_trailing_caret(value, focused, blink)
    };
    let text_color = if value.is_empty() && !focused { p.ui_subtext } else { p.content_title };
    div()
        .w_full()
        .px_3()
        .py_2()
        .rounded(px(radius.min(8.0)))
        .bg(p.surface_elevated)
        .border_1()
        .border_color(if focused { p.accent } else { p.border })
        .text_sm()
        .text_color(text_color)
        .track_focus(&focus_handle)
        .on_mouse_down(MouseButton::Left, cx.listener(move |_, _, window, _| {
            fh.focus(window);
        }))
        .on_key_down(cx.listener(on_key_down))
        .child(content)
        .into_any_element()
}

impl ArcadiaRoot {
    pub fn shortcut_create_modal(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> AnyElement {
        if self.shortcut_create_draft.is_none() {
            return div().into_any_element();
        }

        let p = theme::theme_palette(cx, is_dark);
        let g = theme::glyph_snapshot(cx);
        let radius = g.map(|gg| gg.border_radius).unwrap_or(p.radius_md);
        let chip_radius = radius.min(6.0);

        let label_focused = self.shortcut_create_label_focus.is_focused(window);
        let token_focused = self.shortcut_create_token_focus.is_focused(window);
        let args_focused = self.shortcut_create_args_focus.is_focused(window);
        let blink = self.text_caret_blink_visible;

        let label_fh = self.shortcut_create_label_focus.clone();
        let token_fh = self.shortcut_create_token_focus.clone();
        let args_fh = self.shortcut_create_args_focus.clone();
        let listen_fh = self.shortcut_listen_focus.clone();

        let recording_chord = self.shortcut_draft_recording_chord;
        let recording_seq = self.shortcut_draft_recording_seq;

        // Clone draft fields for rendering (avoids partial borrow issues in closures).
        let draft = self.shortcut_create_draft.clone().unwrap();
        let trigger_kind = draft.trigger_kind.clone();
        let action_kind = draft.action_kind.clone();
        let chord = draft.chord.clone();
        let sequence = draft.sequence.clone();
        let seq_total = draft.sequence_total;

        // --- Trigger type toggles ---
        let chord_toggle = toggle_btn(
            "Chord", trigger_kind == ShortcutCreateTriggerKind::Chord, p, radius,
            |this, _, _, cx| {
                if let Some(ref mut d) = this.shortcut_create_draft {
                    d.trigger_kind = ShortcutCreateTriggerKind::Chord;
                    d.error = None;
                }
                cx.notify();
            },
            cx,
        );
        let seq_toggle = toggle_btn(
            "Sequence", trigger_kind == ShortcutCreateTriggerKind::Sequence, p, radius,
            |this, _, _, cx| {
                if let Some(ref mut d) = this.shortcut_create_draft {
                    d.trigger_kind = ShortcutCreateTriggerKind::Sequence;
                    d.error = None;
                }
                cx.notify();
            },
            cx,
        );

        // --- Sequence step counter (shown when Sequence selected) ---
        let seq_counter: Option<AnyElement> = if trigger_kind == ShortcutCreateTriggerKind::Sequence {
            Some(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_xs()
                            .text_color(p.ui_subtext)
                            .child(format!("{seq_total} steps")),
                    )
                    .child(
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
                            .cursor_pointer()
                            .child("+")
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                if let Some(ref mut d) = this.shortcut_create_draft {
                                    d.sequence_total = (d.sequence_total + 1).min(8);
                                    d.sequence.clear();
                                }
                                cx.notify();
                            })),
                    )
                    .child(
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
                            .cursor_pointer()
                            .child("−")
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                if let Some(ref mut d) = this.shortcut_create_draft {
                                    d.sequence_total = (d.sequence_total - 1).max(2);
                                    d.sequence.clear();
                                }
                                cx.notify();
                            })),
                    )
                    .into_any_element(),
            )
        } else {
            None
        };

        // --- Record button / status ---
        let record_area: AnyElement = if recording_chord || recording_seq {
            let msg = if recording_seq {
                let step = sequence.len();
                format!("Step {} of {} — Press a key…", step + 1, seq_total)
            } else {
                "Press a key…".to_string()
            };
            div()
                .flex()
                .items_center()
                .px_3()
                .py_1()
                .rounded(px(radius.min(8.0)))
                .bg(p.accent)
                .track_focus(&listen_fh)
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(p.on_accent)
                        .child(msg),
                )
                .into_any_element()
        } else {
            let (btn_label, recorded_display): (&str, AnyElement) =
                if trigger_kind == ShortcutCreateTriggerKind::Chord {
                    let display = if let Some(ref c) = chord {
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .children(chord_chips_modal(c).into_iter().map(|ch| chip_el(ch, chip_radius, p)))
                            .into_any_element()
                    } else {
                        div()
                            .text_xs()
                            .text_color(p.ui_subtext)
                            .child("—")
                            .into_any_element()
                    };
                    ("Record chord", display)
                } else {
                    let display = if sequence.is_empty() {
                        div()
                            .text_xs()
                            .text_color(p.ui_subtext)
                            .child("—")
                            .into_any_element()
                    } else {
                        let mut els: Vec<AnyElement> = Vec::new();
                        for (i, step) in sequence.iter().enumerate() {
                            if i > 0 {
                                els.push(
                                    div()
                                        .text_xs()
                                        .text_color(p.ui_subtext)
                                        .child("→")
                                        .into_any_element(),
                                );
                            }
                            for ch in chord_chips_modal(step) {
                                els.push(chip_el(ch, chip_radius, p));
                            }
                        }
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .children(els)
                            .into_any_element()
                    };
                    ("Record sequence", display)
                };

            let listen_fh2 = self.shortcut_listen_focus.clone();
            let is_chord = trigger_kind == ShortcutCreateTriggerKind::Chord;
            div()
                .flex()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .px_3()
                        .py_1()
                        .rounded(px(radius.min(8.0)))
                        .bg(p.surface_elevated)
                        .border_1()
                        .border_color(p.border)
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(p.content_title)
                        .cursor_pointer()
                        .child(btn_label)
                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, window, cx| {
                            if let Some(ref mut d) = this.shortcut_create_draft {
                                if is_chord {
                                    this.shortcut_draft_recording_chord = true;
                                } else {
                                    d.sequence.clear();
                                    this.shortcut_draft_recording_seq = true;
                                }
                                listen_fh2.focus(window);
                            }
                            cx.notify();
                        })),
                )
                .child(recorded_display)
                .into_any_element()
        };

        // --- Action type toggles ---
        let navigate_toggle = toggle_btn(
            "Navigate", action_kind == ShortcutCreateActionKind::Navigate, p, radius,
            |this, _, _, cx| {
                if let Some(ref mut d) = this.shortcut_create_draft {
                    d.action_kind = ShortcutCreateActionKind::Navigate;
                    d.error = None;
                }
                cx.notify();
            },
            cx,
        );
        let exec_toggle = toggle_btn(
            "Execute Command", action_kind == ShortcutCreateActionKind::ExecuteCommand, p, radius,
            |this, _, _, cx| {
                if let Some(ref mut d) = this.shortcut_create_draft {
                    d.action_kind = ShortcutCreateActionKind::ExecuteCommand;
                    d.error = None;
                }
                cx.notify();
            },
            cx,
        );

        // --- Action-specific section ---
        let selected_page = draft.action_page_id.clone();
        let action_section: AnyElement = if action_kind == ShortcutCreateActionKind::Navigate {
            let page_chips: Vec<AnyElement> = navigation::PAGE_DEFINITIONS
                .iter()
                .map(|page| {
                    let page_id = page.id.to_string();
                    let selected = page_id == selected_page;
                    let pid2 = page_id.clone();
                    div()
                        .px_2()
                        .py_1()
                        .rounded(px(chip_radius))
                        .bg(if selected { p.accent } else { p.surface_elevated })
                        .border_1()
                        .border_color(if selected { p.accent } else { p.border })
                        .text_xs()
                        .font_weight(if selected { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                        .text_color(if selected { p.on_accent } else { p.content_title })
                        .cursor_pointer()
                        .child(format!("{} ({})", page.title, page.id))
                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                            if let Some(ref mut d) = this.shortcut_create_draft {
                                d.action_page_id = pid2.clone();
                                d.error = None;
                            }
                            cx.notify();
                        }))
                        .into_any_element()
                })
                .collect();
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .text_xs()
                        .text_color(p.ui_subtext)
                        .child("Select a page:"),
                )
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap_1()
                        .children(page_chips),
                )
                .into_any_element()
        } else {
            // Execute Command
            let label_fh_t = token_fh.clone();
            let label_fh_a = args_fh.clone();
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .text_color(p.ui_subtext)
                                .child("Command token"),
                        )
                        .child(text_field(
                            &draft.action_command_token,
                            "e.g. shell.execute",
                            token_focused,
                            blink,
                            label_fh_t,
                            p,
                            radius,
                            |this, event, _, cx| {
                                let key = event.keystroke.key.as_str();
                                let mods = event.keystroke.modifiers;
                                if let Some(ref mut d) = this.shortcut_create_draft {
                                    if key == "backspace" {
                                        d.action_command_token.pop();
                                        cx.notify();
                                    } else if !mods.control && !mods.alt && !mods.platform && !mods.function {
                                        if let Some(kc) = &event.keystroke.key_char {
                                            d.action_command_token.push_str(kc);
                                            cx.notify();
                                        }
                                    }
                                }
                            },
                            cx,
                        )),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .text_color(p.ui_subtext)
                                .child("Args (space-separated, optional)"),
                        )
                        .child(text_field(
                            &draft.action_command_args,
                            "arg1 arg2 …",
                            args_focused,
                            blink,
                            label_fh_a,
                            p,
                            radius,
                            |this, event, _, cx| {
                                let key = event.keystroke.key.as_str();
                                let mods = event.keystroke.modifiers;
                                if let Some(ref mut d) = this.shortcut_create_draft {
                                    if key == "backspace" {
                                        d.action_command_args.pop();
                                        cx.notify();
                                    } else if key == "space" {
                                        d.action_command_args.push(' ');
                                        cx.notify();
                                    } else if !mods.control && !mods.alt && !mods.platform && !mods.function {
                                        if let Some(kc) = &event.keystroke.key_char {
                                            d.action_command_args.push_str(kc);
                                            cx.notify();
                                        }
                                    }
                                }
                            },
                            cx,
                        )),
                )
                .into_any_element()
        };

        // --- Error message ---
        let error_el: Option<AnyElement> = draft.error.as_ref().map(|err| {
            div()
                .text_xs()
                .text_color(p.danger)
                .child(err.clone())
                .into_any_element()
        });

        let modal_surface = g.map(|gg| gg.surface).unwrap_or(p.surface);
        let modal_border = g.map(|gg| gg.border).unwrap_or(p.border);

        // --- Label field ---
        let label_field = text_field(
            &draft.label,
            "Shortcut label…",
            label_focused,
            blink,
            label_fh,
            p,
            radius,
            |this, event, _, cx| {
                let key = event.keystroke.key.as_str();
                let mods = event.keystroke.modifiers;
                if key == "escape" {
                    this.shortcut_create_draft = None;
                    cx.notify();
                    return;
                }
                if let Some(ref mut d) = this.shortcut_create_draft {
                    if key == "backspace" {
                        d.label.pop();
                        cx.notify();
                    } else if !mods.control && !mods.alt && !mods.platform && !mods.function {
                        if let Some(kc) = &event.keystroke.key_char {
                            d.label.push_str(kc);
                            cx.notify();
                        }
                    }
                }
            },
            cx,
        );

        div()
            .absolute()
            .top_0()
            .left_0()
            .right_0()
            .bottom_0()
            .child(
                // Backdrop
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .bottom_0()
                    .bg(rgb(0x000000))
                    .opacity(0.3)
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                        this.shortcut_create_draft = None;
                        this.shortcut_draft_recording_chord = false;
                        this.shortcut_draft_recording_seq = false;
                        cx.notify();
                    })),
            )
            .child(
                // Centered modal
                div()
                    .size_full()
                    .flex()
                    .justify_center()
                    .items_center()
                    .child(
                        div()
                            .w(px(520.0))
                            .p_5()
                            .rounded(px(radius.min(16.0)))
                            .bg(modal_surface)
                            .border_1()
                            .border_color(modal_border)
                            .flex()
                            .flex_col()
                            .gap_4()
                            .on_mouse_down(MouseButton::Left, cx.listener(|_, _, _, cx| {
                                cx.stop_propagation();
                            }))
                            // Header
                            .child(
                                div()
                                    .flex()
                                    .justify_between()
                                    .items_center()
                                    .child(
                                        div()
                                            .text_base()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(p.content_title)
                                            .child("Create Shortcut"),
                                    )
                                    .child(
                                        div()
                                            .px_2()
                                            .py_0p5()
                                            .text_sm()
                                            .text_color(p.ui_subtext)
                                            .cursor_pointer()
                                            .child("×")
                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                this.shortcut_create_draft = None;
                                                this.shortcut_draft_recording_chord = false;
                                                this.shortcut_draft_recording_seq = false;
                                                cx.notify();
                                            })),
                                    ),
                            )
                            // Label
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(p.ui_subtext)
                                            .child("LABEL"),
                                    )
                                    .child(label_field),
                            )
                            // Trigger
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(p.ui_subtext)
                                            .child("TRIGGER"),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .child(chord_toggle)
                                            .child(seq_toggle)
                                            .when_some(seq_counter, |d, sc| d.child(sc)),
                                    )
                                    .child(record_area),
                            )
                            // Action
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(p.ui_subtext)
                                            .child("ACTION"),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .child(navigate_toggle)
                                            .child(exec_toggle),
                                    )
                                    .child(action_section),
                            )
                            // Error
                            .when_some(error_el, |d, e| d.child(e))
                            // Footer
                            .child(
                                div()
                                    .flex()
                                    .justify_end()
                                    .gap_2()
                                    .child(
                                        div()
                                            .px_3()
                                            .py_1()
                                            .rounded(px(radius.min(8.0)))
                                            .bg(p.surface_elevated)
                                            .border_1()
                                            .border_color(p.border)
                                            .text_xs()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(p.ui_subtext)
                                            .cursor_pointer()
                                            .child("Cancel")
                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                this.shortcut_create_draft = None;
                                                this.shortcut_draft_recording_chord = false;
                                                this.shortcut_draft_recording_seq = false;
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        div()
                                            .px_3()
                                            .py_1()
                                            .rounded(px(radius.min(8.0)))
                                            .bg(p.accent)
                                            .text_xs()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(p.on_accent)
                                            .cursor_pointer()
                                            .child("Save")
                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                this.shortcut_create_save(cx);
                                            })),
                                    ),
                            ),
                    ),
            )
            .into_any_element()
    }

    pub fn shortcut_create_save(&mut self, cx: &mut Context<Self>) {
        let Some(draft) = self.shortcut_create_draft.clone() else { return; };

        let label = draft.label.trim().to_string();
        if label.is_empty() {
            if let Some(ref mut d) = self.shortcut_create_draft {
                d.error = Some("Label is required.".to_string());
            }
            cx.notify();
            return;
        }

        let trigger = match draft.trigger_kind {
            ShortcutCreateTriggerKind::Chord => {
                let Some(chord) = draft.chord else {
                    if let Some(ref mut d) = self.shortcut_create_draft {
                        d.error = Some("Record a chord first.".to_string());
                    }
                    cx.notify();
                    return;
                };
                ShortcutTrigger::Chord(chord)
            }
            ShortcutCreateTriggerKind::Sequence => {
                if draft.sequence.is_empty() {
                    if let Some(ref mut d) = self.shortcut_create_draft {
                        d.error = Some("Record all sequence steps first.".to_string());
                    }
                    cx.notify();
                    return;
                }
                if draft.sequence.len() < draft.sequence_total {
                    let msg = format!(
                        "Record all {} steps ({} so far).",
                        draft.sequence_total,
                        draft.sequence.len()
                    );
                    if let Some(ref mut d) = self.shortcut_create_draft {
                        d.error = Some(msg);
                    }
                    cx.notify();
                    return;
                }
                ShortcutTrigger::Sequence(draft.sequence)
            }
        };

        let actions = match draft.action_kind {
            ShortcutCreateActionKind::Navigate => {
                let page_id = draft.action_page_id.trim().to_string();
                if page_id.is_empty() || navigation::page_by_id(&page_id).is_none() {
                    if let Some(ref mut d) = self.shortcut_create_draft {
                        d.error = Some("Select a page to navigate to.".to_string());
                    }
                    cx.notify();
                    return;
                }
                vec![ShortcutAction::Navigate { page_id }]
            }
            ShortcutCreateActionKind::ExecuteCommand => {
                let token = draft.action_command_token.trim().to_string();
                if token.is_empty() {
                    if let Some(ref mut d) = self.shortcut_create_draft {
                        d.error = Some("Command token is required.".to_string());
                    }
                    cx.notify();
                    return;
                }
                let args: Vec<String> = draft
                    .action_command_args
                    .split_whitespace()
                    .map(str::to_string)
                    .collect();
                vec![ShortcutAction::ExecuteCommand { token, args }]
            }
        };

        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let id = format!("user:{ts}");
        let custom = CustomShortcut { id, label, trigger, actions };

        if let Ok(mut cfg) = ShortcutsConfig::load_or_create() {
            cfg.custom.push(custom);
            let _ = cfg.save();
        }

        sync_os_global_hotkeys();
        self.shortcut_create_draft = None;
        self.shortcut_draft_recording_chord = false;
        self.shortcut_draft_recording_seq = false;
        cx.notify();
    }
}
