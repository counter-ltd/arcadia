//! GPUI shell root view — split across `app/` submodules for readability.

#[cfg(feature = "gui")]
use std::path::PathBuf;

mod ai_chat_panel;
mod ai_diff_panel;
mod ai_models_panel;
mod ai_rules_panel;
pub mod ai_runtime;
mod ai_settings_panel;
mod ai_skills_panel;
mod appearance;
mod code_editor_panel;
mod code_editor_settings;
#[cfg(feature = "gui")]
mod entry;
#[cfg(feature = "ios-gui")]
pub mod entry_ios;
mod extension_nav_panel;
mod extension_token_settings;
mod lan_nodes;
mod late;
mod lifecycle;
mod list_panel_search;
mod llama_cpp_create_model_modal;
mod modules_page;
mod navigation;
mod notification_panel;
mod notification_settings_panel;
mod permissions_panel;
mod python_settings;
mod root;
mod services;
#[cfg(feature = "gui")]
mod shell;
#[cfg(feature = "ios-gui")]
mod shell_ios;
#[cfg(any(feature = "gui", feature = "ios-gui"))]
mod shortcuts;
mod shortcuts_create_modal;
mod shortcuts_panel;
mod shortcuts_row;
mod sidebar;
mod splash;
mod text_input_caret;
mod visual_editor_panel;
mod workspace_create_modal;
mod workspace_panel;
mod workspace_row;

#[cfg(feature = "gui")]
pub use entry::run;

use std::collections::HashMap;
use std::time::Instant;

use arcadia_core::modules::python_registry::{DecorationRect, HighlightSpan, StyleInfo};
use arcadia_core::navigation::NavigationRegistryOwned;
use arcadia_core::shortcuts::KeyChordSpec;
use openframe::{Bounds, FocusHandle, Pixels, Point, ScrollHandle, SharedString, Subscription};
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
            ShellMode::Generic => "System",
            ShellMode::Internal => "Internal",
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
    /// Cached split of `content` into lines. Rebuilt when `highlight_dirty` is true.
    pub cached_lines: Vec<String>,
    /// Byte offset of each line's first character in `content`. Rebuilt with `cached_lines`.
    pub cached_line_byte_starts: Vec<usize>,
}

/// One open document in the visual block editor. The Python source in `content`
/// is canonical; the block tree is a re-derivation of it.
pub struct VisualEditorTab {
    pub id: usize,
    pub title: String,
    /// Python source — the single source of truth for this tab.
    pub content: String,
    /// Workspace directory path bound to this tab, if any.
    pub workspace_path: Option<String>,
    /// Absolute path to the file on disk. `None` = new unsaved buffer.
    pub file_path: Option<String>,
    /// Content at last save. Used for dirty detection.
    pub saved_content: String,
    /// Free-canvas (x, y) position of each top-level block, parallel to the
    /// parsed module's children. Reconciled in length on every render.
    pub block_positions: Vec<(f32, f32)>,
}

/// In-progress drag of a block on the free canvas. Any block can be dragged —
/// dropped on a compound's mouth it nests, dropped on bare canvas it becomes a
/// top-level stack.
pub struct VisualDrag {
    /// Child-index path of the dragged block.
    pub path: Vec<usize>,
    /// Pointer position when the press began (window-space pixels).
    pub origin: openframe::Point<openframe::Pixels>,
    /// Current pointer position (window-space pixels).
    pub cursor: openframe::Point<openframe::Pixels>,
    /// Pointer offset within the block at grab time, so the ghost holds the
    /// point the user grabbed instead of snapping its corner to the cursor.
    pub grab_x: f32,
    pub grab_y: f32,
    /// `false` until the pointer moves past the drag threshold — a press that
    /// never moves is a click (select only), not a drag.
    pub active: bool,
    /// Top-level stack indices snapped below the dragged block that move with
    /// it (shift-drag). Empty for an ordinary single-block drag.
    pub followers: Vec<usize>,
}

/// A registered drop target — the mouth region of a compound block.
pub struct DropZone {
    /// Child-index path of the compound owning this mouth.
    pub path: Vec<usize>,
    /// Mouth bounds in window-space pixels.
    pub bounds: Bounds<Pixels>,
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
    pub provider: String,
}

pub struct AiChat {
    pub id: usize,
    pub title: String,
    pub messages: Vec<AiMessage>,
    pub input_draft: String,
    pub is_loading: bool,
    /// Backing session ID on disk. None until first message is sent.
    pub session_id: Option<String>,
    /// Provider module used in this chat session.
    pub session_provider: String,
    /// Model ID used in this chat session.
    pub session_model_id: String,
    pub workspace_id: Option<String>,
}

#[derive(Clone)]
pub struct AiSessionSummary {
    pub id: String,
    pub title: String,
    #[allow(dead_code)]
    pub updated_at: u64,
    pub provider: String,
    pub workspace_id: Option<String>,
    pub last_messages: Vec<(bool, String)>,
}

#[derive(Clone, PartialEq)]
pub enum DiffHunkKind {
    Added,
    Removed,
    Context,
}

#[derive(Clone)]
pub struct DiffHunk {
    pub index: usize,
    pub orig_lines: Vec<String>,
    pub new_lines: Vec<String>,
    #[allow(dead_code)]
    pub kind: DiffHunkKind,
}

#[derive(Clone)]
pub struct AiPendingEdit {
    pub path: String,
    pub original: String,
    #[allow(dead_code)]
    pub proposed: String,
    pub hunks: Vec<DiffHunk>,
    pub accepted: std::collections::BTreeSet<usize>,
    pub rejected: std::collections::BTreeSet<usize>,
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
pub struct OpenAiProviderDraft {
    pub name: String,
    pub api_key: String,
    pub base_url: String,
    pub error: Option<String>,
}

#[derive(Clone)]
pub struct WorkspaceCreateDraft {
    pub label: String,
    pub path: String,
    pub error: Option<String>,
}

#[derive(Clone)]
pub struct CaretAnim {
    pub start: Instant,
    pub from: f32,
    pub to: f32,
}

#[derive(Clone)]
pub struct TabScrollAnim {
    pub start: Instant,
    pub from_x: f32,
    pub to_x: f32,
}

/// Which action bar to operate on.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ActionBarId {
    Goto,
    Command,
}

/// Shared state for all top-bar action bars (goto, command, and future bars).
pub struct ActionBarState {
    pub open: bool,
    pub input: String,
    pub focus: FocusHandle,
    pub selected_idx: Option<usize>,
    pub anchor: Point<Pixels>,
    pub pill_width: Pixels,
    /// Subcommand suffix shown in the pill prefix label (e.g. "page" → renders "goto.page").
    /// Empty string = no prefix label shown.
    pub command: String,
}

/// Phase of the notification badge preview animation.
#[derive(Clone, PartialEq)]
pub enum NotificationPreviewPhase {
    /// Pill was collapsed: animate open with preview text (300 ms).
    BadgeEnter,
    /// Pill was collapsed: hold expanded showing preview (2 s).
    BadgeHold,
    /// Pill was collapsed: animate closed (300 ms).
    BadgeExit,
    /// Pill was expanded: fade existing label out (200 ms).
    TextFadeOut,
    /// Pill was expanded: fade preview text in after swap (200 ms).
    TextFadeInPreview,
    /// Pill was expanded: hold showing preview (2 s).
    TextHold,
    /// Pill was expanded: fade preview text out (200 ms).
    TextFadeOutPreview,
    /// Pill was expanded: fade original label back in (200 ms).
    TextFadeInLabel,
}

/// State for the multi-phase notification badge preview animation.
#[derive(Clone)]
pub struct NotificationPreviewAnim {
    pub phase_start: Instant,
    pub phase: NotificationPreviewPhase,
    /// The preview title held here until alpha=0 transition point (expanded-pill path).
    /// Moved into `notification_preview_text` when the swap happens invisibly.
    pub pending_title: String,
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

pub struct CodeEditorUiState {
    pub show_indentation_marks: bool,
    pub auto_indent: bool,
    pub auto_close: bool,
    pub undo_enabled: bool,
    pub line_commands: bool,
    pub cursor_style: arcadia_core::config::code_editor::CursorStyle,
    pub undo: code_editor_panel::EditorUndoMap,
    pub workspace_picker_open: bool,
    pub undo_history_open: bool,
    pub explorer_open: bool,
    pub explorer_expanded: std::collections::HashSet<String>,
    pub char_width_override: Option<f32>,
    pub char_width_draft: String,
    pub char_width_editing: bool,
    pub tabs: Vec<CodeEditorTab>,
    pub active_tab: usize,
    pub next_id: usize,
    pub focus: FocusHandle,
    pub char_width_focus: FocusHandle,
    pub context_menu_open: bool,
    pub show_dashboard: bool,
    pub tab_menu: Option<(usize, openframe::Point<openframe::Pixels>)>,
    pub close_confirm: Option<usize>,
    pub line_bounds: Rc<RefCell<Vec<Bounds<Pixels>>>>,
    pub is_dragging: bool,
}

pub struct LateUiState {
    pub compose_focus: FocusHandle,
    pub poll_task_started: bool,
    pub last_revision: u64,
    pub active_room: u32,
    pub compose_text: String,
    pub settings_server_url: String,
    pub settings_username: String,
    pub settings_default_room: String,
    pub settings_feedback: String,
    pub settings_server_url_focus: FocusHandle,
    pub settings_username_focus: FocusHandle,
    pub settings_default_room_focus: FocusHandle,
}

pub struct VisualEditorUiState {
    pub tabs: Vec<VisualEditorTab>,
    pub active_tab: usize,
    pub next_id: usize,
    pub focus: FocusHandle,
    pub show_dashboard: bool,
    pub workspace_picker_open: bool,
    pub explorer_open: bool,
    pub explorer_expanded: std::collections::HashSet<String>,
    pub selected: Option<Vec<usize>>,
    pub edit_draft: String,
    pub edit_caret: usize,
    pub input_focus: FocusHandle,
    pub palette_open: bool,
    pub drag: Option<VisualDrag>,
    pub canvas_origin: Rc<RefCell<openframe::Point<openframe::Pixels>>>,
    pub drop_zones: Rc<RefCell<Vec<DropZone>>>,
    pub block_bounds: Rc<RefCell<Vec<(Vec<usize>, Bounds<Pixels>)>>>,
}

pub struct AiUiState {
    pub chats: Vec<AiChat>,
    pub active_chat_id: usize,
    pub next_id: usize,
    pub chat_show_dashboard: bool,
    pub context_menu_open: bool,
    pub chat_menu: Option<(usize, openframe::Point<openframe::Pixels>)>,
    pub session_menu: Option<(String, openframe::Point<openframe::Pixels>)>,
    pub session_rename: Option<(String, String)>,
    pub rename_focus: FocusHandle,
    pub input_focus: FocusHandle,
    pub default_system_prompt: String,
    pub chat_model_id: Option<String>,
    pub chat_model_picker_open: bool,
    pub chat_workspace_id: Option<String>,
    pub chat_workspace_picker_open: bool,
    /// Whether the context viewer popover is open.
    pub chat_context_viewer_open: bool,
    pub runtime: Option<ai_runtime::AiRuntimeHandle>,
    pub stream_chat_id: Option<usize>,
    pub poll_task_started: bool,
    pub active_rule_ids: Vec<String>,
    pub active_skill_ids: Vec<String>,
    pub rule_picker_open: bool,
    pub skill_picker_open: bool,
    pub sessions: Vec<AiSessionSummary>,
    pub pending_edits: Vec<AiPendingEdit>,
    pub diff_panel_open: bool,
    pub stage_writes: bool,
    pub active_provider_module: String,
    pub detected_cli_providers: Vec<arcadia_core::modules::ai_exec_cli::DetectedCliProvider>,
    pub llama_cpp_models: Vec<arcadia_core::config::llama_cpp::LlamaCppModel>,
    pub ollama_endpoint: String,
    pub ollama_models: Vec<arcadia_core::config::ollama::OllamaModel>,
    pub ollama_discovering: bool,
    pub openai_providers: Vec<arcadia_core::config::openai::OpenAiProvider>,
    pub active_openai_provider_id: Option<String>,
    pub openai_provider_edit_draft: Option<OpenAiProviderDraft>,
    pub openai_edit_name_focus: FocusHandle,
    pub openai_edit_api_key_focus: FocusHandle,
    pub openai_edit_base_url_focus: FocusHandle,
    pub openai_provider_delete_confirm: bool,
    pub openai_create_draft: Option<OpenAiProviderDraft>,
    pub openai_create_name_focus: FocusHandle,
    pub openai_create_api_key_focus: FocusHandle,
    pub openai_create_base_url_focus: FocusHandle,
    pub active_llama_cpp_model_id: Option<String>,
    pub llama_cpp_provider_menu: Option<openframe::Point<openframe::Pixels>>,
    pub ollama_provider_menu: Option<openframe::Point<openframe::Pixels>>,
    pub llama_cpp_create_draft: Option<LlamaCppModelCreateDraft>,
    pub llama_cpp_create_name_focus: FocusHandle,
    pub llama_cpp_create_path_focus: FocusHandle,
    pub llama_cpp_create_mmproj_focus: FocusHandle,
    pub llama_cpp_edit_draft: Option<LlamaCppModelCreateDraft>,
    pub llama_cpp_edit_name_focus: FocusHandle,
    pub llama_cpp_edit_path_focus: FocusHandle,
    pub llama_cpp_edit_mmproj_focus: FocusHandle,
    pub llama_cpp_delete_confirm: bool,
}

#[cfg(feature = "gui")]
pub struct TerminalInstance {
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
    /// Filters rows on extensions.settings (UI-only).
    pub extensions_search_query: String,
    /// Filters rows on global.permissions (UI-only).
    pub permissions_search_query: String,
    /// Filters rows on global.shortcuts (UI-only).
    pub shortcuts_search_query: String,
    /// Filters rows on global.workspaces (UI-only).
    pub workspace_search_query: String,
    /// Filters rows on ai.rules (UI-only).
    pub rules_search_query: String,
    /// Filters rows on ai.skills (UI-only).
    pub skills_search_query: String,
    pub modules_search_focus: FocusHandle,
    pub extensions_search_focus: FocusHandle,
    pub permissions_search_focus: FocusHandle,
    pub shortcuts_search_focus: FocusHandle,
    pub workspace_search_focus: FocusHandle,
    pub rules_search_focus: FocusHandle,
    pub skills_search_focus: FocusHandle,
    /// When `Some(id)`, shortcuts panel captures the next keystroke as a new chord override for that shortcut.
    pub shortcut_listening_id: Option<String>,
    /// When `Some((id, captured_steps, total_steps))`, captures successive keystrokes into a sequence override.
    pub shortcut_listening_sequence:
        Option<(String, Vec<arcadia_core::shortcuts::KeyChordSpec>, usize)>,
    /// Focused while listening (chord or sequence) to keep the div in the dispatch path.
    pub shortcut_listen_focus: FocusHandle,
    /// Keeps the app-level keystroke observer alive for the lifetime of this view.
    pub _shortcut_sub: Subscription,
    /// Draft state for the Create Shortcut modal.
    pub shortcut_create_draft: Option<ShortcutCreateDraft>,
    /// When true, next key event goes to `shortcut_create_draft.chord`.
    pub shortcut_draft_recording_chord: bool,
    /// When true, next key events accumulate into `shortcut_create_draft.sequence`.
    pub shortcut_draft_recording_seq: bool,
    pub shortcut_create_label_focus: FocusHandle,
    pub shortcut_create_token_focus: FocusHandle,
    pub shortcut_create_args_focus: FocusHandle,
    pub code_editor: CodeEditorUiState,
    pub visual_editor: VisualEditorUiState,
    pub ai: AiUiState,
    /// Workspace entries loaded from workspace.toml — refreshed on module reload.
    pub workspace_entries: Vec<arcadia_core::config::workspace::WorkspaceEntry>,
    pub workspace_create_draft: Option<WorkspaceCreateDraft>,
    pub workspace_create_label_focus: FocusHandle,
    pub workspace_create_path_focus: FocusHandle,
    pub module_rows: Vec<(String, bool)>,
    /// (name, version, description, enabled) — refreshed after python-host loads extensions.
    pub python_extension_rows: Vec<(String, String, String, bool, Vec<String>, Vec<String>, Vec<String>)>,
    /// Set once `PythonExtensionHost::start` has been called so `reload_modules` can start the
    /// host on-demand when python-host is enabled at runtime rather than at startup.
    pub python_host_started: bool,
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
    pub terminal_show_dashboard: bool,
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
    pub late: LateUiState,
    pub splash_elapsed_ms: f32,
    pub splash_tick_started: bool,
    pub sidebar_visible: bool,
    pub group_tabs_scroll: ScrollHandle,
    pub caret_left_alpha: f32,
    pub caret_right_alpha: f32,
    pub caret_prev_left: bool,
    pub caret_prev_right: bool,
    pub caret_left_anim: Option<CaretAnim>,
    pub caret_right_anim: Option<CaretAnim>,
    pub tab_scroll_anim: Option<TabScrollAnim>,
    pub tab_scroll_prev_x: f32,
    pub tab_scrolling_left: bool,
    pub tab_scrolling_right: bool,
    pub tab_hover_alphas: HashMap<String, f32>,
    pub tab_active_alphas: HashMap<String, f32>,
    pub tab_hover_anims: HashMap<String, CaretAnim>,
    pub tab_active_anims: HashMap<String, CaretAnim>,
    pub tab_prev_active_id: String,
    /// Hover alpha per sidebar sub-item (keyed by "chat:{id}", "aiprov:{module}", etc.)
    pub item_hover_alphas: HashMap<String, f32>,
    pub item_hover_anims: HashMap<String, CaretAnim>,
    /// When true, the sidebar Settings hub shows nested rows under the Settings header.
    pub settings_hub_expanded: bool,
    pub settings_expand_alpha: f32,
    pub settings_expand_anim: Option<CaretAnim>,
    pub pill_expanded: HashMap<String, bool>,
    pub pill_expand_alphas: HashMap<String, f32>,
    pub pill_expand_anims: HashMap<String, CaretAnim>,
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
    /// Styling extension token values for Appearance (`module_id`, `token_key`).
    pub extension_token_values: HashMap<(String, String), String>,
    pub extension_token_editing: Option<(String, String)>,
    pub extension_token_slider_active: Option<(String, String)>,
    pub extension_token_focus: FocusHandle,
    /// Active color picker modal state: (module, key, current_color, default_color).
    pub color_picker_modal: Option<(String, String, String, String)>,
    /// When the color picker is open for a gradient token, the index of the stop being edited.
    pub gradient_stop_editing_index: Option<usize>,
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
    pub command_bar: ActionBarState,
    pub goto_bar: ActionBarState,
    /// Cached unread notification count for badge display on the Notifications pill.
    pub notification_unread_count: usize,
    /// Feedback line displayed at the bottom of the notification settings panel.
    pub notification_settings_feedback: String,
    /// Draft value for the max-count field in notification settings.
    pub notification_max_count_draft: String,
    pub notification_max_count_focus: FocusHandle,
    /// Whether the notification destination dropdown is expanded.
    pub notification_dest_open: bool,
    /// Non-empty when a badge preview is active; contains the notification title to display.
    pub notification_preview_text: String,
    /// Opacity multiplier for the notification pill's text label during a cross-fade on an
    /// already-expanded pill. Normally 1.0; animated 1→0→1 during text swap.
    pub notification_content_alpha: f32,
    /// 0.0 = idle bg, 1.0 = active bg color. Lerped during preview so the pill shows a
    /// tinted background without the active border (visually distinct from truly-active state).
    pub notification_preview_bg_alpha: f32,
    /// Drives the multi-phase notification badge preview animation.
    pub notification_preview_anim: Option<NotificationPreviewAnim>,
    /// Raw progress 0.0→1.0 for the bell icon shake. Fed into `openframe::tween::shake_offset`.
    pub notification_shake_t: f32,
    /// CaretAnim that drives `notification_shake_t` from 0→1 over the shake duration.
    pub notification_shake_anim: Option<CaretAnim>,
    /// Settings pages the user has pinned to the sidebar. Persisted in `ui-prefs.toml`.
    pub pinned_settings_pages: Vec<String>,
    /// Active right-click context menu on a pinned settings sidebar item: (page_id, position).
    pub settings_pin_context_menu: Option<(String, openframe::Point<openframe::Pixels>)>,
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

    pub(crate) fn save_editor_session(&self) {
        use arcadia_core::config::code_editor::{CodeEditorSession, PersistedTab};
        use arcadia_core::config::ConfigFile;
        let tabs = self
            .code_editor.tabs
            .iter()
            .map(|t| PersistedTab {
                id: t.id,
                title: t.title.clone(),
                file_path: t.file_path.clone(),
                workspace_path: t.workspace_path.clone(),
                cursor: t.cursor,
                unsaved_content: if t.file_path.is_none() {
                    Some(t.content.clone())
                } else {
                    None
                },
            })
            .collect();
        let _ = CodeEditorSession {
            active_tab: self.code_editor.active_tab,
            next_id: self.code_editor.next_id,
            tabs,
        }
        .save();
    }

    pub(crate) fn save_visual_editor_session(&self) {
        use arcadia_core::config::visual_editor::{PersistedVisualTab, VisualEditorSession};
        use arcadia_core::config::ConfigFile;
        let tabs = self
            .visual_editor.tabs
            .iter()
            .map(|t| PersistedVisualTab {
                id: t.id,
                title: t.title.clone(),
                file_path: t.file_path.clone(),
                workspace_path: t.workspace_path.clone(),
                unsaved_content: if t.file_path.is_none() {
                    Some(t.content.clone())
                } else {
                    None
                },
            })
            .collect();
        let _ = VisualEditorSession {
            active_tab: self.visual_editor.active_tab,
            next_id: self.visual_editor.next_id,
            tabs,
        }
        .save();
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
            Some(
                self.remote_nav
                    .as_ref()
                    .unwrap_or(&self.local_navigation_registry),
            )
        }
    }
}
