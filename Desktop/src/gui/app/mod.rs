//! GPUI shell root view — split across `app/` submodules for readability.

use std::path::PathBuf;

mod entry;
mod lan_nodes;
mod late;
mod lifecycle;
mod modules_page;
mod navigation;
mod network_overview;
mod root;
mod shell;
mod sidebar;
mod splash;

pub use entry::run;

use arcadia_core::navigation::NavigationRegistryOwned;
use openframe::{FocusHandle, ScrollHandle, SharedString};

use super::tui::TuiSession;

/// Top inset so window chrome (macOS traffic lights) does not overlap the first row of UI.
pub(crate) fn window_controls_top_padding(window: &openframe::Window) -> openframe::Pixels {
    #[cfg(target_os = "macos")]
    {
        use openframe::px;
        if window.is_fullscreen() {
            px(0.)
        } else {
            (window.rem_size() * 2.25).max(px(28.))
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = window;
        openframe::px(0.)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum ShellMode {
    Generic,
    Internal,
}

impl ShellMode {
    pub(super) fn toggle(self) -> Self {
        match self {
            ShellMode::Generic => ShellMode::Internal,
            ShellMode::Internal => ShellMode::Generic,
        }
    }

    pub(super) fn label(self) -> &'static str {
        match self {
            ShellMode::Generic => "system",
            ShellMode::Internal => "internal",
        }
    }

    pub(super) fn command_token(self) -> &'static str {
        match self {
            ShellMode::Generic => "shell.execute",
            ShellMode::Internal => "shell.internal",
        }
    }
}

pub struct TerminalInstance {
    pub id: usize,
    pub label: String,
    pub shell_history: Vec<String>,
    pub shell_input: String,
    pub shell_cursor: usize,
    pub shell_command_history: Vec<String>,
    pub shell_history_index: Option<usize>,
    pub shell_stream_nonce: u64,
    pub shell_output_scroll: ScrollHandle,
    /// Keeps the embedded PTY viewport pinned to the prompt line (bottom of the terminal grid).
    pub tui_scroll: ScrollHandle,
    pub shell_mode: ShellMode,
    /// Logical cwd for each `sh -c` spawn (persists across commands).
    pub shell_working_dir: PathBuf,
    /// Shown in the top bar while a PTY session is active; tracks the foreground shell process cwd.
    pub shell_display_cwd: String,
    pub tui_session: Option<TuiSession>,
    pub tui_nonce: u64,
    pub tui_ready: bool,
    pub tui_cols: u16,
    pub tui_rows: u16,
}

pub struct ArcadiaRoot {
    pub title: SharedString,
    pub active_page_id: String,
    pub active_group_id: String,
    pub module_rows: Vec<(String, bool)>,
    pub pending_module_enable: Option<(String, Vec<String>)>,
    pub terminals: Vec<TerminalInstance>,
    pub active_terminal_id: usize,
    pub next_terminal_serial: usize,
    pub terminal_context_menu_open: bool,
    pub terminal_kill_menu: Option<usize>,
    pub context_menu_position: openframe::Point<openframe::Pixels>,
    pub shell_focus: FocusHandle,
    pub late_compose_focus: FocusHandle,
    pub shell_caret_visible: bool,
    pub shell_caret_task_started: bool,
    pub splash_elapsed_ms: f32,
    pub splash_tick_started: bool,
    pub sidebar_visible: bool,
    pub app_menu_open: bool,
    pub session_route_menu_open: bool,
    /// When `Some("lan:<ip-or-alias>")`, module visibility and routed commands use this peer.
    pub remote_route: Option<String>,
    /// Host navigation JSON from `surface.snapshot` when connected remotely (multi-client shared truth).
    pub remote_nav: Option<NavigationRegistryOwned>,
    pub surface_client_id: String,
    pub last_surface_revision: Option<u64>,
    pub lan_discovered_peers: Vec<(String, String)>,
    pub lan_command_feedback: String,
    pub lan_service_feedback: String,
    pub pending_lan_port_kill_prompt: Option<String>,
    pub lan_poll_task_started: bool,
    pub late_poll_task_started: bool,
    pub late_last_revision: u64,
    pub late_active_room: u32,
    pub late_compose_text: String,
}

impl ArcadiaRoot {
    pub(crate) fn active_terminal(&self) -> &TerminalInstance {
        &self.terminals[self.active_terminal_id]
    }

    pub(crate) fn active_terminal_mut(&mut self) -> &mut TerminalInstance {
        &mut self.terminals[self.active_terminal_id]
    }

    pub(crate) fn is_module_enabled(&self, name: &str) -> bool {
        self.module_rows
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, enabled)| *enabled)
            .unwrap_or(false)
    }

    pub(crate) fn execution_context(&self) -> arcadia_core::modules::ExecutionContext {
        arcadia_core::modules::ExecutionContext {
            net_as: self.remote_route.clone(),
            net_timeout_ms: None,
        }
    }
}
