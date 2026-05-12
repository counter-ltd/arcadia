//! GPUI shell root view — split across `app/` submodules for readability.

#[cfg(feature = "gui")]
use std::path::PathBuf;

#[cfg(feature = "gui")]
mod entry;
#[cfg(feature = "ios-gui")]
pub mod entry_ios;
mod appearance;
mod extension_token_settings;
mod lan_nodes;
mod late;
mod lifecycle;
mod list_panel_search;
mod text_input_caret;
mod modules_page;
mod navigation;
mod permissions_panel;
mod python_settings;
mod root;
mod services;
#[cfg(feature = "gui")]
mod shell;
#[cfg(feature = "ios-gui")]
mod shell_ios;
mod sidebar;
mod splash;
#[cfg(any(feature = "gui", feature = "ios-gui"))]
mod shortcuts;
mod shortcuts_create_modal;
mod shortcuts_panel;
mod shortcuts_row;
mod ai_chat_panel;
mod ai_models_panel;
mod ai_settings_panel;
mod llama_cpp_create_model_modal;
pub mod ai_runtime;
mod code_editor_panel;
mod code_editor_settings;
mod workspace_create_modal;
mod workspace_panel;
mod workspace_row;

#[cfg(feature = "gui")]
pub use entry::run;

use std::collections::HashMap;
use std::time::Instant;

use arcadia_core::modules::python_registry::{DecorationRect, HighlightSpan, StyleInfo};
use arcadia_core::shortcuts::KeyChordSpec;
use arcadia_core::navigation::NavigationRegistryOwned;
use openframe::{Bounds, FocusHandle, Pixels, ScrollHandle, SharedString};
use std::cell::RefCell;
use std::rc::Rc;

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

#[derive(Clone, PartialEq)]
pub enum ShortcutCreateTriggerKind {
    Chord,
    Sequence,
}

#[derive(Clone, PartialEq)]
pub enum ShortcutCreateActionKind {
    Navigate,
    ExecuteCommand,
}

#[derive(Clone)]
pub struct ShortcutCreateDraft {
    pub label: String,
    pub trigger_kind: ShortcutCreateTriggerKind,
    pub chord: Option<KeyChordSpec>,
    pub sequence: Vec<KeyChordSpec>,
    pub sequence_total: usize,
    pub action_kind: ShortcutCreateActionKind,
    pub action_page_id: String,
    pub action_command_token: String,
    pub action_command_args: String,
    pub error: Option<String>,
}

pub struct CodeEditorTab {
    pub id: usize,
    pub title: String,
    pub content: String,
    pub cursor: usize,
    pub selection_anchor: Option<usize>,
    /// Language identifier derived from the file extension (e.g. `"rust"`, `"python"`).
    /// Used to select a syntax highlight provider. `None` = no highlighting.
    pub language: Option<String>,
    /// Cached syntax highlight spans — recomputed only when `highlight_dirty` is true.
    pub hl_spans: Vec<HighlightSpan>,
    /// Cached decoration rects per line — recomputed only when `highlight_dirty` is true.
    pub decorations: Vec<Vec<DecorationRect>>,
    /// Set to `true` whenever `content` changes; cleared after caches are rebuilt.
    pub highlight_dirty: bool,
    /// Workspace directory path bound to this tab, if any.
    pub workspace_path: Option<String>,
    /// Absolute path to the file on disk. `None` = new unsaved buffer.
    pub file_path: Option<String>,
    /// Content at last save. Used for dirty detection and Restore.
    pub saved_content: String,
}

#[derive(Clone, PartialEq)]
pub enum AiMessageRole {
    User,
    Assistant,
}

#[derive(Clone)]
pub struct AiMessage {
    pub role: AiMessageRole,
    pub content: String,
}

pub struct AiChat {
    pub id: usize,
    pub title: String,
    pub messages: Vec<AiMessage>,
    pub input_draft: String,
    pub is_loading: bool,
}

#[derive(Clone)]
pub struct LlamaCppModelCreateDraft {
    pub name: String,
    pub path: String,
    pub mmproj_path: String,
    pub model_kind: arcadia_core::config::llama_cpp::LlamaCppModelKind,
    pub error: Option<String>,
}

#[derive(Clone)]
pub struct WorkspaceCreateDraft {
    pub label: String,
    pub path: String,
    pub error: Option<String>,
}

#[derive(Clone)]
pub enum PendingPermissionGrant {
    NativeModule {
        module: String,
        missing: Vec<String>,
    },
    PythonExtension {
        extension: String,
        missing: Vec<String>,
    },
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
    /// Filters rows on global.modules (UI-only).
    pub modules_search_query: String,
    /// Filters rows on python.settings (UI-only).
    pub extensions_search_query: String,
    /// Filters rows on global.permissions (UI-only).
    pub permissions_search_query: String,
    /// Filters rows on global.shortcuts (UI-only).
    pub shortcuts_search_query: String,
    /// Filters rows on global.workspaces (UI-only).
    pub workspace_search_query: String,
    pub modules_search_focus: FocusHandle,
    pub extensions_search_focus: FocusHandle,
    pub permissions_search_focus: FocusHandle,
    pub shortcuts_search_focus: FocusHandle,
    pub workspace_search_focus: FocusHandle,
    /// When `Some(id)`, shortcuts panel captures the next keystroke as a new chord override for that shortcut.
    pub shortcut_listening_id: Option<String>,
    /// When `Some((id, captured_steps, total_steps))`, captures successive keystrokes into a sequence override.
    pub shortcut_listening_sequence: Option<(String, Vec<arcadia_core::shortcuts::KeyChordSpec>, usize)>,
    /// Focused while listening (chord or sequence) so key events reach the root `on_key_down`.
    pub shortcut_listen_focus: FocusHandle,
    /// Draft state for the Create Shortcut modal.
    pub shortcut_create_draft: Option<ShortcutCreateDraft>,
    /// When true, next key event goes to `shortcut_create_draft.chord`.
    pub shortcut_draft_recording_chord: bool,
    /// When true, next key events accumulate into `shortcut_create_draft.sequence`.
    pub shortcut_draft_recording_seq: bool,
    pub shortcut_create_label_focus: FocusHandle,
    pub shortcut_create_token_focus: FocusHandle,
    pub shortcut_create_args_focus: FocusHandle,
    pub code_editor_show_indentation_marks: bool,
    pub code_editor_workspace_picker_open: bool,
    pub code_editor_explorer_open: bool,
    pub code_editor_explorer_expanded: std::collections::HashSet<String>,
    pub code_editor_char_width_override: Option<f32>,
    pub code_editor_char_width_draft: String,
    pub code_editor_char_width_editing: bool,
    pub code_editor_tabs: Vec<CodeEditorTab>,
    pub active_code_editor_tab: usize,
    pub code_editor_next_id: usize,
    pub code_editor_focus: FocusHandle,
    pub code_editor_char_width_focus: FocusHandle,
    pub code_editor_context_menu_open: bool,
    pub code_editor_show_dashboard: bool,
    pub code_editor_tab_menu: Option<(usize, openframe::Point<openframe::Pixels>)>,
    pub code_editor_close_confirm: Option<usize>,
    pub code_editor_line_bounds: Rc<RefCell<Vec<Bounds<Pixels>>>>,
    pub code_editor_is_dragging: bool,
    pub ai_chats: Vec<AiChat>,
    pub active_ai_chat_id: usize,
    pub ai_next_id: usize,
    pub ai_context_menu_open: bool,
    pub ai_chat_menu: Option<(usize, openframe::Point<openframe::Pixels>)>,
    pub ai_input_focus: FocusHandle,
    pub ai_default_system_prompt: String,
    /// Model selected for use in chat (model ID string).
    pub ai_chat_model_id: Option<String>,
    /// Whether the model picker dropdown is open.
    pub ai_chat_model_picker_open: bool,
    /// Workspace scoping the AI chat context (workspace ID string).
    pub ai_chat_workspace_id: Option<String>,
    /// Whether the workspace picker dropdown is open.
    pub ai_chat_workspace_picker_open: bool,
    /// Unified AI inference runtime (lazy-started on first send, shared across providers).
    pub ai_runtime: Option<ai_runtime::AiRuntimeHandle>,
    /// Which chat ID is currently receiving streamed tokens.
    pub ai_stream_chat_id: Option<usize>,
    /// Whether the 50ms inference poll task is running.
    pub ai_poll_task_started: bool,
    /// Module name of the provider selected in the Models sidebar (e.g. `"ai-provider-llama-cpp"`).
    pub active_ai_provider_module: String,
    /// Models loaded from llama-cpp.toml.
    pub llama_cpp_models: Vec<arcadia_core::config::llama_cpp::LlamaCppModel>,
    /// Endpoint URL for Ollama (loaded from ollama.toml).
    pub ollama_endpoint: String,
    /// Models discovered from the running Ollama instance (merged with ollama.toml on discovery).
    pub ollama_models: Vec<arcadia_core::config::ollama::OllamaModel>,
    /// True while a background /api/tags discovery call is in flight.
    pub ollama_discovering: bool,
    /// OpenAI API key (loaded from openai.toml).
    pub openai_api_key: String,
    /// OpenAI base URL (loaded from openai.toml).
    pub openai_base_url: String,
    /// Draft values for the OpenAI settings editor fields (Some = editing mode).
    pub openai_api_key_draft: Option<String>,
    pub openai_base_url_draft: Option<String>,
    pub openai_api_key_focus: FocusHandle,
    pub openai_base_url_focus: FocusHandle,
    /// Models loaded from openai.toml.
    pub openai_models: Vec<arcadia_core::config::openai::OpenAiModel>,
    /// Workspace entries loaded from workspace.toml — refreshed on module reload.
    pub workspace_entries: Vec<arcadia_core::config::workspace::WorkspaceEntry>,
    /// ID of the model sub-item selected under llama.cpp in the sidebar.
    pub active_llama_cpp_model_id: Option<String>,
    /// Right-click context menu on the llama.cpp provider sidebar item.
    pub llama_cpp_provider_menu: Option<openframe::Point<openframe::Pixels>>,
    /// Draft state for Create Model modal.
    pub llama_cpp_create_draft: Option<LlamaCppModelCreateDraft>,
    pub llama_cpp_create_name_focus: FocusHandle,
    pub llama_cpp_create_path_focus: FocusHandle,
    pub llama_cpp_create_mmproj_focus: FocusHandle,
    pub workspace_create_draft: Option<WorkspaceCreateDraft>,
    pub workspace_create_label_focus: FocusHandle,
    pub workspace_create_path_focus: FocusHandle,
    pub module_rows: Vec<(String, bool)>,
    /// (name, version, description, enabled) — refreshed after python-host loads extensions.
    pub python_extension_rows: Vec<(String, String, String, bool, Vec<String>, Vec<String>)>,
    /// Active render style name ("default", "tui", or python-registered).
    pub active_style: String,
    /// Built-in styles prepended, then python-registered styles appended on extension reload.
    pub available_styles: Vec<StyleInfo>,
    pub pending_module_enable: Option<(String, Vec<String>)>,
    pub pending_permission_grant: Option<PendingPermissionGrant>,
    /// Most recent error surfaced by a Python extension toggle (enable / disable). Cleared on
    /// the next toggle attempt so the panel can show "couldn't load: …" inline instead of
    /// swallowing the result of `python-host.extension-enable`.
    pub python_extension_action_error: Option<String>,
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
    /// Anchor for remote-session route picker (`session_route_menu_open`); desktop overlay only.
    #[cfg(feature = "gui")]
    pub session_route_menu_position: openframe::Point<openframe::Pixels>,
    #[cfg(feature = "gui")]
    pub shell_focus: FocusHandle,
    pub late_compose_focus: FocusHandle,
    /// Shared blink phase for focused single-line text fields (shell, compose, settings inputs).
    pub text_caret_blink_visible: bool,
    pub text_caret_blink_task_started: bool,
    pub splash_elapsed_ms: f32,
    pub splash_tick_started: bool,
    pub sidebar_visible: bool,
    /// When true, the sidebar Settings hub shows nested rows under the Settings header.
    pub settings_hub_expanded: bool,
    pub app_menu_open: bool,
    pub session_route_menu_open: bool,
    /// When `Some("lan:<ip-or-alias>")`, module visibility and routed commands use this peer.
    pub remote_route: Option<String>,
    /// Host navigation JSON from `surface.snapshot` when connected remotely (multi-client shared truth).
    pub remote_nav: Option<NavigationRegistryOwned>,
    /// Static pages plus extension token settings pages (when not using host-only thin navigation).
    pub local_navigation_registry: NavigationRegistryOwned,
    pub surface_client_id: String,
    pub last_surface_revision: Option<u64>,
    /// When `true` and [`Self::remote_route`] is set, use only host snapshot navigation (see `thin-client.toml`).
    pub navigation_from_host_only: bool,
    /// Host `surface.revision` differed from last loaded snapshot (thin client).
    pub remote_surface_stale: bool,
    pub remote_revision_poll_started: bool,
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
    /// Styling extension token values for Appearance (`module_id`, `token_key`).
    pub extension_token_values: HashMap<(String, String), String>,
    pub extension_token_editing: Option<(String, String)>,
    pub extension_token_focus: FocusHandle,
    /// Active color picker modal state: (module, key, current_color, default_color).
    pub color_picker_modal: Option<(String, String, String, String)>,
    /// Last observed window dark/light mode for mode-aware style re-application.
    pub last_color_scheme_dark: Option<bool>,
    /// Lines at top of each terminal transcript occupied by shell MOTD (incl. trailing blank), when enabled.
    #[cfg(feature = "gui")]
    pub shell_motd_prefix_lines: usize,
    #[cfg(feature = "ios-gui")]
    pub ios_shell_history: Vec<String>,
    #[cfg(feature = "ios-gui")]
    pub ios_shell_input: String,
    #[cfg(feature = "ios-gui")]
    pub ios_shell_cursor: usize,
    #[cfg(feature = "ios-gui")]
    pub ios_shell_command_history: Vec<String>,
    #[cfg(feature = "ios-gui")]
    pub ios_shell_history_index: Option<usize>,
    #[cfg(feature = "ios-gui")]
    pub ios_shell_focus: FocusHandle,
    #[cfg(feature = "ios-gui")]
    pub ios_shell_scroll: ScrollHandle,
    /// Leader-sequence state: `(shortcut_id, next_step_index)`.
    #[cfg(any(feature = "gui", feature = "ios-gui"))]
    pub shortcut_sequence_pending: Option<(String, usize)>,
    #[cfg(any(feature = "gui", feature = "ios-gui"))]
    pub shortcut_sequence_deadline: Option<Instant>,
    #[cfg(any(feature = "gui", feature = "ios-gui"))]
    pub shortcut_edge_drag_start: Option<(f32, f32)>,
    #[cfg(any(feature = "gui", feature = "ios-gui"))]
    pub shortcut_hot_corner_dwell: HashMap<String, u32>,
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
            ..Default::default()
        }
    }

    pub(crate) fn remote_navigation_required(&self) -> bool {
        self.navigation_from_host_only && self.remote_route.is_some()
    }

    pub(crate) fn thin_client_nav_waiting_host(&self) -> bool {
        self.remote_navigation_required() && self.remote_nav.is_none()
    }

    /// Host snapshot when thin-client host-only mode supplies it; otherwise merged local registry.
    pub(crate) fn navigation_registry(&self) -> Option<&NavigationRegistryOwned> {
        if self.remote_navigation_required() {
            self.remote_nav.as_ref()
        } else {
            Some(self.remote_nav.as_ref().unwrap_or(&self.local_navigation_registry))
        }
    }
}
