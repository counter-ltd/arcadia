//! GPUI shell root view — split across `app/` submodules for readability.

#[cfg(feature = "gui")]
use std::path::PathBuf;

#[cfg(feature = "gui")]
mod entry;
#[cfg(feature = "ios-gui")]
pub mod entry_ios;
mod lan_nodes;
mod late;
mod lifecycle;
mod modules_page;
mod navigation;
mod python_settings;
mod root;
mod services;
#[cfg(feature = "gui")]
mod shell;
mod sidebar;
mod splash;

#[cfg(feature = "gui")]
pub use entry::run;

use arcadia_core::navigation::NavigationRegistryOwned;
use openframe::{FocusHandle, ScrollHandle, SharedString};

#[cfg(feature = "gui")]
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

/// Request to recover a port-bound service after a collision. `service_id` indexes into
/// `arcadia_core::services::SERVICE_DEFINITIONS` so the modal can call `controls.start`
/// generically once the existing process has been terminated.
#[derive(Clone, Debug)]
pub struct PendingPortKill {
    pub service_id: &'static str,
    pub service_title: &'static str,
    pub port: u16,
    pub error: String,
}

#[cfg(feature = "gui")]
#[derive(Clone, Copy, PartialEq)]
pub enum ShellMode {
    Generic,
    Internal,
}

#[cfg(feature = "gui")]
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

#[cfg(feature = "gui")]
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
    /// (name, version, description, enabled) — refreshed after python-host loads extensions.
    pub python_extension_rows: Vec<(String, String, String, bool)>,
    pub pending_module_enable: Option<(String, Vec<String>)>,
    #[cfg(feature = "gui")]
    pub terminals: Vec<TerminalInstance>,
    #[cfg(feature = "gui")]
    pub active_terminal_id: usize,
    #[cfg(feature = "gui")]
    pub next_terminal_serial: usize,
    #[cfg(feature = "gui")]
    pub terminal_context_menu_open: bool,
    #[cfg(feature = "gui")]
    pub terminal_kill_menu: Option<usize>,
    #[cfg(feature = "gui")]
    pub context_menu_position: openframe::Point<openframe::Pixels>,
    #[cfg(feature = "gui")]
    pub shell_focus: FocusHandle,
    pub late_compose_focus: FocusHandle,
    #[cfg(feature = "gui")]
    pub shell_caret_visible: bool,
    #[cfg(feature = "gui")]
    pub shell_caret_task_started: bool,
    pub splash_elapsed_ms: f32,
    pub splash_tick_started: bool,
    pub sidebar_visible: bool,
    /// When true, the sidebar Settings hub lists Logs / Modules / Settings rows.
    pub settings_hub_expanded: bool,
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
    /// Pending "kill existing process on this port and retry" prompt for any service whose
    /// `start` failed with a port-collision error. Carries enough context (service id +
    /// detected port + raw error) to recover generically without per-service GUI code.
    pub pending_port_kill_prompt: Option<PendingPortKill>,
    pub lan_poll_task_started: bool,
    pub late_poll_task_started: bool,
    pub late_last_revision: u64,
    pub late_active_room: u32,
    pub late_compose_text: String,
    pub late_settings_server_url: String,
    pub late_settings_username: String,
    pub late_settings_default_room: String,
    pub late_settings_feedback: String,
    pub late_settings_server_url_focus: FocusHandle,
    pub late_settings_username_focus: FocusHandle,
    pub late_settings_default_room_focus: FocusHandle,
}

impl ArcadiaRoot {
    #[cfg(feature = "gui")]
    pub(crate) fn active_terminal(&self) -> &TerminalInstance {
        &self.terminals[self.active_terminal_id]
    }

    #[cfg(feature = "gui")]
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
