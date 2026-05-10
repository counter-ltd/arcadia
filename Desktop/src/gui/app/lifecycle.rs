use std::env;
#[cfg(feature = "gui")]
use std::path::PathBuf;
use std::time::Duration;

use arcadia_core::config::appearance::AppearanceConfig;
use arcadia_core::config::extension_tokens;
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
use arcadia_core::modules::python_registry::StyleInfo;
#[cfg(feature = "gui")]
use arcadia_core::modules::shell_motd;
use arcadia_core::modules::surface::parse_surface_snapshot;
use arcadia_core::navigation;
use openframe::{Context, Rgba, RenderStyle, Timer, UpdateGlobal, Window};
use crate::gui::theme::{
    ActiveGlyphBorderPatterns, ActiveGlyphBorderTypography, ActiveGlyphStyle, GlyphBorderPatterns,
    ActiveGlyphUiFontFamily, GlyphBorderTypography, GlyphStyleConfig,
};

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
        let (history, motd_n) = Self::initial_shell_history(self.current_color_scheme_dark());
        self.shell_motd_prefix_lines = motd_n;
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
    fn initial_shell_history(is_dark: bool) -> (Vec<String>, usize) {
        let Ok(cfg) = ModulesConfig::load_or_create() else {
            return (vec!["Arcadia Terminal ready.".to_string()], 0);
        };
        let shell_on = cfg.modules.get(TERMINAL_MODULE_NAME).copied().unwrap_or(false);
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
        let shell_on = cfg.modules.get(TERMINAL_MODULE_NAME).copied().unwrap_or(false);
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
        let late_compose_focus = cx.focus_handle();
        let late_settings_server_url_focus = cx.focus_handle();
        let late_settings_username_focus = cx.focus_handle();
        let late_settings_default_room_focus = cx.focus_handle();
        let extension_token_focus = cx.focus_handle();
        let late_cfg = LateConfig::load_or_create().unwrap_or_default();
        let module_rows = ModulesConfig::load_or_create()
            .map(|cfg| cfg.modules.into_iter().collect::<Vec<(String, bool)>>())
            .unwrap_or_default();
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
                0,
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
            module_rows,
            python_extension_rows: Vec::new(),
            active_style,
            available_styles,
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
            extension_token_values: std::collections::HashMap::new(),
            extension_token_editing: None,
            extension_token_focus,
            color_picker_modal: None,
            last_color_scheme_dark: None,
            #[cfg(feature = "gui")]
            shell_motd_prefix_lines: initial_motd_n,
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
            // Merge styles from Python extensions into available_styles.
            root.available_styles = merged_available_styles();
        }

        root.refresh_extension_token_cache();

        // Apply persisted style (may activate a glyph config from an extension).
        let active = root.active_style.clone();
        root.apply_style(active, true, cx);

        root
    }

    pub(super) fn refresh_extension_token_cache(&mut self) {
        self.extension_token_values.clear();
        for (module, specs) in arcadia_core::modules::python_registry::list_style_tokens() {
            let file = extension_tokens::load_module_tokens(&module).unwrap_or_default();
            for spec in specs {
                let v = extension_tokens::merged_display_for_key(
                    &spec.key,
                    &spec.default_value,
                    &file,
                );
                self.extension_token_values
                    .insert((module.clone(), spec.key), v);
            }
        }
    }

    pub(crate) fn flush_extension_token_edit(&mut self, module: String, key: String, cx: &mut Context<Self>) {
        let kind = arcadia_core::modules::python_registry::list_style_tokens()
            .into_iter()
            .find(|(m, _)| m == &module)
            .and_then(|(_, specs)| specs.into_iter().find(|s| s.key == key))
            .map(|s| s.kind);
        let Some(kind) = kind else {
            return;
        };
        let pair = (module.clone(), key.clone());
        let Some(raw) = self.extension_token_values.get(&pair).cloned() else {
            return;
        };
        let mut file = extension_tokens::load_module_tokens(&module).unwrap_or_default();
        match extension_tokens::parse_input_to_value(kind, &raw) {
            Ok(val) => {
                file.insert(key.clone(), val.clone());
                if extension_tokens::save_module_tokens(&module, &file).is_ok() {
                    let disp = extension_tokens::value_to_display_string(&val);
                    self.extension_token_values.insert(pair, disp);
                    self.apply_style(self.active_style.clone(), self.current_color_scheme_dark(), cx);
                }
            }
            Err(_) => {}
        }
    }

    pub fn reload_python_extensions(&mut self, cx: &mut Context<Self>) {
        self.python_extension_rows =
            arcadia_core::modules::python_registry::list_modules();
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

        let glyph_cfg = merged_glyph
            .as_ref()
            .map(|p| build_glyph_style_config(p));

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
        let (hist, _) = Self::initial_shell_history(self.current_color_scheme_dark());
        let new_term = TerminalInstance::new(
            new_id,
            format!("Terminal {serial}"),
            working_dir,
            display_cwd,
            hist,
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
    let fallback_dark = Rgba { r: 0.067, g: 0.067, b: 0.067, a: 1.0 };

    let chars: [char; 7] = p.border_chars.as_deref()
        .map(|s| {
            let v: Vec<char> = s.chars().collect();
            if v.len() >= 7 { [v[0], v[1], v[2], v[3], v[4], v[5], v[6]] }
            else { ['┌', '─', '┐', '│', '└', '─', '┘'] }
        })
        .unwrap_or(['┌', '─', '┐', '│', '└', '─', '┘']);

    GlyphStyleConfig {
        bg:      hex(&p.bg,      Rgba { r: 0.047, g: 0.047, b: 0.047, a: 1.0 }),
        surface: hex(&p.surface, fallback_dark),
        surface2: hex(&p.surface2, Rgba { r: 0.102, g: 0.102, b: 0.102, a: 1.0 }),
        text:    hex(&p.text,    Rgba { r: 0.831, g: 0.831, b: 0.831, a: 1.0 }),
        dim:     hex(&p.dim,     Rgba { r: 0.333, g: 0.333, b: 0.333, a: 1.0 }),
        border:  hex(&p.border,  Rgba { r: 0.200, g: 0.200, b: 0.200, a: 1.0 }),
        accent:  hex(&p.accent,  Rgba { r: 0.0, g: 0.800, b: 0.533, a: 1.0 }),
        border_radius: p.border_radius,
        border_chars: chars,
    }
}

pub(crate) fn parse_hex_color(hex: &str) -> Option<Rgba> {
    let hex = hex.trim_start_matches('#');
    if hex.len() < 6 { return None; }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
    Some(Rgba { r, g, b, a: 1.0 })
}
