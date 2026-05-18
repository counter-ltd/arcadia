//! Shortcut dispatch (registry-driven), pointer triggers, and OS-global hotkeys (desktop).

#[cfg(all(feature = "gui", not(target_os = "ios")))]
mod os_hotkey;

pub fn sync_os_global_hotkeys() {
    #[cfg(all(feature = "gui", not(target_os = "ios")))]
    os_hotkey::sync_os_global_hotkeys();
}

pub fn poll_global_hotkey_events(
    this: &mut ArcadiaRoot,
    window: &mut Window,
    cx: &mut Context<ArcadiaRoot>,
) {
    #[cfg(all(feature = "gui", not(target_os = "ios")))]
    os_hotkey::poll_global_hotkey_events(this, window, cx);
    #[cfg(not(all(feature = "gui", not(target_os = "ios"))))]
    let _ = (this, window, cx);
}

use std::collections::HashMap;
use std::time::{Duration, Instant};

use arcadia_core::config::shortcuts::ShortcutsConfig;
use arcadia_core::config::ConfigFile;
use arcadia_core::modules;
use arcadia_core::navigation;
use arcadia_core::shortcuts::{
    self, EffectiveMergedShortcut, KeyChordSpec, ShortcutAction, ShortcutScope, ShortcutTrigger,
    ShortcutVisibility,
};
use openframe::{self, Context, KeyDownEvent, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Window};

use super::ArcadiaRoot;

const SEQUENCE_TIMEOUT: Duration = Duration::from_millis(1600);

fn keystroke_to_chord(event: &KeyDownEvent) -> KeyChordSpec {
    let k = event.keystroke.key.as_str();
    KeyChordSpec {
        key: k.to_string(),
        control: event.keystroke.modifiers.control,
        alt: event.keystroke.modifiers.alt,
        shift: event.keystroke.modifiers.shift,
        platform: event.keystroke.modifiers.platform,
        function: event.keystroke.modifiers.function,
    }
}

fn applicable_shortcuts(this: &ArcadiaRoot) -> Vec<EffectiveMergedShortcut> {
    let mut list: Vec<_> = shortcuts::merged_shortcuts()
        .into_iter()
        .filter(|s| shortcut_applies_to_page(this, s))
        .collect();
    list.sort_by(|a, b| b.priority.cmp(&a.priority));
    list
}

fn shortcut_applies_to_page(this: &ArcadiaRoot, s: &EffectiveMergedShortcut) -> bool {
    match &s.scope {
        ShortcutScope::ArcadiaWide => true,
        ShortcutScope::PageBound(pages) => pages.iter().any(|p| p == this.active_page_id.as_str()),
    }
}

fn text_like_focus_blocks(this: &ArcadiaRoot, window: &Window, cx: &Context<ArcadiaRoot>) -> bool {
    #[cfg(feature = "gui")]
    {
        if this.active_page_id.as_str() == "utility.shell" {
            if this.active_terminal().tui_session.is_some() {
                return true;
            }
            if this.shell_focus.contains_focused(window, cx) {
                return true;
            }
        }
    }
    if this.late.compose_focus.contains_focused(window, cx) {
        return true;
    }
    if this.late.settings_server_url_focus.contains_focused(window, cx) {
        return true;
    }
    if this.late.settings_username_focus.contains_focused(window, cx) {
        return true;
    }
    if this.late.settings_default_room_focus.contains_focused(window, cx) {
        return true;
    }
    if this.extension_token_focus.contains_focused(window, cx) {
        return true;
    }
    if this.modules_search_focus.contains_focused(window, cx) {
        return true;
    }
    if this.extensions_search_focus.contains_focused(window, cx) {
        return true;
    }
    if this.permissions_search_focus.contains_focused(window, cx) {
        return true;
    }
    if this.shortcuts_search_focus.contains_focused(window, cx) {
        return true;
    }
    if this.command_bar_focus.contains_focused(window, cx) {
        return true;
    }
    if this.goto_bar_focus.contains_focused(window, cx) {
        return true;
    }
    if this
        .shortcut_create_label_focus
        .contains_focused(window, cx)
    {
        return true;
    }
    if this
        .shortcut_create_token_focus
        .contains_focused(window, cx)
    {
        return true;
    }
    if this.shortcut_create_args_focus.contains_focused(window, cx) {
        return true;
    }
    false
}

impl ArcadiaRoot {
    pub(crate) fn try_dispatch_shortcuts_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        // Draft chord recording — next key sets draft.chord for the create modal.
        if self.shortcut_draft_recording_chord {
            self.shortcut_draft_recording_chord = false;
            let chord = keystroke_to_chord(event);
            if chord.key != "escape" {
                if let Some(ref mut draft) = self.shortcut_create_draft {
                    draft.chord = Some(chord);
                }
            }
            cx.notify();
            return true;
        }

        // Draft sequence recording — accumulate keys into draft.sequence.
        if self.shortcut_draft_recording_seq {
            let chord = keystroke_to_chord(event);
            if chord.key == "escape" {
                self.shortcut_draft_recording_seq = false;
            } else if let Some(ref mut draft) = self.shortcut_create_draft {
                draft.sequence.push(chord);
                if draft.sequence.len() < draft.sequence_total {
                    self.shortcut_listen_focus.focus(window);
                } else {
                    self.shortcut_draft_recording_seq = false;
                }
            }
            cx.notify();
            return true;
        }

        // Chord capture mode: grab the next keystroke as a single chord override.
        if let Some(id) = self.shortcut_listening_id.take() {
            let chord = keystroke_to_chord(event);
            if chord.key != "escape" {
                if let Ok(mut cfg) = ShortcutsConfig::load_or_create() {
                    let entry = cfg.overrides.entry(id).or_default();
                    entry.chord = Some(chord);
                    let _ = cfg.save();
                }
                sync_os_global_hotkeys();
            }
            cx.notify();
            return true;
        }

        // Sequence capture mode: accumulate keystrokes step-by-step.
        if self.shortcut_listening_sequence.is_some() {
            let chord = keystroke_to_chord(event);
            let (id, mut captured, total) = self.shortcut_listening_sequence.take().unwrap();
            if chord.key == "escape" {
                // cancel — leave shortcut_listening_sequence as None
            } else {
                captured.push(chord);
                if captured.len() >= total {
                    if let Ok(mut cfg) = ShortcutsConfig::load_or_create() {
                        let entry = cfg.overrides.entry(id).or_default();
                        entry.sequence = Some(captured);
                        let _ = cfg.save();
                    }
                    sync_os_global_hotkeys();
                } else {
                    // More steps remain; keep focus so next key comes through.
                    self.shortcut_listen_focus.focus(window);
                    self.shortcut_listening_sequence = Some((id, captured, total));
                }
            }
            cx.notify();
            return true;
        }

        let incoming = keystroke_to_chord(event);
        let block_text = text_like_focus_blocks(self, window, cx);
        let now = Instant::now();

        if let Some(deadline) = self.shortcut_sequence_deadline {
            if now > deadline {
                self.shortcut_sequence_pending = None;
                self.shortcut_sequence_deadline = None;
            }
        }

        if let Some((ref id, step)) = self.shortcut_sequence_pending.clone() {
            if let Some(sc) = applicable_shortcuts(self).into_iter().find(|s| s.id == *id) {
                for t in &sc.triggers {
                    if let ShortcutTrigger::Sequence(seq) = t {
                        if step < seq.len() && shortcuts::chords_match(&seq[step], &incoming) {
                            let next = step + 1;
                            if next >= seq.len() {
                                self.shortcut_sequence_pending = None;
                                self.shortcut_sequence_deadline = None;
                                self.fire_shortcut_actions(&sc, window, cx);
                                return sc.consumes;
                            }
                            self.shortcut_sequence_pending = Some((id.clone(), next));
                            self.shortcut_sequence_deadline = Some(now + SEQUENCE_TIMEOUT);
                            return sc.consumes;
                        }
                    }
                }
            }
            self.shortcut_sequence_pending = None;
            self.shortcut_sequence_deadline = None;
        }

        let list = applicable_shortcuts(self);
        for sc in &list {
            if block_text && !sc.bypass_text_focus {
                continue;
            }
            for t in &sc.triggers {
                if let ShortcutTrigger::Chord(spec) = t {
                    if shortcuts::chords_match(spec, &incoming) {
                        self.fire_shortcut_actions(sc, window, cx);
                        return sc.consumes;
                    }
                }
            }
        }

        for sc in &list {
            if block_text && !sc.bypass_text_focus {
                continue;
            }
            for t in &sc.triggers {
                if let ShortcutTrigger::Sequence(seq) = t {
                    if !seq.is_empty() && shortcuts::chords_match(&seq[0], &incoming) {
                        if seq.len() == 1 {
                            self.fire_shortcut_actions(sc, window, cx);
                            return sc.consumes;
                        }
                        self.shortcut_sequence_pending = Some((sc.id.clone(), 1));
                        self.shortcut_sequence_deadline = Some(now + SEQUENCE_TIMEOUT);
                        return sc.consumes;
                    }
                }
            }
        }

        false
    }

    pub(crate) fn fire_shortcut_actions(
        &mut self,
        sc: &EffectiveMergedShortcut,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        for a in &sc.actions {
            match a {
                ShortcutAction::Navigate { page_id } => {
                    if navigation::page_by_id(page_id).is_some() && self.is_page_visible(page_id) {
                        self.active_page_id = page_id.clone();
                        self.sync_settings_hub_expanded_from_active_page();
                        self.ensure_valid_navigation_selection();
                    }
                }
                ShortcutAction::ExecuteCommand { token, args } => {
                    let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                    let _ = modules::execute_command(token, &refs, &self.execution_context());
                }
                ShortcutAction::UiControl { control_id } => {
                    self.fire_shortcut_ui_control(control_id, window, cx);
                }
            }
        }
        cx.notify();
    }

    fn fire_shortcut_ui_control(
        &mut self,
        control_id: &str,
        window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        match control_id {
            "arcadia.toggle_command_bar" => {
                if self.command_bar_open {
                    self.command_bar_open = false;
                    self.command_bar_input.clear();
                } else {
                    self.command_bar_open = true;
                    self.command_bar_input.clear();
                    self.command_bar_focus.focus(window);
                }
            }
            "goto.open_page" => {
                let blocks = self
                    .page_ref(&self.active_page_id.clone())
                    .map(|p| p.blocks_platform_goto())
                    .unwrap_or(false);
                if !blocks {
                    if self.goto_bar_open {
                        self.goto_bar_open = false;
                        self.goto_bar_input.clear();
                    } else {
                        self.goto_bar_open = true;
                        self.goto_bar_command = "page".to_string();
                        self.goto_bar_input.clear();
                        // Initial rough estimate; canvas prepaint in render_top_bar_goto_bar
                        // refines this to the pill's exact x on the next frame.
                        let sz = window.viewport_size();
                        let vw = f32::from(sz.width);
                        self.goto_bar_anchor = openframe::point(
                            openframe::px(vw / 2.0 - 110.0),
                            openframe::px(42.0),
                        );
                        self.goto_bar_focus.focus(window);
                    }
                }
            }
            "arcadia.dismiss_overlays" => {
                self.command_bar_open = false;
                self.command_bar_input.clear();
                self.goto_bar_open = false;
                self.goto_bar_input.clear();
                self.app_menu_open = false;
                self.session_route_menu_open = false;
                #[cfg(feature = "gui")]
                {
                    self.terminal_context_menu_open = false;
                    self.terminal_kill_menu = None;
                }
                self.color_picker_modal = None;
            }
            "terminal.toggle_shell_mode" => {
                #[cfg(feature = "gui")]
                if self.active_page_id.as_str() == "utility.shell"
                    && self.active_terminal().tui_session.is_none()
                {
                    let new_mode = self.active_terminal().shell_mode.toggle();
                    self.active_terminal_mut().shell_mode = new_mode;
                }
            }
            id if id.starts_with("editor.") => {
                #[cfg(feature = "gui")]
                self.editor_run_command(id);
            }
            _ => {}
        }
    }

    pub(crate) fn handle_shortcut_pointer_move(
        &mut self,
        event: &MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let sz = window.viewport_size();
        let vw = f32::from(sz.width);
        let vh = f32::from(sz.height);
        let px = f32::from(event.position.x);
        let py = f32::from(event.position.y);

        let list = applicable_shortcuts(self);
        let mut next_dwell: HashMap<String, u32> = HashMap::new();

        for sc in &list {
            for t in &sc.triggers {
                if let ShortcutTrigger::HotCorner {
                    quadrant,
                    margin_fraction,
                    dwell_frames,
                } = t
                {
                    if shortcuts::pointer_in_hot_corner(*quadrant, *margin_fraction, px, py, vw, vh)
                    {
                        let prev = self
                            .shortcut_hot_corner_dwell
                            .get(&sc.id)
                            .copied()
                            .unwrap_or(0);
                        let n = prev.saturating_add(1);
                        next_dwell.insert(sc.id.clone(), n);
                        if n >= *dwell_frames {
                            self.fire_shortcut_actions(sc, window, cx);
                            next_dwell.remove(&sc.id);
                        }
                    }
                }
            }
        }
        self.shortcut_hot_corner_dwell = next_dwell;
    }

    pub(crate) fn handle_shortcut_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        if event.button == openframe::MouseButton::Left {
            self.shortcut_edge_drag_start =
                Some((f32::from(event.position.x), f32::from(event.position.y)));
        }
    }

    pub(crate) fn handle_shortcut_mouse_up(
        &mut self,
        event: &MouseUpEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.button != openframe::MouseButton::Left {
            return;
        }
        let Some((sx, sy)) = self.shortcut_edge_drag_start.take() else {
            return;
        };
        let ex = f32::from(event.position.x);
        let ey = f32::from(event.position.y);
        let sz = window.viewport_size();
        let vw = f32::from(sz.width);
        let vh = f32::from(sz.height);
        let margin = 0.05f32;

        let list = applicable_shortcuts(self);
        for sc in list {
            for t in &sc.triggers {
                if let ShortcutTrigger::EdgeSwipe { .. } = t {
                    if shortcuts::touch_swipe_edge_matches(t, sx, sy, ex, ey, vw, vh, margin) {
                        self.fire_shortcut_actions(&sc, window, cx);
                        return;
                    }
                }
            }
        }
    }
}

pub(crate) fn global_shortcuts_for_settings() -> Vec<EffectiveMergedShortcut> {
    shortcuts::merged_shortcuts()
        .into_iter()
        .filter(|s| {
            matches!(
                s.visibility,
                ShortcutVisibility::GlobalPrefsOnly | ShortcutVisibility::Both
            )
        })
        .collect()
}
