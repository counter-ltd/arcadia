use openframe::{
    div, px, rgb, Context, FontWeight, InteractiveElement, IntoElement, MouseButton,
    ParentElement, StatefulInteractiveElement, Styled, Window, WindowAppearance,
};

use crate::gui::tui::shell_history_line;
use crate::gui::theme;

use super::super::ArcadiaRoot;
use super::super::text_input_caret::TEXT_INPUT_CARET_CHAR;

impl ArcadiaRoot {
    pub(crate) fn shell_panel(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        if self.active_page_id.as_str() != "utility.shell" {
            return div();
        }

        let is_dark = matches!(
            window.appearance(),
            WindowAppearance::Dark | WindowAppearance::VibrantDark
        );

        if self.terminal_show_dashboard {
            return self.terminal_dashboard(cx, is_dark);
        }
        let is_focused = self.shell_focus.is_focused(window);
        let shell_bg = theme::ui_surface(cx, is_dark);
        let shell_border = theme::ui_border(cx, is_dark);
        let shell_radius = theme::ui_radius(cx);
        let shell_accent = theme::ui_accent(cx);
        let term = self.active_terminal();

        // Live PTY: vt100 grid fills the panel (transcript returns after the process exits).
        if term.tui_session.is_some() && term.tui_ready {
            return div()
                .w_full()
                .h_full()
                .overflow_hidden()
                .p_1()
                .rounded(px(shell_radius))
                .bg(shell_bg)
                .border_1()
                .border_color(shell_border)
                .flex()
                .flex_col()
                .child(
                    div()
                        .flex_1()
                        .min_h_0()
                        .w_full()
                        .child(self.render_tui_screen(is_dark, cx)),
                );
        }

        div()
            .w_full()
            .h_full()
            .overflow_hidden()
            .p_1()
            .rounded(px(shell_radius))
            .bg(shell_bg)
            .border_1()
            .border_color(shell_border)
            .flex()
            .flex_col()
            .gap_0()
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .min_h_0()
                    .id("arcadia-shell-output")
                    .overflow_y_scroll()
                    .track_scroll(&term.shell_output_scroll)
                    .child(
                        div().w_full().p_3().flex().flex_col().gap_0().children(
                            term.shell_history
                                .iter()
                                .filter(|line| !line.is_empty())
                                .map(|line| shell_history_line(line, is_dark)),
                        ),
                    ),
            )
            .child(
                div()
                    .w_full()
                    .flex_shrink_0()
                    .px_3()
                    .py_2()
                    .flex()
                    .gap_2()
                    .items_center()
                    .border_t_1()
                    .border_color(if is_focused {
                        shell_accent
                    } else {
                        shell_border
                    })
                    .bg(theme::ui_bg(cx, is_dark))
                    .track_focus(&self.shell_focus)
                    .on_mouse_down(
                        openframe::MouseButton::Left,
                        cx.listener(|this, _, window, _| {
                            this.shell_focus.focus(window);
                        }),
                    )
                    .on_key_down(cx.listener(Self::handle_shell_key_down))
                    .child(
                        div()
                            .text_sm()
                            .text_color(shell_accent)
                            .child("$"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme::ui_text(cx, is_dark))
                            .child(self.shell_input_with_cursor(is_focused)),
                    ),
            )
    }

    pub(crate) fn shell_input_with_cursor(&self, is_focused: bool) -> String {
        let term = self.active_terminal();
        let chars = term.shell_input.chars().collect::<Vec<_>>();
        let cursor = term.shell_cursor.min(chars.len());
        let mut out = String::with_capacity(chars.len() + 1);
        for (idx, ch) in chars.iter().enumerate() {
            if idx == cursor && is_focused && self.text_caret_blink_visible {
                out.push(TEXT_INPUT_CARET_CHAR);
            }
            out.push(*ch);
        }
        if cursor == chars.len() && is_focused && self.text_caret_blink_visible {
            out.push(TEXT_INPUT_CARET_CHAR);
        }
        out
    }

    pub(crate) fn shell_working_directory_label(&self) -> String {
        // Always use per-terminal logical cwd (updated after each `spawn_tui_command` and while
        // a live PTY runs). `env::current_dir()` is the GUI process — wrong between transcript prompts.
        self.active_terminal().shell_display_cwd.clone()
    }

    pub(crate) fn terminal_dashboard(
        &mut self,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> openframe::Div {
        let p = theme::theme_palette(cx, is_dark);
        let pal = theme::nav_accent_palette("emerald", is_dark);
        let r = p.radius_md.min(12.0);
        let term_bg_preview = if is_dark { rgb(0x0d1117) } else { rgb(0xf0f2f5) };

        let terminal_count = self.terminals.len();
        let cards: Vec<openframe::AnyElement> = (0..terminal_count)
            .map(|idx| {
                let label = self.terminals[idx].label.clone();
                let cwd   = self.terminals[idx].shell_display_cwd.clone();
                let is_active_term = self.active_terminal_id == idx;
                let preview_lines: Vec<String> = self.terminals[idx]
                    .shell_history
                    .iter()
                    .map(|l| strip_ansi(l))
                    .filter(|l| is_meaningful_line(l))
                    .rev()
                    .take(6)
                    .map(|l| l.chars().take(52).collect::<String>())
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect();
                let border_col = if is_active_term { pal.icon_active } else { p.panel_border };
                div()
                    .flex_1()
                    .min_w(px(220.))
                    .cursor_pointer()
                    .rounded(px(r))
                    .bg(p.panel_bg)
                    .border_1()
                    .border_color(border_col)
                    .hover(move |s| s.bg(p.row_bg).border_color(pal.row_hover))
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.active_terminal_id = idx;
                            this.active_page_id = "utility.shell".to_string();
                            this.terminal_show_dashboard = false;
                            cx.notify();
                        }),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_h(px(100.))
                            .bg(term_bg_preview)
                            .p_2()
                            .overflow_hidden()
                            .flex()
                            .flex_col()
                            .gap_0()
                            .children(preview_lines.into_iter().map(|line| {
                                div()
                                    .text_xs()
                                    .font_family("monospace")
                                    .text_color(p.content_meta)
                                    .flex_shrink_0()
                                    .child(if line.is_empty() { " ".to_string() } else { line })
                                    .into_any_element()
                            })),
                    )
                    .child(
                        div()
                            .p_3()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .border_t_1()
                            .border_color(p.panel_border)
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(p.content_title)
                                    .child(label),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(p.content_meta)
                                    .child(cwd),
                            ),
                    )
                    .into_any_element()
            })
            .collect();

        div()
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .child(
                div()
                    .id("terminal-dashboard-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .p_6()
                    .flex()
                    .flex_col()
                    .gap_8()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_4()
                            .child(
                                div()
                                    .text_xl()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(p.content_title)
                                    .child("Terminals"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .flex_wrap()
                                    .gap_4()
                                    .children(cards)
                                    .children(
                                        (0..5).map(|_| div().flex_1().min_w(px(220.)).into_any_element()).collect::<Vec<_>>()
                                    ),
                            ),
                    ),
            )
    }
}

fn is_meaningful_line(s: &str) -> bool {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return false;
    }
    let total: usize = trimmed.chars().count();
    let alphanumeric: usize = trimmed.chars().filter(|c| c.is_alphanumeric()).count();
    // Reject lines that are mostly box/block drawing chars (MOTD art, separators).
    total > 0 && alphanumeric * 3 >= total
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // CSI sequence: ESC [ ... letter
            if chars.peek() == Some(&'[') {
                chars.next();
                for nc in chars.by_ref() {
                    if nc.is_ascii_alphabetic() { break; }
                }
            } else {
                // Other escape sequences: consume until letter or ESC
                for nc in chars.by_ref() {
                    if nc.is_ascii_alphabetic() || nc == '\x1b' { break; }
                }
            }
        } else if is_visual_only_char(c) {
            // Replace block/box/braille chars with a space so they don't render as rectangles
            out.push(' ');
        } else {
            out.push(c);
        }
    }
    // Collapse runs of spaces left by stripped chars
    let mut result = String::with_capacity(out.len());
    let mut prev_space = false;
    for c in out.chars() {
        if c == ' ' {
            if !prev_space { result.push(' '); }
            prev_space = true;
        } else {
            result.push(c);
            prev_space = false;
        }
    }
    result.trim().to_string()
}

fn is_visual_only_char(c: char) -> bool {
    matches!(c,
        // Box drawing
        '\u{2500}'..='\u{257F}' |
        // Block elements (█ ▀ ▄ etc.)
        '\u{2580}'..='\u{259F}' |
        // Geometric shapes
        '\u{25A0}'..='\u{25FF}' |
        // Braille (used as progress bars)
        '\u{2800}'..='\u{28FF}' |
        // Misc technical
        '\u{2300}'..='\u{23FF}'
    )
}
