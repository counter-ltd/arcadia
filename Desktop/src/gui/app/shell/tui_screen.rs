use openframe::Rgba;
use openframe::{
    div, px, Context, Div, FontWeight, InteractiveElement, ParentElement,
    StatefulInteractiveElement, Styled,
};

use super::super::super::tui::{self};
use super::super::ArcadiaRoot;

/// Pixel row height; must stay aligned with `shell/execute.rs` `CHAR_H` (PTY rows × this ≈ panel).
const TUI_ROW_HEIGHT: openframe::Pixels = px(18.);

impl ArcadiaRoot {
    pub(crate) fn render_tui_screen(&self, is_dark: bool, cx: &mut Context<Self>) -> Div {
        let term = self.active_terminal();
        let Some(session) = &term.tui_session else {
            return div();
        };
        let Ok(parser) = session.parser.lock() else {
            return div();
        };
        let screen = parser.screen();
        let (rows, cols) = screen.size();
        let (cur_row, cur_col) = screen.cursor_position();

        // Snapshot screen data before releasing the lock.
        let mut snapshot: Vec<Vec<(String, Rgba, Rgba, bool)>> = Vec::with_capacity(rows as usize);
        for r in 0..rows {
            let mut row = Vec::with_capacity(cols as usize);
            for c in 0..cols {
                let is_cursor = r == cur_row && c == cur_col;
                let (ch, mut fg, mut bg, bold) = match screen.cell(r, c) {
                    Some(cell) => {
                        let content = cell.contents();
                        let text = if content.is_empty() {
                            " ".to_string()
                        } else {
                            content.to_string()
                        };
                        let fg = tui::vt_color(cell.fgcolor(), true, is_dark);
                        let bg = tui::vt_color(cell.bgcolor(), false, is_dark);
                        (text, fg, bg, cell.bold())
                    }
                    None => (
                        " ".to_string(),
                        tui::default_fg(is_dark),
                        tui::default_bg(is_dark),
                        false,
                    ),
                };
                if is_cursor {
                    std::mem::swap(&mut fg, &mut bg);
                }
                row.push((ch, fg, bg, bold));
            }
            snapshot.push(row);
        }
        drop(parser); // release lock before building elements

        let term_bg = tui::default_bg(is_dark);

        let row_els: Vec<_> = snapshot
            .into_iter()
            .map(|row_cells| {
                // Run-length encode consecutive same-style cells.
                let mut runs: Vec<(String, Rgba, Rgba, bool)> = Vec::new();
                for (ch, fg, bg, bold) in row_cells {
                    let same = runs
                        .last()
                        .map(|(_, rf, rb, rb2)| {
                            tui::rgba_eq(*rf, fg) && tui::rgba_eq(*rb, bg) && *rb2 == bold
                        })
                        .unwrap_or(false);
                    if same {
                        runs.last_mut().unwrap().0.push_str(&ch);
                    } else {
                        runs.push((ch, fg, bg, bold));
                    }
                }
                let spans: Vec<_> = runs
                    .into_iter()
                    .map(|(text, fg, bg, bold)| {
                        div()
                            .font_family("monospace")
                            .text_sm()
                            .text_color(fg)
                            .bg(bg)
                            .font_weight(if bold {
                                FontWeight::BOLD
                            } else {
                                FontWeight::NORMAL
                            })
                            .child(text)
                    })
                    .collect();
                div()
                    .flex()
                    .flex_row()
                    .items_start()
                    .h(TUI_ROW_HEIGHT)
                    .children(spans)
            })
            .collect();

        let tui_scroll = &self.active_terminal().tui_scroll;

        div()
            .w_full()
            .h_full()
            .bg(term_bg)
            .px_2()
            .pt_2()
            .pb_2()
            .flex()
            .flex_col()
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
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .id("arcadia-tui-scroll")
                    .overflow_y_scroll()
                    .track_scroll(tui_scroll)
                    .child(div().w_full().flex().flex_col().children(row_els)),
            )
    }
}
