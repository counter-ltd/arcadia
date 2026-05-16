use std::env;
#[cfg(feature = "gui")]
use std::path::PathBuf;
use std::time::Duration;

use crate::gui::theme::{
    ActiveGlyphBorderPatterns, ActiveGlyphBorderTypography, ActiveGlyphStyle,
    ActiveGlyphUiFontFamily, GlyphBorderPatterns, GlyphBorderTypography, GlyphStyleConfig,
};
use arcadia_core::config::ai::AiConfig;
use arcadia_core::config::ai_exec_providers::{AiExecProvidersConfig, ExecCliEntry};
use arcadia_core::config::appearance::AppearanceConfig;
use arcadia_core::config::code_editor::{CodeEditorConfig, CodeEditorSession};
use arcadia_core::config::extension_tokens;
use arcadia_core::config::late::LateConfig;
use arcadia_core::config::llama_cpp::LlamaCppConfig;
#[cfg(feature = "python-extensions")]
use arcadia_core::config::modules::PYTHON_HOST_MODULE_NAME;
use arcadia_core::config::modules::{ModulesConfig, LAN_MODULE_NAME, REMOTE_SESSION_MODULE_NAME};
#[cfg(feature = "gui")]
use arcadia_core::config::modules::{TERMINAL_MODULE_NAME, TERMINAL_MOTD_MODULE_NAME};
use arcadia_core::config::thin_client::ThinClientConfig;
use arcadia_core::config::ConfigFile;
use arcadia_core::modules;
use arcadia_core::modules::ai_chat_store::{self, ChatSession, StoredMessage};
use arcadia_core::modules::python_registry::{
    clamp_numeric_display_for_spec, StyleInfo, StyleTokenKind,
};
#[cfg(feature = "gui")]
use arcadia_core::modules::shell_motd;
use arcadia_core::modules::surface::{parse_surface_revision, parse_surface_snapshot};
use arcadia_core::navigation;
use openframe::ScrollHandle;
use openframe::{point, px, Context, RenderStyle, Rgba, Timer, UpdateGlobal, Window};

#[cfg(feature = "gui")]
use super::super::tui;
use super::ArcadiaRoot;
#[cfg(feature = "gui")]
use super::{CodeEditorTab, ShellMode, TerminalInstance};

/// Blocking call to Ollama /api/tags. Returns a vec of discovered models with default
/// TextGeneration kind. Called from a background thread in `discover_ollama_models`.
fn fetch_ollama_tags(
    endpoint: &str,
) -> Result<Vec<arcadia_core::config::ollama::OllamaModel>, String> {
    use arcadia_core::config::ollama::{OllamaModel, OllamaModelKind};

    const HTTP_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
    let url = format!("{}/api/tags", endpoint.trim_end_matches('/'));
    let agent = ureq::AgentBuilder::new().timeout(HTTP_TIMEOUT).build();
    let resp = agent
        .get(&url)
        .call()
        .map_err(|e| format!("Ollama /api/tags request failed: {e}"))?;
    let body: serde_json::Value = resp
        .into_json()
        .map_err(|e| format!("Ollama /api/tags parse failed: {e}"))?;

    let models = body["models"]
        .as_array()
        .ok_or_else(|| "unexpected Ollama /api/tags response shape".to_string())?
        .iter()
        .filter_map(|m| {
            let name = m["name"].as_str()?;
            Some(OllamaModel {
                id: name.to_string(),
                name: name.to_string(),
                model_tag: name.to_string(),
                model_kind: OllamaModelKind::TextGeneration,
            })
        })
        .collect();
    Ok(models)
}

#[cfg(feature = "gui")]
impl TerminalInstance {
    pub(crate) fn new(
        label: String,
        shell_working_dir: PathBuf,
        shell_display_cwd: String,
        initial_history: Vec<String>,
    ) -> Self {
        Self {
            label,
            shell_history: initial_history,
            shell_input: String::new(),
            shell_cursor: 0,
            shell_command_history: Vec::new(),
            shell_history_index: None,
            shell_stream_nonce: 0,
            shell_output_scroll: openframe::ScrollHandle::new(),
            tui_scroll: openframe::ScrollHandle::new(),
            shell_mode: ShellMode::Generic,
            shell_working_dir,
            shell_display_cwd,
            tui_session: None,
            tui_nonce: 0,
            tui_ready: false,
            tui_cols: tui::DEFAULT_COLS,
            tui_rows: tui::DEFAULT_ROWS,
        }
    }

    pub(crate) fn reset(
        &mut self,
        initial_history: Vec<String>,
        shell_working_dir: PathBuf,
        shell_display_cwd: String,
    ) {
        self.shell_stream_nonce = self.shell_stream_nonce.wrapping_add(1);
        self.shell_history = initial_history;
        self.shell_input.clear();
        self.shell_cursor = 0;
        self.shell_history_index = None;
        self.shell_output_scroll.scroll_to_bottom();
        self.shell_working_dir = shell_working_dir;
        self.shell_display_cwd = shell_display_cwd;
    }
}

impl ArcadiaRoot {
    #[cfg(feature = "gui")]
    pub(super) fn reset_shell_state(&mut self) {
        let (working_dir, display_cwd) = Self::current_dir_strings();
        let (history, motd_n) = Self::initial_shell_history(self.current_color_scheme_dark());
        self.shell_motd_prefix_lines = motd_n;
        self.active_terminal_mut()
            .reset(history, working_dir, display_cwd);
    }

    #[cfg(feature = "gui")]
    fn current_dir_strings() -> (PathBuf, String) {
        match env::current_dir() {
            Ok(path) => {
                let display = path
                    .clone()
                    .into_os_string()
                    .into_string()
                    .unwrap_or_else(|_| "cwd: unavailable".to_string());
                (path, display)
            }
            Err(_) => (PathBuf::from("/"), "cwd: unavailable".to_string()),
        }
    }

    #[cfg(feature = "gui")]
    fn initial_shell_history(is_dark: bool) -> (Vec<String>, usize) {
        let Ok(cfg) = ModulesConfig::load_or_create() else {
            return (vec!["Arcadia Terminal ready.".to_string()], 0);
        };
        let shell_on = cfg
            .modules
            .get(TERMINAL_MODULE_NAME)
            .copied()
            .unwrap_or(false);
        let motd_on = cfg
            .modules
            .get(TERMINAL_MOTD_MODULE_NAME)
            .copied()
            .unwrap_or(false);
        if shell_on && motd_on {
            let mut lines = shell_motd::motd_lines_for_scheme(is_dark);
            let n = lines.len() + 1;
            lines.push(String::new());
            (lines, n)
        } else {
            (vec!["Arcadia Terminal ready.".to_string()], 0)
        }
    }

    #[cfg(feature = "gui")]
    fn refresh_shell_motd_prefix(&mut self, is_dark: bool) {
        let Ok(cfg) = ModulesConfig::load_or_create() else {
            return;
        };
        let shell_on = cfg
            .modules
            .get(TERMINAL_MODULE_NAME)
            .copied()
            .unwrap_or(false);
        let motd_on = cfg
            .modules
            .get(TERMINAL_MOTD_MODULE_NAME)
            .copied()
            .unwrap_or(false);
        if !shell_on || !motd_on {
            let old = self.shell_motd_prefix_lines;
            if old > 0 {
                for term in &mut self.terminals {
                    if term.shell_history.len() >= old {
                        term.shell_history.drain(0..old);
                    }
                }
            }
            self.shell_motd_prefix_lines = 0;
            return;
        }

        let mut block = shell_motd::motd_lines_for_scheme(is_dark);
        let new_n = block.len() + 1;
        block.push(String::new());
        let old_n = self.shell_motd_prefix_lines;

        for term in &mut self.terminals {
            if old_n > 0 && term.shell_history.len() >= old_n {
                let tail: Vec<_> = term.shell_history[old_n..].to_vec();
                term.shell_history.truncate(0);
                term.shell_history.extend(block.iter().cloned());
                term.shell_history.extend(tail);
            }
        }
        self.shell_motd_prefix_lines = new_n;
    }

    pub fn new(cx: &mut openframe::Context<Self>) -> Self {
        #[cfg(feature = "gui")]
        let shell_focus = cx.focus_handle();
        #[cfg(feature = "ios-gui")]
        let ios_shell_focus = cx.focus_handle();
        let late_compose_focus = cx.focus_handle();
        let late_settings_server_url_focus = cx.focus_handle();
        let late_settings_username_focus = cx.focus_handle();
        let late_settings_default_room_focus = cx.focus_handle();
        let extension_token_focus = cx.focus_handle();
        let modules_search_focus = cx.focus_handle();
        let extensions_search_focus = cx.focus_handle();
        let permissions_search_focus = cx.focus_handle();
        let shortcuts_search_focus = cx.focus_handle();
        let shortcut_listen_focus = cx.focus_handle();
        let shortcut_sub = cx.observe_keystrokes(|this, event, window, cx| {
            let key_down = openframe::KeyDownEvent {
                keystroke: event.keystroke.clone(),
                is_held: false,
            };
            this.try_dispatch_shortcuts_key(&key_down, window, cx);
        });
        let shortcut_create_label_focus = cx.focus_handle();
        let shortcut_create_token_focus = cx.focus_handle();
        let shortcut_create_args_focus = cx.focus_handle();
        let workspace_search_focus = cx.focus_handle();
        let rules_search_focus = cx.focus_handle();
        let skills_search_focus = cx.focus_handle();
        let workspace_create_label_focus = cx.focus_handle();
        let workspace_create_path_focus = cx.focus_handle();
        let code_editor_focus = cx.focus_handle();
        let code_editor_char_width_focus = cx.focus_handle();
        let ai_rename_focus = cx.focus_handle();
        let ai_input_focus = cx.focus_handle();
        let llama_cpp_create_name_focus = cx.focus_handle();
        let llama_cpp_create_path_focus = cx.focus_handle();
        let llama_cpp_create_mmproj_focus = cx.focus_handle();
        let llama_cpp_edit_name_focus = cx.focus_handle();
        let llama_cpp_edit_path_focus = cx.focus_handle();
        let llama_cpp_edit_mmproj_focus = cx.focus_handle();
        let openai_edit_name_focus = cx.focus_handle();
        let openai_edit_api_key_focus = cx.focus_handle();
        let openai_edit_base_url_focus = cx.focus_handle();
        let openai_create_name_focus = cx.focus_handle();
        let openai_create_api_key_focus = cx.focus_handle();
        let openai_create_base_url_focus = cx.focus_handle();
        let command_bar_focus = cx.focus_handle();
        let notification_max_count_focus = cx.focus_handle();
        let notification_unread_count =
            arcadia_core::config::notifications::NotificationsConfig::load_or_create()
                .map(|c| c.unread_count())
                .unwrap_or(0);
        let ui_prefs_cfg =
            arcadia_core::config::ui_prefs::UiPrefsConfig::load_or_create().unwrap_or_default();
        let late_cfg = LateConfig::load_or_create().unwrap_or_default();
        let code_editor_cfg = CodeEditorConfig::load_or_create().unwrap_or_default();
        let editor_session = CodeEditorSession::load_or_create().unwrap_or_default();
        let session_active_tab = editor_session.active_tab;
        let session_next_id = editor_session.next_id.max(1);
        let restored_tabs: Vec<CodeEditorTab> = editor_session
            .tabs
            .into_iter()
            .filter_map(|pt| {
                let (content, saved_content) = if let Some(ref path) = pt.file_path {
                    match std::fs::read_to_string(path) {
                        Ok(text) => (text.clone(), text),
                        Err(_) => return None,
                    }
                } else {
                    let c = pt.unsaved_content.unwrap_or_default();
                    (c.clone(), String::new())
                };
                let lang = crate::gui::app::code_editor_panel::detect_language(&pt.title);
                let cursor = pt.cursor.min(content.len());
                Some(CodeEditorTab {
                    id: pt.id,
                    title: pt.title,
                    content,
                    cursor,
                    selection_anchor: None,
                    language: lang,
                    hl_spans: vec![],
                    decorations: vec![],
                    highlight_dirty: true,
                    workspace_path: pt.workspace_path,
                    file_path: pt.file_path,
                    saved_content,
                    cached_lines: vec![],
                    cached_line_byte_starts: vec![],
                })
            })
            .collect();
        let restored_active = if !restored_tabs.is_empty() {
            session_active_tab.min(restored_tabs.len() - 1)
        } else {
            0
        };
        let restored_show_dashboard = !restored_tabs.is_empty();
        let ai_cfg = AiConfig::load_or_create().unwrap_or_default();
        let llama_cpp_cfg = LlamaCppConfig::load_or_create().unwrap_or_default();
        let ollama_cfg =
            arcadia_core::config::ollama::OllamaConfig::load_or_create().unwrap_or_default();
        let openai_cfg =
            arcadia_core::config::openai::OpenAiConfig::load_or_create().unwrap_or_default();
        let workspace_entries = arcadia_core::config::workspace::WorkspacesConfig::load_or_create()
            .map(|c| c.workspaces)
            .unwrap_or_default();
        let mut module_rows = ModulesConfig::load_or_create()
            .map(|cfg| cfg.modules.into_iter().collect::<Vec<(String, bool)>>())
            .unwrap_or_default();
        module_rows.sort_by(|a, b| a.0.cmp(&b.0));
        let appearance_cfg = AppearanceConfig::load_or_create().unwrap_or_default();
        let active_style = appearance_cfg.active_style.clone();
        let available_styles = built_in_styles();
        // Initialize globals; glyph config is populated once extensions load.
        RenderStyle::set_global(cx, RenderStyle::Default);
        ActiveGlyphStyle::set_global(cx, ActiveGlyphStyle(None));
        ActiveGlyphBorderTypography::set_global(cx, ActiveGlyphBorderTypography(None));
        ActiveGlyphBorderPatterns::set_global(cx, ActiveGlyphBorderPatterns(None));
        ActiveGlyphUiFontFamily::set_global(cx, ActiveGlyphUiFontFamily(None));
        #[cfg(feature = "gui")]
        let (initial_hist, initial_motd_n) = Self::initial_shell_history(true);
        #[cfg(feature = "gui")]
        let first_terminal = {
            let (shell_working_dir, shell_display_cwd) = Self::current_dir_strings();
            TerminalInstance::new(
                "Terminal 1".to_string(),
                shell_working_dir,
                shell_display_cwd,
                initial_hist,
            )
        };
        let mut root = ArcadiaRoot {
            title: openframe::SharedString::new_static("Arcadia"),
            active_page_id: navigation::DEFAULT_PAGE_ID.to_string(),
            active_group_id: navigation::DEFAULT_GROUP_ID.to_string(),
            modules_search_query: String::new(),
            extensions_search_query: String::new(),
            permissions_search_query: String::new(),
            shortcuts_search_query: String::new(),
            workspace_search_query: String::new(),
            rules_search_query: String::new(),
            skills_search_query: String::new(),
            modules_search_focus,
            extensions_search_focus,
            permissions_search_focus,
            shortcuts_search_focus,
            workspace_search_focus,
            rules_search_focus,
            skills_search_focus,
            shortcut_listening_id: None,
            shortcut_listening_sequence: None,
            shortcut_listen_focus,
            _shortcut_sub: shortcut_sub,
            shortcut_create_draft: None,
            shortcut_draft_recording_chord: false,
            shortcut_draft_recording_seq: false,
            shortcut_create_label_focus,
            shortcut_create_token_focus,
            shortcut_create_args_focus,
            code_editor_show_indentation_marks: code_editor_cfg.show_indentation_marks,
            code_editor_auto_indent: code_editor_cfg.auto_indent,
            code_editor_auto_close: code_editor_cfg.auto_close_brackets,
            code_editor_undo_enabled: code_editor_cfg.undo_enabled,
            code_editor_line_commands: code_editor_cfg.line_commands,
            code_editor_cursor_style: code_editor_cfg.cursor_style,
            code_editor_undo: std::collections::HashMap::new(),
            code_editor_char_width_override: code_editor_cfg.char_width_override,
            code_editor_char_width_draft: code_editor_cfg
                .char_width_override
                .map(|v| format!("{:.2}", v))
                .unwrap_or_default(),
            code_editor_char_width_editing: false,
            code_editor_tabs: restored_tabs,
            active_code_editor_tab: restored_active,
            code_editor_next_id: session_next_id,
            code_editor_focus,
            code_editor_char_width_focus,
            code_editor_workspace_picker_open: false,
            code_editor_undo_history_open: false,
            code_editor_explorer_open: false,
            code_editor_explorer_expanded: std::collections::HashSet::new(),
            code_editor_context_menu_open: false,
            code_editor_show_dashboard: restored_show_dashboard,
            code_editor_tab_menu: None,
            code_editor_close_confirm: None,
            code_editor_line_bounds: std::rc::Rc::new(std::cell::RefCell::new(vec![])),
            code_editor_is_dragging: false,
            ai_chats: vec![],
            active_ai_chat_id: 0,
            ai_next_id: 1,
            ai_chat_show_dashboard: false,
            ai_context_menu_open: false,
            ai_chat_menu: None,
            ai_session_menu: None,
            ai_session_rename: None,
            ai_rename_focus,
            ai_input_focus,
            ai_default_system_prompt: ai_cfg.default_system_prompt,
            ai_chat_model_id: None,
            ai_chat_model_picker_open: false,
            ai_chat_workspace_id: None,
            ai_chat_workspace_picker_open: false,
            ai_runtime: None,
            ai_stream_chat_id: None,
            ai_poll_task_started: false,
            ai_active_rule_ids: Vec::new(),
            ai_active_skill_ids: Vec::new(),
            ai_rule_picker_open: false,
            ai_skill_picker_open: false,
            ai_sessions: ai_chat_store::list_sessions()
                .unwrap_or_default()
                .into_iter()
                .map(|s| crate::gui::app::AiSessionSummary {
                    id: s.id,
                    title: s.title,
                    updated_at: s.updated_at,
                    provider: s.provider,
                    workspace_id: s.workspace_id,
                    last_messages: s.last_messages,
                })
                .collect(),
            ai_pending_edits: Vec::new(),
            ai_diff_panel_open: false,
            ai_stage_writes: false,
            active_ai_provider_module: String::new(),
            detected_cli_providers: {
                let detected = arcadia_core::modules::ai_exec_cli::scan_for_cli_providers();
                // Merge newly detected providers into the persisted config so users can
                // customise model_flag / model_value / extra_args without editing code.
                let mut exec_cfg = AiExecProvidersConfig::load_or_create().unwrap_or_default();
                let mut changed = false;
                for p in &detected {
                    if !exec_cfg.entries.iter().any(|e| e.id == p.id) {
                        exec_cfg.entries.push(ExecCliEntry {
                            id: p.id.clone(),
                            binary: p.binary.clone(),
                            label: p.label.clone(),
                            model_flag: p.model_flag.clone(),
                            model_value: String::new(),
                            extra_args: p.extra_args.clone(),
                            enabled: true,
                        });
                        changed = true;
                    }
                }
                if changed {
                    let _ = exec_cfg.save();
                }
                detected
            },
            llama_cpp_models: llama_cpp_cfg.models,
            ollama_endpoint: ollama_cfg.endpoint,
            ollama_models: ollama_cfg.models,
            ollama_discovering: false,
            openai_providers: openai_cfg.providers,
            active_openai_provider_id: None,
            openai_provider_edit_draft: None,
            openai_edit_name_focus,
            openai_edit_api_key_focus,
            openai_edit_base_url_focus,
            openai_provider_delete_confirm: false,
            openai_create_draft: None,
            openai_create_name_focus,
            openai_create_api_key_focus,
            openai_create_base_url_focus,
            workspace_entries,
            active_llama_cpp_model_id: None,
            llama_cpp_provider_menu: None,
            ollama_provider_menu: None,
            llama_cpp_create_draft: None,
            llama_cpp_create_name_focus,
            llama_cpp_create_path_focus,
            llama_cpp_create_mmproj_focus,
            llama_cpp_edit_draft: None,
            llama_cpp_edit_name_focus,
            llama_cpp_edit_path_focus,
            llama_cpp_edit_mmproj_focus,
            llama_cpp_delete_confirm: false,
            workspace_create_draft: None,
            workspace_create_label_focus,
            workspace_create_path_focus,
            module_rows,
            python_extension_rows: Vec::new(),
            python_host_started: false,
            active_style,
            available_styles,
            pending_module_enable: None,
            pending_permission_grant: None,
            python_extension_action_error: None,
            #[cfg(feature = "gui")]
            terminals: vec![first_terminal],
            #[cfg(feature = "gui")]
            active_terminal_id: 0,
            #[cfg(feature = "gui")]
            next_terminal_serial: 2,
            #[cfg(feature = "gui")]
            terminal_show_dashboard: false,
            #[cfg(feature = "gui")]
            terminal_context_menu_open: false,
            #[cfg(feature = "gui")]
            terminal_kill_menu: None,
            #[cfg(feature = "gui")]
            context_menu_position: openframe::Point::default(),
            #[cfg(feature = "gui")]
            session_route_menu_position: openframe::Point::default(),
            #[cfg(feature = "gui")]
            shell_focus,
            late_compose_focus,
            text_caret_blink_visible: true,
            text_caret_blink_task_started: false,
            splash_elapsed_ms: 0.0,
            splash_tick_started: false,
            sidebar_visible: true,
            group_tabs_scroll: ScrollHandle::new(),
            caret_left_alpha: 0.0,
            caret_right_alpha: 0.0,
            caret_prev_left: false,
            caret_prev_right: false,
            caret_left_anim: None,
            caret_right_anim: None,
            tab_scroll_anim: None,
            tab_scroll_prev_x: 0.0,
            tab_scrolling_left: false,
            tab_scrolling_right: false,
            tab_hover_alphas: std::collections::HashMap::new(),
            tab_active_alphas: std::collections::HashMap::new(),
            tab_hover_anims: std::collections::HashMap::new(),
            tab_active_anims: std::collections::HashMap::new(),
            tab_prev_active_id: navigation::DEFAULT_GROUP_ID.to_string(),
            item_hover_alphas: std::collections::HashMap::new(),
            item_hover_anims: std::collections::HashMap::new(),
            settings_hub_expanded: false,
            settings_expand_alpha: 0.0,
            settings_expand_anim: None,
            pill_expanded: std::collections::HashMap::new(),
            pill_expand_alphas: std::collections::HashMap::new(),
            pill_expand_anims: std::collections::HashMap::new(),
            app_menu_open: false,
            session_route_menu_open: false,
            remote_route: None,
            remote_nav: None,
            local_navigation_registry: navigation::NavigationRegistryOwned::from_static_registry(),
            surface_client_id: ThinClientConfig::load_surface_client_id(),
            last_surface_revision: None,
            navigation_from_host_only: false,
            remote_surface_stale: false,
            remote_revision_poll_started: false,
            lan_discovered_peers: Vec::new(),
            lan_command_feedback: String::new(),
            lan_service_feedback: String::new(),
            pending_port_kill_prompt: None,
            lan_poll_task_started: false,
            late_poll_task_started: false,
            late_last_revision: 0,
            late_active_room: 1,
            late_compose_text: String::new(),
            late_settings_server_url: late_cfg.server_url,
            late_settings_username: late_cfg.username,
            late_settings_default_room: late_cfg.default_room.to_string(),
            late_settings_feedback: String::new(),
            late_settings_server_url_focus,
            late_settings_username_focus,
            late_settings_default_room_focus,
            extension_token_values: std::collections::HashMap::new(),
            extension_token_editing: None,
            extension_token_focus,
            color_picker_modal: None,
            last_color_scheme_dark: None,
            #[cfg(feature = "gui")]
            shell_motd_prefix_lines: initial_motd_n,
            #[cfg(feature = "ios-gui")]
            ios_shell_history: Vec::new(),
            #[cfg(feature = "ios-gui")]
            ios_shell_input: String::new(),
            #[cfg(feature = "ios-gui")]
            ios_shell_cursor: 0,
            #[cfg(feature = "ios-gui")]
            ios_shell_command_history: Vec::new(),
            #[cfg(feature = "ios-gui")]
            ios_shell_history_index: None,
            #[cfg(feature = "ios-gui")]
            ios_shell_focus,
            #[cfg(feature = "ios-gui")]
            ios_shell_scroll: ScrollHandle::new(),
            #[cfg(any(feature = "gui", feature = "ios-gui"))]
            shortcut_sequence_pending: None,
            #[cfg(any(feature = "gui", feature = "ios-gui"))]
            shortcut_sequence_deadline: None,
            #[cfg(any(feature = "gui", feature = "ios-gui"))]
            shortcut_edge_drag_start: None,
            #[cfg(any(feature = "gui", feature = "ios-gui"))]
            shortcut_hot_corner_dwell: std::collections::HashMap::new(),
            command_bar_open: false,
            command_bar_input: String::new(),
            command_bar_focus,
            notification_unread_count,
            notification_settings_feedback: String::new(),
            notification_max_count_draft: String::new(),
            notification_max_count_focus,
            notification_dest_open: false,
            notification_preview_text: String::new(),
            notification_content_alpha: 1.0,
            notification_preview_bg_alpha: 0.0,
            notification_preview_anim: None,
            notification_shake_t: 0.0,
            notification_shake_anim: None,
            pinned_settings_pages: ui_prefs_cfg.pinned_settings_pages,
            settings_pin_context_menu: None,
        };

        if let Ok(tc) = ThinClientConfig::load_or_create() {
            root.navigation_from_host_only = tc.navigation_from_host_only;
        }

        // Thin client bootstrap: ARCADIA_NET_AS overrides persisted thin-client.toml route.
        let mut picked_route: Option<String> = None;
        if let Ok(route) = env::var("ARCADIA_NET_AS") {
            let trimmed = route.trim();
            if !trimmed.is_empty() {
                picked_route = Some(trimmed.to_string());
            }
        } else if let Ok(tc) = ThinClientConfig::load_or_create() {
            if let Some(pref) = tc.preferred_remote_route.filter(|s| !s.trim().is_empty()) {
                picked_route = Some(pref.trim().to_string());
            }
        }
        if let Some(route) = picked_route {
            if root.is_module_enabled(LAN_MODULE_NAME)
                && root.is_module_enabled(REMOTE_SESSION_MODULE_NAME)
            {
                root.remote_route = Some(route);
                root.reload_modules();
            }
        }

        #[cfg(feature = "python-extensions")]
        if root.is_module_enabled(PYTHON_HOST_MODULE_NAME) {
            let ext_dir = arcadia_core::config::config_root_dir()
                .ok()
                .and_then(|d| d.parent().map(|p| p.join("Extensions")))
                .unwrap_or_else(|| std::path::PathBuf::from("Extensions"));
            if let Err(e) = arcadia_python::PythonExtensionHost::start(ext_dir) {
                eprintln!("python-host: {e}");
            }
            root.python_host_started = true;
            // Startup may co-enable native modules (e.g. `overlay` for HUD extensions).
            root.reload_modules();
            root.python_extension_rows = {
                let mut rows = arcadia_core::modules::python_registry::list_modules();
                rows.sort_by(|a, b| a.0.cmp(&b.0));
                rows
            };
            // Merge styles from Python extensions into available_styles.
            root.available_styles = merged_available_styles();
        }

        root.refresh_local_navigation_registry();
        root.refresh_extension_token_cache();

        // Apply persisted style (may activate a glyph config from an extension).
        let active = root.active_style.clone();
        root.apply_style(active, true, cx);

        // Sync hub/pill expansion state for the initial page (e.g. global.settings on first launch).
        root.sync_settings_hub_expanded_from_active_page();

        #[cfg(all(feature = "gui", not(target_os = "ios")))]
        super::shortcuts::sync_os_global_hotkeys();

        root
    }

    pub(crate) fn refresh_local_navigation_registry(&mut self) {
        self.local_navigation_registry =
            navigation::NavigationRegistryOwned::with_extension_token_settings_merged();
    }

    pub(super) fn refresh_extension_token_cache(&mut self) {
        self.extension_token_values.clear();
        for (module, specs) in arcadia_core::modules::python_registry::list_style_tokens() {
            let file = extension_tokens::load_module_tokens(&module).unwrap_or_default();
            for spec in specs {
                let v =
                    extension_tokens::merged_display_for_key(&spec.key, &spec.default_value, &file);
                self.extension_token_values
                    .insert((module.clone(), spec.key), v);
            }
        }
    }

    pub(crate) fn flush_extension_token_edit(
        &mut self,
        module: String,
        key: String,
        cx: &mut Context<Self>,
    ) {
        let spec = arcadia_core::modules::python_registry::list_style_tokens()
            .into_iter()
            .find(|(m, _)| m == &module)
            .and_then(|(_, specs)| specs.into_iter().find(|s| s.key == key));
        let Some(spec) = spec else {
            return;
        };
        let pair = (module.clone(), key.clone());
        let Some(raw) = self.extension_token_values.get(&pair).cloned() else {
            return;
        };
        let text_in = if matches!(spec.kind, StyleTokenKind::Int | StyleTokenKind::Float) {
            clamp_numeric_display_for_spec(&spec, raw.trim()).unwrap_or(raw)
        } else {
            raw
        };
        let mut file = extension_tokens::load_module_tokens(&module).unwrap_or_default();
        match extension_tokens::parse_input_to_value(spec.kind, &text_in) {
            Ok(val) => {
                file.insert(key.clone(), val.clone());
                if extension_tokens::save_module_tokens(&module, &file).is_ok() {
                    let disp = extension_tokens::value_to_display_string(&val);
                    self.extension_token_values.insert(pair, disp);
                    self.apply_style(
                        self.active_style.clone(),
                        self.current_color_scheme_dark(),
                        cx,
                    );
                    let reload_cmd = format!("{module}.reload");
                    let ctx = arcadia_core::modules::ExecutionContext::default();
                    let _ = modules::execute_command(&reload_cmd, &[], &ctx);
                }
            }
            Err(_) => {}
        }
    }

    pub fn reload_python_extensions(&mut self, cx: &mut Context<Self>) {
        self.python_extension_rows = {
            let mut rows = arcadia_core::modules::python_registry::list_modules();
            rows.sort_by(|a, b| a.0.cmp(&b.0));
            rows
        };
        // Merge any newly registered styles from Python extensions.
        let styles = merged_available_styles();
        // Re-apply active style; fall back to default if the extension providing it was disabled.
        let active = self.active_style.clone();
        let active = if styles.iter().any(|s| s.name == active) {
            active
        } else {
            "default".to_string()
        };
        self.available_styles = styles;
        self.refresh_extension_token_cache();
        self.refresh_local_navigation_registry();
        for tab in &mut self.code_editor_tabs {
            tab.highlight_dirty = true;
        }
        self.apply_style(active, self.current_color_scheme_dark(), cx);
    }

    pub(crate) fn current_color_scheme_dark(&self) -> bool {
        self.last_color_scheme_dark.unwrap_or(true)
    }

    pub(crate) fn refresh_style_for_mode(&mut self, is_dark: bool, cx: &mut Context<Self>) {
        if self.last_color_scheme_dark == Some(is_dark) {
            return;
        }
        self.last_color_scheme_dark = Some(is_dark);
        self.apply_style(self.active_style.clone(), is_dark, cx);
    }

    /// Persist and apply a new style selection.
    pub fn apply_style(&mut self, name: String, is_dark: bool, cx: &mut Context<Self>) {
        // Look up glyph params for the chosen style.
        let style_row = self.available_styles.iter().find(|s| s.name == name);

        let merged_glyph = style_row.and_then(|s| {
            let mut g = if is_dark {
                s.glyph.clone()
            } else {
                s.glyph_light.clone().or_else(|| s.glyph.clone())
            }?;
            if let Some(module) = s.module_name.as_deref() {
                if let Ok(file) = extension_tokens::load_module_tokens(module) {
                    extension_tokens::apply_file_tokens_to_glyph(&mut g, &file, is_dark);
                }
            }
            Some(g)
        });

        let glyph_cfg = merged_glyph.as_ref().map(|p| build_glyph_style_config(p));

        let border_typography = merged_glyph.as_ref().map(|p| GlyphBorderTypography {
            font_family: p.border_font_family.clone(),
            font_size_rems: p.border_font_size_rems,
            side_rail_px: p.border_side_rail_px,
        });

        let border_patterns = merged_glyph.as_ref().map(|p| GlyphBorderPatterns {
            horizontal: p.border_horizontal_pattern.clone(),
            vertical: p.border_vertical_pattern.clone(),
        });
        let ui_font_family = merged_glyph
            .as_ref()
            .and_then(|p| p.ui_font_family.clone())
            .filter(|s| !s.trim().is_empty());

        if glyph_cfg.is_some() {
            RenderStyle::set_global(cx, RenderStyle::Custom);
        } else {
            RenderStyle::set_global(cx, RenderStyle::Default);
        }
        ActiveGlyphStyle::set_global(cx, ActiveGlyphStyle(glyph_cfg));
        ActiveGlyphBorderTypography::set_global(cx, ActiveGlyphBorderTypography(border_typography));
        ActiveGlyphBorderPatterns::set_global(cx, ActiveGlyphBorderPatterns(border_patterns));
        ActiveGlyphUiFontFamily::set_global(cx, ActiveGlyphUiFontFamily(ui_font_family));

        self.active_style = name.clone();
        let mut cfg = AppearanceConfig::load_or_create().unwrap_or_default();
        cfg.active_style = name;
        let _ = cfg.save();
        #[cfg(feature = "gui")]
        self.refresh_shell_motd_prefix(is_dark);
        cx.notify();
    }

    pub fn reload_modules(&mut self) {
        if let Some(ref route) = self.remote_route {
            let ctx = modules::ExecutionContext {
                net_as: Some(route.clone()),
                net_timeout_ms: None,
                ..Default::default()
            };
            match modules::execute_command("surface.snapshot", &[], &ctx) {
                Ok(Some(json)) => {
                    let parsed = parse_surface_snapshot(&json);
                    self.module_rows = {
                        let mut r = parsed.modules;
                        r.sort_by(|a, b| a.0.cmp(&b.0));
                        r
                    };
                    self.remote_nav = parsed.navigation_registry;
                    self.last_surface_revision = Some(parsed.revision);
                    self.remote_surface_stale = false;
                }
                _ => {
                    self.module_rows = Vec::new();
                    self.remote_nav = None;
                    self.last_surface_revision = None;
                    self.remote_surface_stale = false;
                }
            }
        } else {
            self.remote_nav = None;
            self.last_surface_revision = None;
            self.remote_surface_stale = false;
            self.module_rows = {
                let mut r = ModulesConfig::load_or_create()
                    .map(|cfg| cfg.modules.into_iter().collect::<Vec<_>>())
                    .unwrap_or_default();
                r.sort_by(|a, b| a.0.cmp(&b.0));
                r
            };
        }
        // If python-host was just enabled at runtime (wasn't running at startup), start it now
        // so extensions are discovered and the Extensions settings page shows them immediately.
        #[cfg(feature = "python-extensions")]
        if !self.python_host_started && self.is_module_enabled(PYTHON_HOST_MODULE_NAME) {
            let ext_dir = arcadia_core::config::config_root_dir()
                .ok()
                .and_then(|d| d.parent().map(|p| p.join("Extensions")))
                .unwrap_or_else(|| std::path::PathBuf::from("Extensions"));
            if let Err(e) = arcadia_python::PythonExtensionHost::start(ext_dir) {
                eprintln!("python-host: {e}");
            }
            self.python_host_started = true;
            self.python_extension_rows = {
                let mut rows = arcadia_core::modules::python_registry::list_modules();
                rows.sort_by(|a, b| a.0.cmp(&b.0));
                rows
            };
            self.available_styles = merged_available_styles();
        }
        self.refresh_local_navigation_registry();
        self.ensure_valid_navigation_selection();
        for tab in &mut self.code_editor_tabs {
            tab.highlight_dirty = true;
        }
        #[cfg(all(feature = "gui", not(target_os = "ios")))]
        super::shortcuts::sync_os_global_hotkeys();
    }

    #[cfg(feature = "gui")]
    pub fn create_new_terminal(&mut self) {
        let serial = self.next_terminal_serial;
        self.next_terminal_serial += 1;
        let new_id = self.terminals.len();
        let (working_dir, display_cwd) = Self::current_dir_strings();
        let (hist, _) = Self::initial_shell_history(self.current_color_scheme_dark());
        let new_term = TerminalInstance::new(
            format!("Terminal {serial}"),
            working_dir,
            display_cwd,
            hist,
        );
        self.terminals.push(new_term);
        self.active_terminal_id = new_id;
        self.active_page_id = "utility.shell".to_string();
        self.sync_settings_hub_expanded_from_active_page();
        self.terminal_context_menu_open = false;
    }

    #[cfg(feature = "gui")]
    pub fn kill_terminal(&mut self, idx: usize) {
        if self.terminals.len() <= 1 {
            return;
        }
        self.terminals.remove(idx);
        if self.active_terminal_id >= self.terminals.len() {
            self.active_terminal_id = self.terminals.len() - 1;
        } else if self.active_terminal_id > idx {
            self.active_terminal_id -= 1;
        }
        self.terminal_kill_menu = None;
    }

    // NOTE: ensure_ai_poll_task, ensure_lan_poll_task, and ensure_text_caret_blink_task share
    // an identical spawn_in → loop → Timer → should_stop pattern. Extracting a generic helper
    // requires careful GPUI async-closure typing; left as a TODO for a dedicated refactor pass.
    pub fn ensure_ai_poll_task(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.ai_poll_task_started {
            return;
        }
        self.ai_poll_task_started = true;
        cx.spawn_in(
            window,
            move |view: openframe::WeakEntity<ArcadiaRoot>,
                  cx: &mut openframe::AsyncWindowContext| {
                let mut cx = cx.clone();
                async move {
                    loop {
                        Timer::after(Duration::from_millis(50)).await;
                        let should_stop = cx
                            .update(|_, app| {
                                view.update(app, |this, cx| {
                                    let had_event = this.poll_ai_events(cx);
                                    if had_event || this.ai_stream_chat_id.is_some() {
                                        false
                                    } else {
                                        this.ai_poll_task_started = false;
                                        true
                                    }
                                })
                                .unwrap_or(true)
                            })
                            .unwrap_or(true);
                        if should_stop {
                            break;
                        }
                    }
                }
            },
        )
        .detach();
    }

    /// Drain the inference event channel. Returns `true` if any event was processed.
    pub fn poll_ai_events(&mut self, cx: &mut Context<Self>) -> bool {
        use crate::gui::app::ai_runtime::RuntimeEvent;
        use crate::gui::app::{AiMessage, AiMessageRole, AiPendingEdit};

        let mut any = false;
        let mut pending_autosave: Option<usize> = None;

        loop {
            // Borrow ai_runtime only for the try_recv call; drop before any &mut self calls.
            let event = match self.ai_runtime.as_ref() {
                Some(r) => r.event_rx.try_recv(),
                None => break,
            };

            match event {
                Ok(RuntimeEvent::Token(s)) => {
                    any = true;
                    if let Some(chat_id) = self.ai_stream_chat_id {
                        if let Some(chat) = self.ai_chats.iter_mut().find(|c| c.id == chat_id) {
                            if let Some(last) = chat.messages.last_mut() {
                                if last.role == AiMessageRole::Assistant {
                                    last.content.push_str(&s);
                                }
                            }
                        }
                    }
                }
                Ok(RuntimeEvent::Done) => {
                    any = true;
                    if let Some(chat_id) = self.ai_stream_chat_id {
                        if let Some(chat) = self.ai_chats.iter_mut().find(|c| c.id == chat_id) {
                            chat.is_loading = false;
                        }
                        pending_autosave = Some(chat_id);
                    }
                    self.ai_stream_chat_id = None;
                }
                Ok(RuntimeEvent::Error(e)) => {
                    any = true;
                    if let Some(chat_id) = self.ai_stream_chat_id {
                        if let Some(chat) = self.ai_chats.iter_mut().find(|c| c.id == chat_id) {
                            chat.is_loading = false;
                            let chat_provider = chat.session_provider.clone();
                            if let Some(last) = chat.messages.last_mut() {
                                if last.role == AiMessageRole::Assistant && last.content.is_empty()
                                {
                                    last.content = format!("Error: {e}");
                                } else {
                                    chat.messages.push(AiMessage {
                                        role: AiMessageRole::Assistant,
                                        content: format!("Error: {e}"),
                                        provider: chat_provider,
                                    });
                                }
                            }
                        }
                        pending_autosave = Some(chat_id);
                    }
                    self.ai_stream_chat_id = None;
                }
                Ok(RuntimeEvent::PendingEdit {
                    path,
                    original,
                    proposed,
                }) => {
                    any = true;
                    let hunks = compute_diff_hunks(&original, &proposed);
                    self.ai_pending_edits.push(AiPendingEdit {
                        path,
                        original,
                        proposed,
                        hunks,
                        accepted: std::collections::BTreeSet::new(),
                        rejected: std::collections::BTreeSet::new(),
                    });
                    self.ai_diff_panel_open = true;
                }
                Err(_) => break,
            }
        }

        if let Some(id) = pending_autosave {
            self.autosave_session(id);
        }

        if any {
            cx.notify();
        }
        any
    }

    fn autosave_session(&mut self, chat_id: usize) {
        use crate::gui::app::AiMessageRole;

        let Some(chat) = self.ai_chats.iter_mut().find(|c| c.id == chat_id) else {
            return;
        };

        // Assign a session_id on first save.
        if chat.session_id.is_none() {
            chat.session_id = Some(ai_chat_store::new_session_id());
        }
        let session_id = chat.session_id.clone().unwrap();

        // Build or update the session from current chat state.
        let chat_title = chat.title.clone();
        let mut session = ai_chat_store::load_session(&session_id).unwrap_or_else(|_| {
            let mut s = ChatSession::new(&chat.session_provider, &chat.session_model_id);
            s.title = chat_title;
            s
        });

        // Sync id from chat.
        session.id = session_id.clone();
        // Propagate explicit rename; auto-derive from first user message otherwise.
        let default_title = format!("Chat {}", chat.id);
        let is_default = session.title == "New Chat" || session.title == default_title;
        if !is_default && chat.title != "New Chat" && chat.title != default_title {
            session.title = chat.title.clone();
        } else if is_default {
            if let Some(first_user) = session.messages.iter().find(|m| m.role == "user") {
                let derived: String = first_user
                    .content
                    .split_whitespace()
                    .take(6)
                    .collect::<Vec<_>>()
                    .join(" ");
                if !derived.is_empty() {
                    session.title = derived;
                }
            }
        }
        session.active_rule_ids = self.ai_active_rule_ids.clone();
        session.active_skill_ids = self.ai_active_skill_ids.clone();
        session.workspace_id = self.ai_chat_workspace_id.clone();

        // Rebuild messages from chat (source of truth for in-memory state).
        session.messages = chat
            .messages
            .iter()
            .map(|m| {
                let role = match m.role {
                    AiMessageRole::User => "user",
                    AiMessageRole::Assistant => "assistant",
                };
                StoredMessage {
                    role: role.to_string(),
                    content: m.content.clone(),
                    token_count: ai_chat_store::estimate_tokens(&m.content),
                }
            })
            .collect();

        // Truncate at 80% of 8192-token default budget.
        session.truncate_to_token_budget(6553);

        if let Err(e) = ai_chat_store::save_session(&session) {
            eprintln!("autosave_session: {e}");
            return;
        }

        // Back-sync title from session to in-memory chat.
        let chat = self.ai_chats.iter_mut().find(|c| c.id == chat_id).unwrap();
        if session.title != "New Chat" && session.title != format!("Chat {}", chat.id) {
            chat.title = session.title.clone();
        }

        // Refresh session summary list.
        if let Ok(summaries) = ai_chat_store::list_sessions() {
            self.ai_sessions = summaries
                .into_iter()
                .map(|s| crate::gui::app::AiSessionSummary {
                    id: s.id,
                    title: s.title,
                    updated_at: s.updated_at,
                    provider: s.provider,
                    workspace_id: s.workspace_id,
                    last_messages: s.last_messages,
                })
                .collect();
        }
    }
}

fn compute_diff_hunks(original: &str, proposed: &str) -> Vec<crate::gui::app::DiffHunk> {
    use crate::gui::app::{DiffHunk, DiffHunkKind};

    let orig_lines: Vec<&str> = original.lines().collect();
    let new_lines: Vec<&str> = proposed.lines().collect();

    // Myers-inspired simple diff: LCS-based line diff.
    // Build edit script then group into hunks.
    let edits = lcs_diff(&orig_lines, &new_lines);
    let mut hunks: Vec<DiffHunk> = Vec::new();
    let mut idx = 0usize;
    let mut i = 0;
    while i < edits.len() {
        match &edits[i] {
            Edit::Keep(o, n) => {
                let _ = (o, n);
                i += 1;
            }
            _ => {
                // Collect a contiguous block of non-Keep edits.
                let start = i;
                let mut orig_chunk: Vec<String> = Vec::new();
                let mut new_chunk: Vec<String> = Vec::new();
                while i < edits.len() {
                    match &edits[i] {
                        Edit::Keep(_, _) => break,
                        Edit::Delete(o) => {
                            orig_chunk.push(orig_lines[*o].to_string());
                            i += 1;
                        }
                        Edit::Insert(n) => {
                            new_chunk.push(new_lines[*n].to_string());
                            i += 1;
                        }
                    }
                }
                let kind = match (orig_chunk.is_empty(), new_chunk.is_empty()) {
                    (true, false) => DiffHunkKind::Added,
                    (false, true) => DiffHunkKind::Removed,
                    _ => DiffHunkKind::Context,
                };
                let _ = start;
                hunks.push(DiffHunk {
                    index: idx,
                    orig_lines: orig_chunk,
                    new_lines: new_chunk,
                    kind,
                });
                idx += 1;
            }
        }
    }
    hunks
}

enum Edit {
    Keep(usize, usize),
    Delete(usize),
    Insert(usize),
}

fn lcs_diff<'a>(a: &[&'a str], b: &[&'a str]) -> Vec<Edit> {
    let m = a.len();
    let n = b.len();
    // DP table for LCS lengths.
    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in (0..m).rev() {
        for j in (0..n).rev() {
            dp[i][j] = if a[i] == b[j] {
                dp[i + 1][j + 1] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }
    let mut edits = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < m && j < n {
        if a[i] == b[j] {
            edits.push(Edit::Keep(i, j));
            i += 1;
            j += 1;
        } else if dp[i + 1][j] >= dp[i][j + 1] {
            edits.push(Edit::Delete(i));
            i += 1;
        } else {
            edits.push(Edit::Insert(j));
            j += 1;
        }
    }
    while i < m {
        edits.push(Edit::Delete(i));
        i += 1;
    }
    while j < n {
        edits.push(Edit::Insert(j));
        j += 1;
    }
    edits
}

impl ArcadiaRoot {
    /// Spawn a background thread to fetch models from the running Ollama instance via /api/tags.
    /// Updates `self.ollama_models` on completion and clears `self.ollama_discovering`.
    pub fn discover_ollama_models(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.ollama_discovering {
            return;
        }
        self.ollama_discovering = true;
        cx.notify();

        let endpoint = self.ollama_endpoint.clone();
        cx.spawn_in(
            window,
            move |view: openframe::WeakEntity<ArcadiaRoot>,
                  cx: &mut openframe::AsyncWindowContext| {
                let mut cx = cx.clone();
                async move {
                    let (tx, rx) = std::sync::mpsc::sync_channel::<
                        Result<Vec<arcadia_core::config::ollama::OllamaModel>, String>,
                    >(1);
                    std::thread::spawn(move || {
                        let _ = tx.send(fetch_ollama_tags(&endpoint));
                    });
                    loop {
                        Timer::after(Duration::from_millis(200)).await;
                        match rx.try_recv() {
                            Ok(result) => {
                                cx.update(|_, app| {
                                    view.update(app, |this, cx| {
                                        this.ollama_discovering = false;
                                        if let Ok(models) = result {
                                            this.ollama_models = models;
                                        }
                                        cx.notify();
                                    })
                                    .ok();
                                })
                                .ok();
                                break;
                            }
                            Err(std::sync::mpsc::TryRecvError::Empty) => continue,
                            Err(_) => {
                                cx.update(|_, app| {
                                    view.update(app, |this, cx| {
                                        this.ollama_discovering = false;
                                        cx.notify();
                                    })
                                    .ok();
                                })
                                .ok();
                                break;
                            }
                        }
                    }
                }
            },
        )
        .detach();
    }

    /// Save edits to the active OpenAI provider.
    pub fn openai_provider_save_edit(&mut self, cx: &mut Context<Self>) {
        let Some(provider_id) = self.active_openai_provider_id.clone() else {
            return;
        };
        let Some(draft) = self.openai_provider_edit_draft.clone() else {
            return;
        };
        let name = draft.name.trim().to_string();
        if name.is_empty() {
            if let Some(ref mut d) = self.openai_provider_edit_draft {
                d.error = Some("Name is required.".to_string());
            }
            cx.notify();
            return;
        }
        let mut cfg =
            arcadia_core::config::openai::OpenAiConfig::load_or_create().unwrap_or_default();
        if let Some(p) = cfg.providers.iter_mut().find(|p| p.id == provider_id) {
            p.name = name.clone();
            p.api_key = draft.api_key.clone();
            p.base_url = draft.base_url.clone();
        }
        match cfg.save() {
            Ok(()) => {
                if let Some(p) = self.openai_providers.iter_mut().find(|p| p.id == provider_id) {
                    p.name = name;
                    p.api_key = draft.api_key;
                    p.base_url = draft.base_url;
                }
                self.openai_provider_edit_draft = None;
            }
            Err(e) => {
                eprintln!("openai config save failed: {e}");
            }
        }
        cx.notify();
    }

    /// Delete the active OpenAI provider.
    pub fn openai_provider_delete(&mut self, cx: &mut Context<Self>) {
        let Some(provider_id) = self.active_openai_provider_id.clone() else {
            return;
        };
        let mut cfg =
            arcadia_core::config::openai::OpenAiConfig::load_or_create().unwrap_or_default();
        cfg.providers.retain(|p| p.id != provider_id);
        if cfg.save().is_err() {
            return;
        }
        self.openai_providers.retain(|p| p.id != provider_id);
        self.active_openai_provider_id = None;
        self.openai_provider_delete_confirm = false;
        cx.notify();
    }

    /// Save a new OpenAI provider from the create draft.
    pub fn openai_provider_save_create(&mut self, cx: &mut Context<Self>) {
        let Some(draft) = self.openai_create_draft.clone() else {
            return;
        };
        let name = draft.name.trim().to_string();
        if name.is_empty() {
            if let Some(ref mut d) = self.openai_create_draft {
                d.error = Some("Name is required.".to_string());
            }
            cx.notify();
            return;
        }
        use std::time::{SystemTime, UNIX_EPOCH};
        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let id = format!("p_{ms}");
        let base_url = if draft.base_url.trim().is_empty() {
            "https://api.openai.com".to_string()
        } else {
            draft.base_url.trim().to_string()
        };
        let provider = arcadia_core::config::openai::OpenAiProvider {
            id: id.clone(),
            name,
            api_key: draft.api_key.trim().to_string(),
            base_url,
            models: Vec::new(),
        };
        let mut cfg =
            arcadia_core::config::openai::OpenAiConfig::load_or_create().unwrap_or_default();
        cfg.providers.push(provider.clone());
        match cfg.save() {
            Ok(()) => {
                self.openai_providers.push(provider);
                self.active_openai_provider_id = Some(id);
                self.openai_create_draft = None;
            }
            Err(e) => {
                if let Some(ref mut d) = self.openai_create_draft {
                    d.error = Some(format!("Save failed: {e}"));
                }
            }
        }
        cx.notify();
    }

    pub fn ensure_lan_poll_task(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.lan_poll_task_started {
            return;
        }
        if self.active_page_id != "network.nodes" {
            return;
        }
        self.lan_poll_task_started = true;
        cx.spawn_in(
            window,
            move |view: openframe::WeakEntity<ArcadiaRoot>,
                  cx: &mut openframe::AsyncWindowContext| {
                let mut cx = cx.clone();
                async move {
                    loop {
                        Timer::after(Duration::from_secs(1)).await;
                        let should_stop = cx
                            .update(|_, app| {
                                view.update(app, |this, cx| {
                                    if this.active_page_id != "network.nodes" {
                                        this.lan_poll_task_started = false;
                                        return true;
                                    }
                                    cx.notify();
                                    false
                                })
                                .unwrap_or(true)
                            })
                            .unwrap_or(true);
                        if should_stop {
                            break;
                        }
                    }
                }
            },
        )
        .detach();
    }

    #[cfg(any(feature = "gui", feature = "ios-gui"))]
    pub(crate) fn any_text_input_focused(&self, window: &Window, cx: &Context<Self>) -> bool {
        self.shell_focus.contains_focused(window, cx)
            || self.ai_input_focus.contains_focused(window, cx)
            || self.late_compose_focus.contains_focused(window, cx)
            || self
                .late_settings_server_url_focus
                .contains_focused(window, cx)
            || self
                .late_settings_username_focus
                .contains_focused(window, cx)
            || self
                .late_settings_default_room_focus
                .contains_focused(window, cx)
            || self.extension_token_focus.contains_focused(window, cx)
            || self.modules_search_focus.contains_focused(window, cx)
            || self.extensions_search_focus.contains_focused(window, cx)
            || self.permissions_search_focus.contains_focused(window, cx)
            || self.shortcuts_search_focus.contains_focused(window, cx)
            || self.workspace_search_focus.contains_focused(window, cx)
            || self
                .shortcut_create_label_focus
                .contains_focused(window, cx)
            || self
                .shortcut_create_token_focus
                .contains_focused(window, cx)
            || self.shortcut_create_args_focus.contains_focused(window, cx)
            || self.code_editor_focus.contains_focused(window, cx)
            || self
                .code_editor_char_width_focus
                .contains_focused(window, cx)
            || self.openai_edit_name_focus.contains_focused(window, cx)
            || self.openai_edit_api_key_focus.contains_focused(window, cx)
            || self.openai_edit_base_url_focus.contains_focused(window, cx)
            || self.openai_create_name_focus.contains_focused(window, cx)
            || self.openai_create_api_key_focus.contains_focused(window, cx)
            || self.openai_create_base_url_focus.contains_focused(window, cx)
            || self
                .llama_cpp_create_name_focus
                .contains_focused(window, cx)
            || self
                .llama_cpp_create_path_focus
                .contains_focused(window, cx)
            || self
                .llama_cpp_create_mmproj_focus
                .contains_focused(window, cx)
            || self.llama_cpp_edit_name_focus.contains_focused(window, cx)
            || self.llama_cpp_edit_path_focus.contains_focused(window, cx)
            || self
                .llama_cpp_edit_mmproj_focus
                .contains_focused(window, cx)
            || self
                .workspace_create_label_focus
                .contains_focused(window, cx)
            || self
                .workspace_create_path_focus
                .contains_focused(window, cx)
    }

    /// Tick the nav caret fade-in/out animations using the arcadia animation engine's easing
    /// functions. Returns `true` while any animation is still running (caller should request
    /// another animation frame).
    pub fn tick_caret_anims(&mut self, can_left: bool, can_right: bool) -> bool {
        use arcadia_core::modules::animation::{apply_easing, Easing};
        use std::time::Instant;
        const DURATION_S: f32 = 0.18;

        let now = Instant::now();

        // Detect scroll direction from frame-to-frame offset delta.
        let cur_x = f32::from(self.group_tabs_scroll.offset().x);
        let dx = cur_x - self.tab_scroll_prev_x;
        self.tab_scrolling_left = dx > 0.5;
        self.tab_scrolling_right = dx < -0.5;
        self.tab_scroll_prev_x = cur_x;

        if can_left != self.caret_prev_left {
            self.caret_prev_left = can_left;
            self.caret_left_anim = Some(super::CaretAnim {
                start: now,
                from: self.caret_left_alpha,
                to: if can_left { 1.0 } else { 0.0 },
            });
            if !can_left {
                self.tab_scrolling_left = false;
            }
        }
        if can_right != self.caret_prev_right {
            self.caret_prev_right = can_right;
            self.caret_right_anim = Some(super::CaretAnim {
                start: now,
                from: self.caret_right_alpha,
                to: if can_right { 1.0 } else { 0.0 },
            });
            if !can_right {
                self.tab_scrolling_right = false;
            }
        }

        let mut running = false;

        if let Some(anim) = &self.caret_left_anim {
            let raw_t = (now - anim.start).as_secs_f32() / DURATION_S;
            let t = apply_easing(Easing::EaseOutCubic, raw_t.min(1.0));
            self.caret_left_alpha = anim.from + (anim.to - anim.from) * t;
            if raw_t >= 1.0 {
                self.caret_left_anim = None;
            } else {
                running = true;
            }
        }

        if let Some(anim) = &self.caret_right_anim {
            let raw_t = (now - anim.start).as_secs_f32() / DURATION_S;
            let t = apply_easing(Easing::EaseOutCubic, raw_t.min(1.0));
            self.caret_right_alpha = anim.from + (anim.to - anim.from) * t;
            if raw_t >= 1.0 {
                self.caret_right_anim = None;
            } else {
                running = true;
            }
        }

        if let Some(anim) = &self.tab_scroll_anim.clone() {
            let raw_t = (now - anim.start).as_secs_f32() / DURATION_S;
            let t = apply_easing(Easing::EaseOutCubic, raw_t.min(1.0));
            let new_x = anim.from_x + (anim.to_x - anim.from_x) * t;
            let cur_y = self.group_tabs_scroll.offset().y;
            self.group_tabs_scroll.set_offset(point(px(new_x), cur_y));
            if raw_t >= 1.0 {
                self.tab_scroll_anim = None;
                self.tab_scrolling_left = false;
                self.tab_scrolling_right = false;
                self.tab_scroll_prev_x = new_x;
            } else {
                running = true;
            }
        }

        // Detect active group change and kick off active-state transition.
        if self.active_group_id != self.tab_prev_active_id {
            let old_id = self.tab_prev_active_id.clone();
            let new_id = self.active_group_id.clone();
            if !old_id.is_empty() {
                let from = *self.tab_active_alphas.get(&old_id).unwrap_or(&1.0);
                self.tab_active_anims.insert(
                    old_id,
                    super::CaretAnim {
                        start: now,
                        from,
                        to: 0.0,
                    },
                );
            }
            let from = *self.tab_active_alphas.get(&new_id).unwrap_or(&0.0);
            self.tab_active_anims.insert(
                new_id.clone(),
                super::CaretAnim {
                    start: now,
                    from,
                    to: 1.0,
                },
            );
            self.tab_prev_active_id = self.active_group_id.clone();
        }

        // Tick per-tab hover animations.
        const HOVER_DURATION_S: f32 = 0.12;
        let hover_keys: Vec<String> = self.tab_hover_anims.keys().cloned().collect();
        for gid in hover_keys {
            if let Some(anim) = self.tab_hover_anims.get(&gid).cloned() {
                let raw_t = (now - anim.start).as_secs_f32() / HOVER_DURATION_S;
                let t = apply_easing(Easing::EaseOutCubic, raw_t.min(1.0));
                self.tab_hover_alphas
                    .insert(gid.clone(), anim.from + (anim.to - anim.from) * t);
                if raw_t >= 1.0 {
                    self.tab_hover_anims.remove(&gid);
                } else {
                    running = true;
                }
            }
        }

        // Tick per-tab active animations.
        let active_keys: Vec<String> = self.tab_active_anims.keys().cloned().collect();
        for gid in active_keys {
            if let Some(anim) = self.tab_active_anims.get(&gid).cloned() {
                let raw_t = (now - anim.start).as_secs_f32() / HOVER_DURATION_S;
                let t = apply_easing(Easing::EaseOutCubic, raw_t.min(1.0));
                self.tab_active_alphas
                    .insert(gid.clone(), anim.from + (anim.to - anim.from) * t);
                if raw_t >= 1.0 {
                    self.tab_active_anims.remove(&gid);
                } else {
                    running = true;
                }
            }
        }

        // Tick per-item hover animations (AI chat/model/provider sub-items).
        let item_keys: Vec<String> = self.item_hover_anims.keys().cloned().collect();
        for key in item_keys {
            if let Some(anim) = self.item_hover_anims.get(&key).cloned() {
                let raw_t = (now - anim.start).as_secs_f32() / HOVER_DURATION_S;
                let t = apply_easing(Easing::EaseOutCubic, raw_t.min(1.0));
                self.item_hover_alphas
                    .insert(key.clone(), anim.from + (anim.to - anim.from) * t);
                if raw_t >= 1.0 {
                    self.item_hover_anims.remove(&key);
                } else {
                    running = true;
                }
            }
        }

        // Tick settings hub expand/collapse animation.
        if let Some(anim) = self.settings_expand_anim.clone() {
            let raw_t = (now - anim.start).as_secs_f32() / HOVER_DURATION_S;
            let t = apply_easing(Easing::EaseOutCubic, raw_t.min(1.0));
            self.settings_expand_alpha = anim.from + (anim.to - anim.from) * t;
            if raw_t >= 1.0 {
                self.settings_expand_anim = None;
            } else {
                running = true;
            }
        }

        // Tick top-bar pill expand/collapse animations.
        const PILL_DURATION_S: f32 = 0.18;
        let pill_keys: Vec<String> = self.pill_expand_anims.keys().cloned().collect();
        for key in pill_keys {
            if let Some(anim) = self.pill_expand_anims.get(&key).cloned() {
                let raw_t = (now - anim.start).as_secs_f32() / PILL_DURATION_S;
                let t = apply_easing(Easing::EaseOutCubic, raw_t.min(1.0));
                self.pill_expand_alphas
                    .insert(key.clone(), anim.from + (anim.to - anim.from) * t);
                if raw_t >= 1.0 {
                    self.pill_expand_anims.remove(&key);
                } else {
                    running = true;
                }
            }
        }

        // Tick notification badge preview animation.
        if self.tick_notification_preview(now) {
            running = true;
        }

        // Tick bell-icon shake animation (raw t — no easing, shake_offset does its own math).
        const SHAKE_DURATION_S: f32 = 0.5;
        if let Some(anim) = &self.notification_shake_anim.clone() {
            let raw_t = (now - anim.start).as_secs_f32() / SHAKE_DURATION_S;
            self.notification_shake_t = raw_t.min(1.0);
            if raw_t >= 1.0 {
                self.notification_shake_anim = None;
                self.notification_shake_t = 0.0;
            } else {
                running = true;
            }
        }

        running
    }

    pub fn start_settings_expand_anim(&mut self, expanding: bool) {
        use std::time::Instant;
        let from = self.settings_expand_alpha;
        let to   = if expanding { 1.0_f32 } else { 0.0_f32 };
        self.settings_expand_anim = Some(super::CaretAnim { start: Instant::now(), from, to });
    }

    pub fn start_pill_expand_anim(&mut self, page_id: &str, expand: bool) {
        use std::time::Instant;
        let from = *self.pill_expand_alphas.get(page_id).unwrap_or(&0.0);
        let to = if expand { 1.0_f32 } else { 0.0_f32 };
        self.pill_expand_anims.insert(
            page_id.to_string(),
            super::CaretAnim { start: Instant::now(), from, to },
        );
    }

    /// Tick the notification badge preview animation. Returns true while still running.
    pub fn tick_notification_preview(&mut self, now: std::time::Instant) -> bool {
        use arcadia_core::modules::animation::{apply_easing, Easing};
        use super::{NotificationPreviewAnim, NotificationPreviewPhase};

        const PILL_FADE_S: f32 = 0.30;
        const TEXT_FADE_S: f32 = 0.20;
        const HOLD_S: f32 = 2.0;

        let Some(anim) = self.notification_preview_anim.clone() else {
            return false;
        };
        let elapsed = (now - anim.phase_start).as_secs_f32();

        match anim.phase {
            NotificationPreviewPhase::PillEnter => {
                let t = apply_easing(Easing::EaseOutCubic, (elapsed / PILL_FADE_S).min(1.0));
                self.pill_expand_alphas.insert("notification.main".to_string(), t);
                self.notification_preview_bg_alpha = t;
                if elapsed >= PILL_FADE_S {
                    self.pill_expand_alphas.insert("notification.main".to_string(), 1.0);
                    self.notification_preview_bg_alpha = 1.0;
                    self.notification_preview_anim = Some(NotificationPreviewAnim {
                        phase_start: now,
                        phase: NotificationPreviewPhase::PillHold,
                        pending_title: anim.pending_title,
                    });
                }
                true
            }
            NotificationPreviewPhase::PillHold => {
                if elapsed >= HOLD_S {
                    self.notification_preview_anim = Some(NotificationPreviewAnim {
                        phase_start: now,
                        phase: NotificationPreviewPhase::PillExit,
                        pending_title: anim.pending_title,
                    });
                }
                true
            }
            NotificationPreviewPhase::PillExit => {
                let t = apply_easing(Easing::EaseInCubic, (elapsed / PILL_FADE_S).min(1.0));
                self.pill_expand_alphas.insert("notification.main".to_string(), 1.0 - t);
                self.notification_preview_bg_alpha = 1.0 - t;
                if elapsed >= PILL_FADE_S {
                    self.notification_preview_anim = None;
                    self.notification_preview_text.clear();
                    self.notification_preview_bg_alpha = 0.0;
                    self.pill_expand_alphas.remove("notification.main");
                    // Restore pill expand state from current active page.
                    self.sync_pill_expand_from_active_page();
                    return false;
                }
                true
            }
            NotificationPreviewPhase::TextFadeOut => {
                // Fades the *original* label out. notification_preview_text is still empty
                // so the render shows the real page title during this phase.
                // bg fades in as label fades out (complementary).
                let t = apply_easing(Easing::EaseInCubic, (elapsed / TEXT_FADE_S).min(1.0));
                self.notification_content_alpha = 1.0 - t;
                self.notification_preview_bg_alpha = t;
                if elapsed >= TEXT_FADE_S {
                    // Alpha is 0 — invisible. Swap text now so the change is seamless.
                    self.notification_content_alpha = 0.0;
                    self.notification_preview_bg_alpha = 1.0;
                    self.notification_preview_text = anim.pending_title.clone();
                    self.notification_preview_anim = Some(NotificationPreviewAnim {
                        phase_start: now,
                        phase: NotificationPreviewPhase::TextFadeInPreview,
                        pending_title: anim.pending_title,
                    });
                }
                true
            }
            NotificationPreviewPhase::TextFadeInPreview => {
                let t = apply_easing(Easing::EaseOutCubic, (elapsed / TEXT_FADE_S).min(1.0));
                self.notification_content_alpha = t;
                // bg stays at 1.0
                if elapsed >= TEXT_FADE_S {
                    self.notification_content_alpha = 1.0;
                    self.notification_preview_anim = Some(NotificationPreviewAnim {
                        phase_start: now,
                        phase: NotificationPreviewPhase::TextHold,
                        pending_title: anim.pending_title,
                    });
                }
                true
            }
            NotificationPreviewPhase::TextHold => {
                // bg stays at 1.0
                if elapsed >= HOLD_S {
                    self.notification_preview_anim = Some(NotificationPreviewAnim {
                        phase_start: now,
                        phase: NotificationPreviewPhase::TextFadeOutPreview,
                        pending_title: anim.pending_title,
                    });
                }
                true
            }
            NotificationPreviewPhase::TextFadeOutPreview => {
                let t = apply_easing(Easing::EaseInCubic, (elapsed / TEXT_FADE_S).min(1.0));
                self.notification_content_alpha = 1.0 - t;
                // bg stays at 1.0 until label starts fading back in
                if elapsed >= TEXT_FADE_S {
                    // Alpha is 0 — invisible. Clear preview text; render reverts to page title.
                    self.notification_preview_text.clear();
                    self.notification_content_alpha = 0.0;
                    self.notification_preview_anim = Some(NotificationPreviewAnim {
                        phase_start: now,
                        phase: NotificationPreviewPhase::TextFadeInLabel,
                        pending_title: anim.pending_title,
                    });
                }
                true
            }
            NotificationPreviewPhase::TextFadeInLabel => {
                let t = apply_easing(Easing::EaseOutCubic, (elapsed / TEXT_FADE_S).min(1.0));
                self.notification_content_alpha = t;
                // bg fades out as label fades back in (complementary).
                self.notification_preview_bg_alpha = 1.0 - t;
                if elapsed >= TEXT_FADE_S {
                    self.notification_content_alpha = 1.0;
                    self.notification_preview_bg_alpha = 0.0;
                    self.notification_preview_anim = None;
                    return false;
                }
                true
            }
        }
    }

    /// Trigger a notification preview on the badge pill.
    /// If the pill is already expanded (user is on the notifications page or a settings page),
    /// cross-fades the label text to `title` and back. Otherwise expands the pill, shows the
    /// preview, then collapses it again.
    pub fn start_notification_preview(&mut self, title: String) {
        use std::time::Instant;
        use super::{NotificationPreviewAnim, NotificationPreviewPhase};
        if self.notification_preview_anim.is_some() {
            return;
        }
        // Always shake the bell icon regardless of pill state.
        self.notification_shake_t = 0.0;
        self.notification_shake_anim = Some(super::CaretAnim {
            start: Instant::now(),
            from: 0.0,
            to: 1.0,
        });

        let pill_is_expanded = *self.pill_expanded.get("notification.main").unwrap_or(&false);
        if pill_is_expanded {
            // Don't set notification_preview_text yet — the TextFadeOut phase fades the
            // *original* label out. The swap to preview text happens at the invisible
            // alpha=0 transition into TextFadeInPreview.
            self.notification_content_alpha = 1.0;
            self.notification_preview_anim = Some(NotificationPreviewAnim {
                phase_start: Instant::now(),
                phase: NotificationPreviewPhase::TextFadeOut,
                pending_title: title,
            });
        } else {
            // Collapsed pill: set text immediately — pill starts invisible so no flash.
            self.notification_preview_text = title.clone();
            self.notification_preview_anim = Some(NotificationPreviewAnim {
                phase_start: Instant::now(),
                phase: NotificationPreviewPhase::PillEnter,
                pending_title: title,
            });
        }
    }

    /// Start or reverse a hover fade for a tab group item.
    pub fn start_tab_hover_anim(&mut self, group_id: String, hovered: bool) {
        use std::time::Instant;
        let from = *self.tab_hover_alphas.get(&group_id).unwrap_or(&0.0);
        let to = if hovered { 1.0_f32 } else { 0.0_f32 };
        self.tab_hover_anims.insert(
            group_id,
            super::CaretAnim {
                start: Instant::now(),
                from,
                to,
            },
        );
    }

    /// Start or reverse a hover fade for a sidebar sub-item (chat, model, provider, etc.).
    pub fn start_item_hover_anim(&mut self, key: String, hovered: bool) {
        use std::time::Instant;
        let from = *self.item_hover_alphas.get(&key).unwrap_or(&0.0);
        let to = if hovered { 1.0_f32 } else { 0.0_f32 };
        self.item_hover_anims.insert(
            key,
            super::CaretAnim {
                start: Instant::now(),
                from,
                to,
            },
        );
    }

    /// Compute the minimal scroll offset to bring tab `ix` fully into view, then animate to it.
    /// Falls back to `scroll_to_item` snap if child bounds are not yet available.
    pub fn start_tab_scroll_anim(&mut self, ix: usize) {
        let viewport = self.group_tabs_scroll.bounds();
        let cur_x = self.group_tabs_scroll.offset().x;

        let to_x = if let Some(item) = self.group_tabs_scroll.bounds_for_item(ix) {
            if item.left() + cur_x < viewport.left() {
                viewport.left() - item.left()
            } else if item.right() + cur_x > viewport.right() {
                viewport.right() - item.right()
            } else {
                cur_x // already fully in view
            }
        } else {
            self.group_tabs_scroll.scroll_to_item(ix);
            return;
        };

        if (to_x - cur_x).abs() < px(0.5) {
            return;
        }

        self.start_scroll_x_anim(to_x);
    }

    /// Animate `group_tabs_scroll` from its current x to `to_x`.
    pub fn start_scroll_x_anim(&mut self, to_x: openframe::Pixels) {
        use std::time::Instant;
        let from_x = self.group_tabs_scroll.offset().x;
        if (to_x - from_x).abs() < px(0.5) {
            return;
        }
        self.tab_scroll_anim = Some(super::TabScrollAnim {
            start: Instant::now(),
            from_x: f32::from(from_x),
            to_x: f32::from(to_x),
        });
    }

    #[cfg(any(feature = "gui", feature = "ios-gui"))]
    pub fn ensure_text_caret_blink_task(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.text_caret_blink_task_started {
            return;
        }
        self.text_caret_blink_task_started = true;
        cx.spawn_in(
            window,
            move |view: openframe::WeakEntity<ArcadiaRoot>,
                  cx: &mut openframe::AsyncWindowContext| {
                let mut cx = cx.clone();
                async move {
                    loop {
                        Timer::after(Duration::from_millis(500)).await;
                        let should_stop = cx
                            .update(|window, app| {
                                view.update(app, |this, cx| {
                                    this.text_caret_blink_visible = !this.text_caret_blink_visible;
                                    if this.any_text_input_focused(window, cx) {
                                        cx.notify();
                                    }
                                    false
                                })
                                .unwrap_or(true)
                            })
                            .unwrap_or(true);
                        if should_stop {
                            break;
                        }
                    }
                }
            },
        )
        .detach();
    }

    #[cfg(any(feature = "gui", feature = "ios-gui"))]
    pub fn ensure_remote_revision_poll_task(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.remote_revision_poll_started {
            return;
        }
        if self.remote_route.is_none() {
            return;
        }
        self.remote_revision_poll_started = true;
        cx.spawn_in(
            window,
            move |view: openframe::WeakEntity<ArcadiaRoot>,
                  cx: &mut openframe::AsyncWindowContext| {
                let mut cx = cx.clone();
                async move {
                    loop {
                        Timer::after(Duration::from_secs(12)).await;
                        let stop = cx
                            .update(|_, app| {
                                view.update(app, |this, cx| {
                                    if this.remote_route.is_none() {
                                        this.remote_revision_poll_started = false;
                                        return true;
                                    }
                                    let Some(route) = this.remote_route.clone() else {
                                        this.remote_revision_poll_started = false;
                                        return true;
                                    };
                                    let ctx = modules::ExecutionContext {
                                        net_as: Some(route),
                                        net_timeout_ms: Some(8000),
                                        ..Default::default()
                                    };
                                    if let Ok(Some(json)) =
                                        modules::execute_command("surface.revision", &[], &ctx)
                                    {
                                        if let Some(rev) = parse_surface_revision(&json) {
                                            if let Some(last) = this.last_surface_revision {
                                                if rev != last {
                                                    this.remote_surface_stale = true;
                                                    cx.notify();
                                                }
                                            }
                                        }
                                    }
                                    false
                                })
                                .unwrap_or(true)
                            })
                            .unwrap_or(true);
                        if stop {
                            break;
                        }
                    }
                }
            },
        )
        .detach();
    }

    pub fn toggle_settings_pin(&mut self, page_id: &str) {
        if let Some(i) = self.pinned_settings_pages.iter().position(|p| p == page_id) {
            self.pinned_settings_pages.remove(i);
        } else {
            self.pinned_settings_pages.push(page_id.to_string());
        }
        let mut cfg =
            arcadia_core::config::ui_prefs::UiPrefsConfig::load_or_create().unwrap_or_default();
        cfg.pinned_settings_pages = self.pinned_settings_pages.clone();
        if let Err(e) = cfg.save() {
            eprintln!("ui-prefs save failed: {e}");
        }
    }
}

pub(crate) fn built_in_styles() -> Vec<StyleInfo> {
    vec![StyleInfo {
        name: "default".to_string(),
        label: "Default".to_string(),
        description: "Standard GPU-rendered appearance.".to_string(),
        glyph: None,
        glyph_light: None,
        module_name: None,
    }]
}

pub(crate) fn merged_available_styles() -> Vec<StyleInfo> {
    let mut styles = built_in_styles();
    for style in arcadia_core::modules::python_registry::list_styles() {
        styles.retain(|s| s.name != style.name);
        styles.push(style);
    }
    styles.sort_by(|a, b| {
        a.label
            .to_ascii_lowercase()
            .cmp(&b.label.to_ascii_lowercase())
            .then_with(|| a.name.cmp(&b.name))
    });
    styles
}

/// Convert a `GlyphParams` (hex strings from the Python extension) into a
/// `GlyphStyleConfig` (Rgba values) the Desktop rendering layer can use.
pub(crate) fn build_glyph_style_config(
    p: &arcadia_core::modules::python_registry::GlyphParams,
) -> GlyphStyleConfig {
    let hex = |s: &Option<String>, default: Rgba| -> Rgba {
        s.as_deref().and_then(parse_hex_color).unwrap_or(default)
    };
    let fallback_dark = Rgba {
        r: 0.067,
        g: 0.067,
        b: 0.067,
        a: 1.0,
    };

    let chars: [char; 7] = p
        .border_chars
        .as_deref()
        .map(|s| {
            let v: Vec<char> = s.chars().collect();
            if v.len() >= 7 {
                [v[0], v[1], v[2], v[3], v[4], v[5], v[6]]
            } else {
                ['┌', '─', '┐', '│', '└', '─', '┘']
            }
        })
        .unwrap_or(['┌', '─', '┐', '│', '└', '─', '┘']);

    GlyphStyleConfig {
        bg: hex(
            &p.bg,
            Rgba {
                r: 0.047,
                g: 0.047,
                b: 0.047,
                a: 1.0,
            },
        ),
        surface: hex(&p.surface, fallback_dark),
        surface2: hex(
            &p.surface2,
            Rgba {
                r: 0.102,
                g: 0.102,
                b: 0.102,
                a: 1.0,
            },
        ),
        text: hex(
            &p.text,
            Rgba {
                r: 0.831,
                g: 0.831,
                b: 0.831,
                a: 1.0,
            },
        ),
        dim: hex(
            &p.dim,
            Rgba {
                r: 0.333,
                g: 0.333,
                b: 0.333,
                a: 1.0,
            },
        ),
        border: hex(
            &p.border,
            Rgba {
                r: 0.200,
                g: 0.200,
                b: 0.200,
                a: 1.0,
            },
        ),
        accent: hex(
            &p.accent,
            Rgba {
                r: 0.0,
                g: 0.800,
                b: 0.533,
                a: 1.0,
            },
        ),
        border_radius: p.border_radius,
        border_chars: chars,
    }
}

pub(crate) fn parse_hex_color(hex: &str) -> Option<Rgba> {
    let hex = hex.trim_start_matches('#');
    if hex.len() < 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
    Some(Rgba { r, g, b, a: 1.0 })
}
