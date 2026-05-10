use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::time::Duration;

use arcadia_core::modules;
use openframe::{Context, Timer, Window, WindowAppearance};

use super::super::super::tui::TuiSession;
use super::super::{window_controls_top_padding, ArcadiaRoot, ShellMode};

/// Approximate character width/height for monospace text_sm (14 px font).
const CHAR_W: f32 = 8.4;
const CHAR_H: f32 = 18.0;
/// Layout overhead: shell panel p_2 (8×2) + tui_screen p_1 (4×2) + border (1×2).
const PADDING_H: f32 = 26.0;
const PADDING_V: f32 = 26.0;
/// Top-bar height: py_2 (8×2) + text_sm content (~14px) + border_b_1.
const TOP_BAR_H: f32 = 37.0;
/// Sidebar width when visible (w_64 = 256 px).
const SIDEBAR_W: f32 = 256.0;

/// Matches shell input `$` styling in `shell/panel.rs` (`theme::ui_accent` — default `#10b981`).
fn shell_history_prompt_prefix(window: &Window) -> String {
    let is_dark = matches!(
        window.appearance(),
        WindowAppearance::Dark | WindowAppearance::VibrantDark
    );
    if is_dark {
        "\x1b[38;2;16;185;129m$\x1b[0m ".to_string()
    } else {
        "\x1b[38;2;4;120;87m$\x1b[0m ".to_string()
    }
}

fn compute_tui_size(window: &Window, sidebar_visible: bool) -> (u16, u16) {
    let vp = window.viewport_size();
    let sidebar = if sidebar_visible { SIDEBAR_W } else { 0.0 };
    let chrome = window_controls_top_padding(window).to_f64() as f32;
    let w = vp.width.to_f64() as f32;
    let h = vp.height.to_f64() as f32;
    let usable_w = (w - sidebar - PADDING_H).max(CHAR_W * 40.0);
    let usable_h = (h - chrome - TOP_BAR_H - PADDING_V).max(CHAR_H * 10.0);
    ((usable_h / CHAR_H) as u16, (usable_w / CHAR_W) as u16)
}

impl ArcadiaRoot {
    fn stream_shell_command_output(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
        history_command_display: &str,
        token: &str,
        args: &[&str],
        exec_ctx: &modules::ExecutionContext,
    ) {
        let result = modules::execute_command(token, args, exec_ctx);
        let terminal_id = self.active_terminal_id;
        let term = &mut self.terminals[terminal_id];
        term.shell_stream_nonce = term.shell_stream_nonce.wrapping_add(1);
        let stream_nonce = term.shell_stream_nonce;
        term.shell_history.push(format!(
            "{}{history_command_display}",
            shell_history_prompt_prefix(window)
        ));
        term.shell_output_scroll.scroll_to_bottom();
        let output = match result {
            Ok(Some(output)) => output,
            Ok(None) => "Unknown shell command token.".to_string(),
            Err(err) => err,
        };
        self.terminals[terminal_id].shell_output_scroll.scroll_to_bottom();
        let lines: Vec<String> = output.lines().map(str::to_string).collect();
        cx.spawn_in(
            window,
            move |view: openframe::WeakEntity<ArcadiaRoot>, cx: &mut openframe::AsyncWindowContext| {
                let mut cx = cx.clone();
                async move {
                    for line in lines {
                        Timer::after(Duration::from_millis(4)).await;
                        let _ = cx.update(|_, app| {
                            let _ = view.update(app, |this, cx| {
                                if this.terminals[terminal_id].shell_stream_nonce != stream_nonce {
                                    return;
                                }
                                this.terminals[terminal_id].shell_history.push(line);
                                this.terminals[terminal_id].shell_output_scroll.scroll_to_bottom();
                                cx.notify();
                            });
                        });
                    }
                    let _ = cx.update(|_, app| {
                        let _ = view.update(app, |this, cx| {
                            if this.terminals[terminal_id].shell_stream_nonce == stream_nonce {
                                this.terminals[terminal_id].shell_output_scroll.scroll_to_bottom();
                                cx.notify();
                            }
                        });
                    });
                }
            },
        )
        .detach();
    }

    pub fn sync_tui_size(&mut self, window: &Window) {
        let terminal_id = self.active_terminal_id;
        let (rows, cols) = compute_tui_size(window, self.sidebar_visible);
        let term = &mut self.terminals[terminal_id];
        if cols != term.tui_cols || rows != term.tui_rows {
            term.tui_cols = cols;
            term.tui_rows = rows;
            if let Some(session) = &term.tui_session {
                session.resize(rows, cols);
            }
        }
    }

    pub fn run_shell_execute(
        &mut self,
        command: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let normalized = command.trim();
        if normalized.eq_ignore_ascii_case("clear") || normalized.eq_ignore_ascii_case("cls") {
            let terminal_id = self.active_terminal_id;
            let term = &mut self.terminals[terminal_id];
            term.shell_stream_nonce = term.shell_stream_nonce.wrapping_add(1);
            term.shell_history.clear();
            term.shell_output_scroll.scroll_to_bottom();
            cx.notify();
            return;
        }
        let ctx = self.execution_context();
        if self.active_terminal().shell_mode == ShellMode::Generic {
            if self.remote_route.is_some() {
                self.stream_shell_command_output(
                    window,
                    cx,
                    normalized,
                    "shell.execute",
                    &[normalized],
                    &ctx,
                );
                return;
            }
            self.spawn_tui_command(normalized, window, cx);
            return;
        }
        let command_token = self.active_terminal().shell_mode.command_token();
        self.stream_shell_command_output(
            window,
            cx,
            command,
            command_token,
            &[command],
            &ctx,
        );
    }

    fn spawn_tui_command(&mut self, command: &str, window: &mut Window, cx: &mut Context<Self>) {
        let terminal_id = self.active_terminal_id;

        // Compute size before taking mutable borrow of terminals.
        let prompt = format!("{}{command}", shell_history_prompt_prefix(window));
        let (rows, cols) = compute_tui_size(window, self.sidebar_visible);

        {
            let term = &mut self.terminals[terminal_id];
            term.tui_session = None;
            term.tui_ready = false;
            term.tui_nonce = term.tui_nonce.wrapping_add(1);
            term.tui_rows = rows;
            term.tui_cols = cols;
            term.shell_history.push(prompt);
            term.shell_output_scroll.scroll_to_bottom();
        }
        let nonce = self.terminals[terminal_id].tui_nonce;
        let cwd_at_spawn = self.terminals[terminal_id].shell_working_dir.clone();

        match TuiSession::spawn(command, rows, cols, &cwd_at_spawn) {
            Err(e) => {
                self.terminals[terminal_id].shell_history.push(format!("error: {e}"));
                self.terminals[terminal_id].shell_output_scroll.scroll_to_bottom();
                cx.notify();
            }
            Ok(session) => {
                let cwd_fallback = self.terminals[terminal_id]
                    .shell_working_dir
                    .clone()
                    .into_os_string()
                    .into_string()
                    .ok();
                self.terminals[terminal_id].shell_display_cwd = session
                    .foreground_cwd()
                    .or(cwd_fallback)
                    .unwrap_or_else(|| "cwd: unavailable".to_string());
                let parser = session.parser.clone();
                let queue = session.queue.clone();
                let done = session.done.clone();
                self.terminals[terminal_id].tui_scroll.scroll_to_bottom();
                self.terminals[terminal_id].tui_session = Some(session);
                cx.notify();

                let command_owned = command.to_string();
                cx.spawn_in(
                    window,
                    move |view: openframe::WeakEntity<ArcadiaRoot>,
                          cx: &mut openframe::AsyncWindowContext| {
                        let mut cx = cx.clone();
                        async move {
                            let mut showed_tui = false;
                            loop {
                                Timer::after(Duration::from_millis(16)).await;

                                let chunks: Vec<Vec<u8>> = queue
                                    .lock()
                                    .map(|mut q| q.drain(..).collect())
                                    .unwrap_or_default();
                                let is_done = done.load(Ordering::SeqCst);

                                let _ = cx.update(|_, app| {
                                    let _ = view.update(app, |this, cx| {
                                        if this.terminals[terminal_id].tui_nonce != nonce {
                                            return;
                                        }
                                        if let Some(ref sess) = this.terminals[terminal_id].tui_session {
                                            if let Some(cwd) = sess.foreground_cwd() {
                                                if cwd != this.terminals[terminal_id].shell_display_cwd {
                                                    this.terminals[terminal_id].shell_display_cwd = cwd.clone();
                                                    this.terminals[terminal_id].shell_working_dir = PathBuf::from(cwd);
                                                    this.terminals[terminal_id].tui_scroll.scroll_to_bottom();
                                                    cx.notify();
                                                }
                                            }
                                        }
                                    });
                                });

                                if !is_done && !showed_tui {
                                    showed_tui = true;
                                    let _ = cx.update(|_, app| {
                                        let _ = view.update(app, |this, cx| {
                                            if this.terminals[terminal_id].tui_nonce == nonce {
                                                this.terminals[terminal_id].tui_ready = true;
                                                this.terminals[terminal_id].tui_scroll.scroll_to_bottom();
                                                cx.notify();
                                            }
                                        });
                                    });
                                }

                                if !chunks.is_empty() {
                                    if let Ok(mut p) = parser.lock() {
                                        for chunk in &chunks {
                                            p.process(chunk);
                                        }
                                    }
                                    let _ = cx.update(|_, app| {
                                        let _ = view.update(app, |this, cx| {
                                            this.terminals[terminal_id].tui_scroll.scroll_to_bottom();
                                            cx.notify();
                                        });
                                    });
                                }

                                if is_done && chunks.is_empty() {
                                    let screen_lines: Vec<String> = parser
                                        .lock()
                                        .map(|p| {
                                            let screen = p.screen();
                                            let (rows, cols) = screen.size();
                                            (0..rows)
                                                .filter_map(|r| {
                                                    crate::gui::tui::vt100_row_for_shell_history(
                                                        screen, r, cols,
                                                    )
                                                })
                                                .collect()
                                        })
                                        .unwrap_or_default();
                                    let _ = cx.update(|_, app| {
                                        let _ = view.update(app, |this, cx| {
                                            if this.terminals[terminal_id].tui_nonce == nonce {
                                                for line in screen_lines {
                                                    this.terminals[terminal_id].shell_history.push(line);
                                                }
                                                if let Some(ref sess) = this.terminals[terminal_id].tui_session {
                                                    let from_fg =
                                                        sess.foreground_cwd().map(PathBuf::from);
                                                    let from_cd =
                                                        crate::gui::tui::resolve_simple_cd(
                                                            &cwd_at_spawn,
                                                            &command_owned,
                                                        );
                                                    if let Some(p) = from_fg.or(from_cd) {
                                                        this.terminals[terminal_id].shell_working_dir = p.clone();
                                                        this.terminals[terminal_id].shell_display_cwd =
                                                            p.to_string_lossy().into_owned();
                                                    }
                                                }
                                                this.terminals[terminal_id].tui_session = None;
                                                this.terminals[terminal_id].shell_output_scroll.scroll_to_bottom();
                                                cx.notify();
                                            }
                                        });
                                    });
                                    break;
                                }
                            }
                        }
                    },
                )
                .detach();
            }
        }
    }
}
