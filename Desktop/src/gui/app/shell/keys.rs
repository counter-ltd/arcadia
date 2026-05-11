use crate::cli;
use openframe::{Context, KeyDownEvent, Window};

use super::super::super::tui;
use super::super::ArcadiaRoot;

impl ArcadiaRoot {
    pub(crate) fn handle_shell_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.active_page_id.as_str() != "utility.shell" {
            return;
        }
        let key = event.keystroke.key.as_str();
        let mods = event.keystroke.modifiers;
        let terminal_id = self.active_terminal_id;

        // When a TUI session is active, forward all keys to the PTY.
        if self.terminals[terminal_id].tui_session.is_some() {
            let bytes = tui::key_to_bytes(key, mods).or_else(|| {
                if !mods.control && !mods.alt && !mods.platform {
                    event
                        .keystroke
                        .key_char
                        .as_ref()
                        .map(|c| c.as_bytes().to_vec())
                } else {
                    None
                }
            });
            if let Some(b) = bytes {
                if let Some(session) = self.terminals[terminal_id].tui_session.as_mut() {
                    session.write_input(&b);
                }
            }
            self.terminals[terminal_id].tui_scroll.scroll_to_bottom();
            cx.notify();
            return;
        }

        match key {
            "enter" => {
                let command = self.terminals[terminal_id].shell_input.trim().to_string();
                if !command.is_empty() {
                    self.run_shell_execute(&command, _window, cx);
                    self.terminals[terminal_id].shell_command_history.push(command);
                }
                self.terminals[terminal_id].shell_input.clear();
                self.terminals[terminal_id].shell_cursor = 0;
                self.terminals[terminal_id].shell_history_index = None;
            }
            "backspace" => {
                let term = &mut self.terminals[terminal_id];
                if term.shell_cursor > 0 {
                    let mut chars = term.shell_input.chars().collect::<Vec<_>>();
                    chars.remove(term.shell_cursor - 1);
                    term.shell_input = chars.into_iter().collect();
                    term.shell_cursor -= 1;
                }
            }
            "left" => {
                self.terminals[terminal_id].shell_cursor =
                    self.terminals[terminal_id].shell_cursor.saturating_sub(1);
            }
            "right" => {
                let term = &mut self.terminals[terminal_id];
                let len = term.shell_input.chars().count();
                term.shell_cursor = (term.shell_cursor + 1).min(len);
            }
            "up" => {
                let term = &mut self.terminals[terminal_id];
                if !term.shell_command_history.is_empty() {
                    let next_index = match term.shell_history_index {
                        Some(index) => index.saturating_sub(1),
                        None => term.shell_command_history.len().saturating_sub(1),
                    };
                    term.shell_history_index = Some(next_index);
                    term.shell_input = term.shell_command_history[next_index].clone();
                    term.shell_cursor = term.shell_input.chars().count();
                }
            }
            "down" => {
                let term = &mut self.terminals[terminal_id];
                if let Some(index) = term.shell_history_index {
                    let next_index = index + 1;
                    if next_index < term.shell_command_history.len() {
                        term.shell_history_index = Some(next_index);
                        term.shell_input = term.shell_command_history[next_index].clone();
                        term.shell_cursor = term.shell_input.chars().count();
                    } else {
                        term.shell_history_index = None;
                        term.shell_input.clear();
                        term.shell_cursor = 0;
                    }
                }
            }
            "home" => self.terminals[terminal_id].shell_cursor = 0,
            "end" => {
                self.terminals[terminal_id].shell_cursor =
                    self.terminals[terminal_id].shell_input.chars().count();
            }
            "space" => {
                let term = &mut self.terminals[terminal_id];
                let mut chars = term.shell_input.chars().collect::<Vec<_>>();
                chars.insert(term.shell_cursor, ' ');
                term.shell_input = chars.into_iter().collect();
                term.shell_cursor += 1;
            }
            _ => {
                if !mods.control && !mods.alt && !mods.platform && !mods.function {
                    if let Some(key_char) = &event.keystroke.key_char {
                        let term = &mut self.terminals[terminal_id];
                        let mut chars = term.shell_input.chars().collect::<Vec<_>>();
                        for ch in key_char.chars() {
                            chars.insert(term.shell_cursor, ch);
                            term.shell_cursor += 1;
                        }
                        term.shell_input = chars.into_iter().collect();
                    }
                }
            }
        }
        cx.notify();
    }

    pub(crate) fn handle_global_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.keystroke.key.as_str() == "escape"
            && (self.app_menu_open || self.session_route_menu_open)
        {
            self.app_menu_open = false;
            self.session_route_menu_open = false;
            cx.notify();
            return;
        }
        if self.active_page_id.as_str() != "utility.shell" {
            return;
        }
        let key = event.keystroke.key.as_str();
        let mods = event.keystroke.modifiers;
        if key == "tab" && mods.shift && self.active_terminal().tui_session.is_none() {
            let new_mode = self.active_terminal().shell_mode.toggle();
            self.active_terminal_mut().shell_mode = new_mode;
            cx.notify();
        }
    }

    pub(crate) fn run_internal_quit_command(&mut self) {
        if let crate::cli::CommandResult::Quit = cli::handle("quit") {
            std::process::exit(0);
        }
    }
}
