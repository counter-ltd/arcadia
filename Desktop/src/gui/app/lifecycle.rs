use std::env;
use std::path::PathBuf;
use std::time::Duration;

use arcadia_core::config::modules::{
    ModulesConfig, LAN_MODULE_NAME, REMOTE_SESSION_MODULE_NAME, TERMINAL_MODULE_NAME,
    TERMINAL_MOTD_MODULE_NAME,
};
use arcadia_core::config::thin_client::ThinClientConfig;
use arcadia_core::config::ConfigFile;
use arcadia_core::modules;
use arcadia_core::modules::shell_motd;
use arcadia_core::modules::surface::parse_surface_snapshot;
use arcadia_core::navigation;
use gpui::{Context, Timer, Window};

use super::super::tui;
use super::ArcadiaRoot;

impl ArcadiaRoot {
    pub(super) fn reset_shell_state(&mut self) {
        self.shell_stream_nonce = self.shell_stream_nonce.wrapping_add(1);
        self.shell_history = Self::initial_shell_history();
        self.shell_input.clear();
        self.shell_cursor = 0;
        self.shell_history_index = None;
        self.shell_output_scroll.scroll_to_bottom();
        self.sync_shell_display_cwd_from_env();
    }

    pub(super) fn sync_shell_display_cwd_from_env(&mut self) {
        match env::current_dir() {
            Ok(path) => {
                self.shell_working_dir = path.clone();
                self.shell_display_cwd = path
                    .into_os_string()
                    .into_string()
                    .unwrap_or_else(|_| "cwd: unavailable".to_string());
            }
            Err(_) => {
                self.shell_display_cwd = "cwd: unavailable".to_string();
            }
        }
    }

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

    pub fn new(cx: &mut gpui::Context<Self>) -> Self {
        let shell_focus = cx.focus_handle();
        let late_compose_focus = cx.focus_handle();
        let module_rows = ModulesConfig::load_or_create()
            .map(|cfg| cfg.modules.into_iter().collect::<Vec<(String, bool)>>())
            .unwrap_or_default();
        let shell_working_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
        let shell_display_cwd = shell_working_dir
            .clone()
            .into_os_string()
            .into_string()
            .unwrap_or_else(|_| "cwd: unavailable".to_string());
        let mut root = ArcadiaRoot {
            title: gpui::SharedString::new_static("Arcadia"),
            active_page_id: navigation::DEFAULT_PAGE_ID.to_string(),
            active_group_id: navigation::DEFAULT_GROUP_ID.to_string(),
            module_rows,
            pending_module_enable: None,
            shell_history: Self::initial_shell_history(),
            shell_input: String::new(),
            shell_focus,
            late_compose_focus,
            shell_cursor: 0,
            shell_command_history: Vec::new(),
            shell_history_index: None,
            shell_caret_visible: true,
            shell_caret_task_started: false,
            shell_stream_nonce: 0,
            shell_output_scroll: gpui::ScrollHandle::new(),
            tui_scroll: gpui::ScrollHandle::new(),
            shell_mode: super::ShellMode::Generic,
            shell_working_dir,
            shell_display_cwd,
            tui_session: None,
            tui_nonce: 0,
            tui_ready: false,
            tui_cols: tui::DEFAULT_COLS,
            tui_rows: tui::DEFAULT_ROWS,
            splash_elapsed_ms: 0.0,
            splash_tick_started: false,
            sidebar_visible: true,
            app_menu_open: false,
            session_route_menu_open: false,
            remote_route: None,
            remote_nav: None,
            surface_client_id: ThinClientConfig::load_surface_client_id(),
            last_surface_revision: None,
            lan_discovered_peers: Vec::new(),
            lan_command_feedback: String::new(),
            lan_service_feedback: String::new(),
            pending_lan_port_kill_prompt: None,
            lan_poll_task_started: false,
            late_poll_task_started: false,
            late_last_revision: 0,
            late_active_room: 1,
            late_compose_text: String::new(),
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

        root
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
            move |view: gpui::WeakEntity<ArcadiaRoot>, cx: &mut gpui::AsyncWindowContext| {
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

    pub fn ensure_shell_caret_task(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.shell_caret_task_started {
            return;
        }
        self.shell_caret_task_started = true;
        cx.spawn_in(
            window,
            move |view: gpui::WeakEntity<ArcadiaRoot>, cx: &mut gpui::AsyncWindowContext| {
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
