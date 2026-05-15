use arcadia_core::modules::ai_sandbox;
use arcadia_core::modules::ai_types::AiWorkspaceContext;
use openframe::{
    div, px, rgba, Context, FontWeight, InteractiveElement, IntoElement, MouseButton,
    ParentElement, StatefulInteractiveElement, Styled,
};

use crate::gui::app::{AiPendingEdit, ArcadiaRoot};
use crate::gui::theme;

impl ArcadiaRoot {
    pub(crate) fn ai_diff_panel(
        &mut self,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> impl IntoElement {
        let p = theme::theme_palette(cx, is_dark);
        let g_snap = theme::glyph_snapshot(cx);
        let radius = g_snap.map(|g| g.border_radius).unwrap_or(p.radius_md);
        let accent = theme::ui_accent(cx);
        let accent_fg = theme::ui_accent_fg(cx);

        let pending_count = self.ai_pending_edits.len();

        // Outer panel: fixed right side or bottom bar.
        let mut edit_list = div().flex().flex_col().gap_4().p_4();

        for (edit_idx, edit) in self.ai_pending_edits.iter().enumerate() {
            let path = edit.path.clone();
            let _all_resolved = edit
                .hunks
                .iter()
                .all(|h| edit.accepted.contains(&h.index) || edit.rejected.contains(&h.index));
            let pending_hunk_count = edit
                .hunks
                .iter()
                .filter(|h| !edit.accepted.contains(&h.index) && !edit.rejected.contains(&h.index))
                .count();

            let mut file_block = div()
                .flex()
                .flex_col()
                .gap_2()
                .p_3()
                .rounded(px(radius.min(8.0)))
                .border_1()
                .border_color(p.panel_border)
                .bg(p.panel_bg);

            // File header: path + Accept All / Reject All.
            file_block = file_block.child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .flex_1()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(p.content_title)
                            .child(path.clone()),
                    )
                    .child(div().text_xs().text_color(p.content_meta).child(
                        if pending_hunk_count == 0 {
                            "All resolved".to_string()
                        } else {
                            format!("{pending_hunk_count} hunks")
                        },
                    ))
                    .child(
                        div()
                            .px_2()
                            .py_0p5()
                            .rounded(px(radius.min(5.0)))
                            .bg(accent)
                            .text_xs()
                            .text_color(accent_fg)
                            .cursor_pointer()
                            .child("Accept All")
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, _, cx| {
                                    if let Some(edit) = this.ai_pending_edits.get_mut(edit_idx) {
                                        for h in &edit.hunks {
                                            edit.accepted.insert(h.index);
                                        }
                                    }
                                    this.apply_edit_if_resolved(edit_idx, cx);
                                    cx.notify();
                                }),
                            ),
                    )
                    .child(
                        div()
                            .px_2()
                            .py_0p5()
                            .rounded(px(radius.min(5.0)))
                            .bg(p.panel_bg)
                            .border_1()
                            .border_color(p.panel_border)
                            .text_xs()
                            .text_color(p.content_meta)
                            .cursor_pointer()
                            .child("Reject All")
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, _, cx| {
                                    if let Some(edit) = this.ai_pending_edits.get_mut(edit_idx) {
                                        for h in &edit.hunks {
                                            edit.rejected.insert(h.index);
                                        }
                                    }
                                    this.apply_edit_if_resolved(edit_idx, cx);
                                    cx.notify();
                                }),
                            ),
                    ),
            );

            // Hunk list.
            for hunk in &edit.hunks {
                let hunk_idx = hunk.index;
                let is_accepted = edit.accepted.contains(&hunk_idx);
                let is_rejected = edit.rejected.contains(&hunk_idx);

                let mut hunk_block = div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .rounded(px(radius.min(5.0)))
                    .border_1()
                    .border_color(p.panel_border)
                    .overflow_hidden();

                // Removed lines (red tint).
                for line in &hunk.orig_lines {
                    let line = line.clone();
                    hunk_block = hunk_block.child(
                        div()
                            .px_3()
                            .py_0p5()
                            .bg(removed_color(is_dark))
                            .text_xs()
                            .font_family("monospace")
                            .text_color(p.content_body)
                            .child(format!("- {line}")),
                    );
                }

                // Added lines (green tint).
                for line in &hunk.new_lines {
                    let line = line.clone();
                    hunk_block = hunk_block.child(
                        div()
                            .px_3()
                            .py_0p5()
                            .bg(added_color(is_dark))
                            .text_xs()
                            .font_family("monospace")
                            .text_color(p.content_body)
                            .child(format!("+ {line}")),
                    );
                }

                // Hunk action bar.
                let action_bar = if is_accepted {
                    div()
                        .px_3()
                        .py_1()
                        .bg(p.panel_bg)
                        .text_xs()
                        .text_color(accent)
                        .child("✓ Accepted")
                } else if is_rejected {
                    div()
                        .px_3()
                        .py_1()
                        .bg(p.panel_bg)
                        .text_xs()
                        .text_color(p.content_meta)
                        .child("✕ Rejected")
                } else {
                    div()
                        .px_3()
                        .py_1()
                        .bg(p.panel_bg)
                        .flex()
                        .flex_row()
                        .gap_2()
                        .child(
                            div()
                                .px_2()
                                .py_0p5()
                                .rounded(px(radius.min(4.0)))
                                .bg(accent)
                                .text_xs()
                                .text_color(accent_fg)
                                .cursor_pointer()
                                .child("Accept")
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |this, _, _, cx| {
                                        if let Some(edit) = this.ai_pending_edits.get_mut(edit_idx)
                                        {
                                            edit.accepted.insert(hunk_idx);
                                        }
                                        this.apply_edit_if_resolved(edit_idx, cx);
                                        cx.notify();
                                    }),
                                ),
                        )
                        .child(
                            div()
                                .px_2()
                                .py_0p5()
                                .rounded(px(radius.min(4.0)))
                                .bg(p.panel_bg)
                                .border_1()
                                .border_color(p.panel_border)
                                .text_xs()
                                .text_color(p.content_meta)
                                .cursor_pointer()
                                .child("Reject")
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |this, _, _, cx| {
                                        if let Some(edit) = this.ai_pending_edits.get_mut(edit_idx)
                                        {
                                            edit.rejected.insert(hunk_idx);
                                        }
                                        this.apply_edit_if_resolved(edit_idx, cx);
                                        cx.notify();
                                    }),
                                ),
                        )
                };

                file_block = file_block.child(hunk_block.child(action_bar));
            }

            edit_list = edit_list.child(file_block);
        }

        // Panel wrapper with close button.
        div()
            .w(px(480.))
            .flex_shrink_0()
            .h_full()
            .border_l_1()
            .border_color(p.panel_border)
            .bg(p.panel_bg)
            .flex()
            .flex_col()
            .child(
                div()
                    .flex_shrink_0()
                    .px_4()
                    .py_2()
                    .border_b_1()
                    .border_color(p.panel_border)
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .flex_1()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(p.content_title)
                            .child(format!("Pending Edits ({pending_count})")),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(p.content_meta)
                            .cursor_pointer()
                            .child("✕")
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _, cx| {
                                    this.ai_diff_panel_open = false;
                                    cx.notify();
                                }),
                            ),
                    ),
            )
            .child(
                div()
                    .id("ai-diff-panel-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .child(edit_list),
            )
    }

    /// Apply accepted hunks to disk if all hunks in an edit are resolved, then remove it.
    pub(crate) fn apply_edit_if_resolved(&mut self, edit_idx: usize, cx: &mut Context<Self>) {
        let Some(edit) = self.ai_pending_edits.get(edit_idx) else {
            return;
        };
        let all_resolved = edit.hunks.is_empty()
            || edit
                .hunks
                .iter()
                .all(|h| edit.accepted.contains(&h.index) || edit.rejected.contains(&h.index));
        if !all_resolved {
            return;
        }

        // Build the accepted content by applying accepted hunks to original.
        let final_content = build_accepted_content(edit);
        let path = edit.path.clone();
        let any_accepted = !edit.accepted.is_empty();

        // Write to disk if any hunk was accepted.
        if any_accepted {
            let workspace_ctx: Option<AiWorkspaceContext> =
                self.ai_chat_workspace_id.as_deref().and_then(|ws_id| {
                    self.workspace_entries
                        .iter()
                        .find(|w| w.id == ws_id)
                        .map(AiWorkspaceContext::from_workspace_entry)
                });
            match ai_sandbox::sandboxed_write(workspace_ctx.as_ref(), &path, &final_content) {
                Ok(_) => {}
                Err(e) => eprintln!("apply_edit: write failed: {e}"),
            }
        }

        // Remove the resolved edit.
        self.ai_pending_edits.remove(edit_idx);
        if self.ai_pending_edits.is_empty() {
            self.ai_diff_panel_open = false;
        }
        cx.notify();
    }
}

/// Apply accepted hunks to the original text, skipping rejected ones.
fn build_accepted_content(edit: &AiPendingEdit) -> String {
    let orig_lines: Vec<&str> = edit.original.lines().collect();
    let mut result: Vec<String> = orig_lines.iter().map(|l| l.to_string()).collect();

    // Walk hunks in reverse index order so line numbers stay valid after replacements.
    let mut sorted_hunks: Vec<_> = edit.hunks.iter().collect();
    sorted_hunks.sort_by(|a, b| b.index.cmp(&a.index));

    for hunk in sorted_hunks {
        if !edit.accepted.contains(&hunk.index) {
            continue;
        }
        // For accepted hunks: replace orig_lines range with new_lines.
        // Since we track orig_lines and new_lines per hunk but not positions,
        // find the first occurrence of the orig_lines block and replace it.
        if let Some(start) = find_lines_start(&result, &hunk.orig_lines) {
            let end = start + hunk.orig_lines.len();
            result.splice(start..end, hunk.new_lines.iter().cloned());
        } else if hunk.orig_lines.is_empty() && !hunk.new_lines.is_empty() {
            // Pure insertion — append.
            result.extend(hunk.new_lines.iter().cloned());
        }
    }

    result.join("\n")
}

fn find_lines_start(haystack: &[String], needle: &[String]) -> Option<usize> {
    if needle.is_empty() {
        return None;
    }
    for i in 0..haystack.len().saturating_sub(needle.len() - 1) {
        if haystack[i..i + needle.len()]
            .iter()
            .zip(needle.iter())
            .all(|(h, n)| h == n)
        {
            return Some(i);
        }
    }
    None
}

fn removed_color(is_dark: bool) -> openframe::Rgba {
    if is_dark {
        rgba(0x3d1515ff)
    } else {
        rgba(0xfff0f0ff)
    }
}

fn added_color(is_dark: bool) -> openframe::Rgba {
    if is_dark {
        rgba(0x143d14ff)
    } else {
        rgba(0xf0fff0ff)
    }
}
