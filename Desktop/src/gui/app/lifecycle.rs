use std::env;
#[cfg(feature = "gui")]
use std::path::PathBuf;
use std::time::Duration;

use arcadia_core::config::late::LateConfig;
use arcadia_core::config::modules::{
    ModulesConfig, LAN_MODULE_NAME, REMOTE_SESSION_MODULE_NAME,
};
#[cfg(feature = "gui")]
use arcadia_core::config::modules::{TERMINAL_MODULE_NAME, TERMINAL_MOTD_MODULE_NAME};
#[cfg(feature = "python-extensions")]
use arcadia_core::config::modules::PYTHON_HOST_MODULE_NAME;
use arcadia_core::config::thin_client::ThinClientConfig;
use arcadia_core::config::ConfigFile;
use arcadia_core::modules;
#[cfg(feature = "gui")]
use arcadia_core::modules::shell_motd;
use arcadia_core::modules::surface::parse_surface_snapshot;
use arcadia_core::navigation;
use openframe::{Context, Timer, Window};

#[cfg(feature = "gui")]
use super::super::tui;
use super::ArcadiaRoot;
#[cfg(feature = "gui")]
use super::{ShellMode, TerminalInstance};

#[cfg(feature = "gui")]
impl TerminalInstance {
    pub(crate) fn new(
        id: usize,
        label: String,
        shell_working_dir: PathBuf,
        shell_display_cwd: String,
        initial_history: Vec<String>,
    ) -> Self {
        Self {
            id,
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
        let history = Self::initial_shell_history();
        self.active_terminal_mut().reset(history, working_dir, display_cwd);
    }

    #[cfg(feature = "gui")]
    pub(super) fn sync_shell_display_cwd_from_env(&mut self) {
        let (working_dir, display_cwd) = Self::current_dir_strings();
        let term = self.active_terminal_mut();
        term.shell_working_dir = working_dir;
        term.shell_display_cwd = display_cwd;
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
            Err(_) => (
                PathBuf::from("/"),
                "cwd: unavailable".to_string(),
            ),
        }
    }

    #[cfg(feature = "gui")]
    fn initial_shell_history() -> Vec<String> {
        let Ok(cfg) = ModulesConfig::load_or_create() else {
            return vec!["Arcadia Terminal ready.".to_string()];
        };
        let shell_on = cfg.modules.get(TERMINAL_MODULE_NAME).copied().unwrap_or(false);
        let motd_on = cfg
            .modules
            .get(TERMINAL_MOTD_MODULE_NAME)
            .copied()
            .unwrap_or(false);
        if shell_on && motd_on {
            let mut lines = shell_motd::motd_lines();
            lines.push(String::new());
            lines
        } else {
            vec!["Arcadia Terminal ready.".to_string()]
        }
    }

    pub fn new(cx: &mut openframe::Context<Self>) -> Self {
        #[cfg(feature = "gui")]
        let shell_focus = cx.focus_handle();
        let late_compose_focus = cx.focus_handle();
        let late_settings_server_url_focus = cx.focus_handle();
        let late_settings_username_focus = cx.focus_handle();
        let late_settings_default_room_focus = cx.focus_handle();
        let late_cfg = LateConfig::load_or_create().unwrap_or_default();
        let module_rows = ModulesConfig::load_or_create()
            .map(|cfg| cfg.modules.into_iter().collect::<Vec<(String, bool)>>())
            .unwrap_or_default();
        #[cfg(feature = "gui")]
        let first_terminal = {
            let (shell_working_dir, shell_display_cwd) = Self::current_dir_strings();
            let initial_history = Self::initial_shell_history();
            TerminalInstance::new(
                0,
                "Terminal 1".to_string(),
                shell_working_dir,
                shell_display_cwd,
                initial_history,
            )
        };
        let mut root = ArcadiaRoot {
            title: openframe::SharedString::new_static("Arcadia"),
            active_page_id: navigation::DEFAULT_PAGE_ID.to_string(),
            active_group_id: navigation::DEFAULT_GROUP_ID.to_string(),
            module_rows,
            python_extension_rows: Vec::new(),
            pending_module_enable: None,
            #[cfg(feature = "gui")]
            terminals: vec![first_terminal],
            #[cfg(feature = "gui")]
            active_terminal_id: 0,
            #[cfg(feature = "gui")]
            next_terminal_serial: 2,
            #[cfg(feature = "gui")]
            terminal_context_menu_open: false,
            #[cfg(feature = "gui")]
            terminal_kill_menu: None,
            #[cfg(feature = "gui")]
            context_menu_position: openframe::Point::default(),
            #[cfg(feature = "gui")]
            shell_focus,
            late_compose_focus,
            #[cfg(feature = "gui")]
            shell_caret_visible: true,
            #[cfg(feature = "gui")]
            shell_caret_task_started: false,
            splash_elapsed_ms: 0.0,
            splash_tick_started: false,
            sidebar_visible: true,
            settings_hub_expanded: false,
            app_menu_open: false,
            session_route_menu_open: false,
            remote_route: None,
            remote_nav: None,
            surface_client_id: ThinClientConfig::load_surface_client_id(),
            last_surface_revision: None,
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
        };

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
            root.python_extension_rows =
                arcadia_core::modules::python_registry::list_modules();
        }

        root
    }

    pub fn reload_python_extensions(&mut self) {
        self.python_extension_rows =
            arcadia_core::modules::python_registry::list_modules();
    }

    pub fn reload_modules(&mut self) {
        if let Some(ref route) = self.remote_route {
            let ctx = modules::ExecutionContext {
                net_as: Some(route.clone()),
                net_timeout_ms: None,
            };
            match modules::execute_command("surface.snapshot", &[], &ctx) {
                Ok(Some(json)) => {
                    let parsed = parse_surface_snapshot(&json);
                    self.module_rows = parsed.modules;
                    self.remote_nav = parsed.navigation_registry;
                    self.last_surface_revision = Some(parsed.revision);
                }
                _ => {
                    self.module_rows = Vec::new();
                    self.remote_nav = None;
                    self.last_surface_revision = None;
                }
            }
        } else {
            self.remote_nav = None;
            self.last_surface_revision = None;
            self.module_rows = ModulesConfig::load_or_create()
                .map(|cfg| cfg.modules.into_iter().collect())
                .unwrap_or_default();
        }
        self.ensure_valid_navigation_selection();
    }

    #[cfg(feature = "gui")]
    pub fn create_new_terminal(&mut self) {
        let serial = self.next_terminal_serial;
        self.next_terminal_serial += 1;
        let new_id = self.terminals.len();
        let (working_dir, display_cwd) = Self::current_dir_strings();
        let new_term = TerminalInstance::new(
            new_id,
            format!("Terminal {serial}"),
            working_dir,
            display_cwd,
            vec!["Arcadia Terminal ready.".to_string()],
        );
        self.terminals.push(new_term);
        self.active_terminal_id = new_id;
        self.active_page_id = "utility.shell".to_string();
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
            move |view: openframe::WeakEntity<ArcadiaRoot>, cx: &mut openframe::AsyncWindowContext| {
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

    #[cfg(feature = "gui")]
    pub fn ensure_shell_caret_task(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.shell_caret_task_started {
            return;
        }
        self.shell_caret_task_started = true;
        cx.spawn_in(
            window,
            move |view: openframe::WeakEntity<ArcadiaRoot>, cx: &mut openframe::AsyncWindowContext| {
                let mut cx = cx.clone();
                async move {
                    loop {
                        Timer::after(Duration::from_millis(500)).await;
                        let should_stop = cx
                            .update(|_, app| {
                                view.update(app, |this, cx| {
                                    if !this.is_module_enabled(TERMINAL_MODULE_NAME) {
                                        this.shell_caret_task_started = false;
                                        return true;
                                    }
                                    this.shell_caret_visible = !this.shell_caret_visible;
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
}
