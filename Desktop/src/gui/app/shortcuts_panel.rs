use arcadia_core::config::shortcuts::ShortcutsConfig;
use arcadia_core::config::ConfigFile;
use arcadia_core::shortcuts;
use openframe::prelude::FluentBuilder as _;
use openframe::{
    AnyElement, Context, FontWeight, InteractiveElement, IntoElement, MouseButton, ParentElement,
    Styled, Window, div, px,
};

use super::shortcuts::global_shortcuts_for_settings;
use super::{ArcadiaRoot, ShortcutCreateActionKind, ShortcutCreateDraft, ShortcutCreateTriggerKind};
use crate::gui::app::list_panel_search::{ListPanelSearchKind, list_panel_row_matches};
use crate::gui::theme::{self, GLYPH_PANEL_CONTENT_MAX_W_PX};

impl ArcadiaRoot {
    pub fn shortcuts_panel(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> AnyElement {
        if self.active_page_id.as_str() != "global.shortcuts" {
            return div().into_any_element();
        }

        let p = theme::theme_palette(cx, is_dark);
        let glyph_cfg = theme::glyph_snapshot(cx);
        let panel_radius = glyph_cfg.map(|g| g.border_radius).unwrap_or(p.radius_md);

        let overrides_map = ShortcutsConfig::load_or_create()
            .ok()
            .map(|c| c.overrides)
            .unwrap_or_default();

        let conflicts = shortcuts::chord_conflicts();
        let mut conflict_lines: Vec<String> = conflicts
            .iter()
            .map(|c| format!("{} → {}", c.fingerprint, c.shortcut_ids.join(", ")))
            .collect();
        conflict_lines.sort();

        let q = self.shortcuts_search_query.trim().to_ascii_lowercase();
        let list = global_shortcuts_for_settings();

        let filtered: Vec<_> = list
            .iter()
            .filter(|sc| {
                list_panel_row_matches(&q, &sc.label, &[sc.id.as_str(), sc.owner.as_str()])
            })
            .collect();

        let listening_id = self.shortcut_listening_id.clone();
        let listening_seq = self.shortcut_listening_sequence.clone();
        let listen_focus = self.shortcut_listen_focus.clone();

        let search_bar =
            self.list_panel_search_bar(window, cx, is_dark, ListPanelSearchKind::Shortcuts);

        let create_btn = div()
            .flex()
            .items_center()
            .justify_between()
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(p.ui_subtext)
                    .child("Custom shortcuts"),
            )
            .child(
                div()
                    .px_3()
                    .py_1()
                    .rounded(px(panel_radius.min(8.0)))
                    .bg(p.accent)
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(p.on_accent)
                    .cursor_pointer()
                    .child("+ Create")
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.shortcut_create_draft = Some(ShortcutCreateDraft {
                                label: String::new(),
                                trigger_kind: ShortcutCreateTriggerKind::Chord,
                                chord: None,
                                sequence: Vec::new(),
                                sequence_total: 2,
                                action_kind: ShortcutCreateActionKind::Navigate,
                                action_page_id: String::new(),
                                action_command_token: String::new(),
                                action_command_args: String::new(),
                                error: None,
                            });
                            cx.notify();
                        }),
                    ),
            );

        let rows: Vec<AnyElement> = filtered
            .into_iter()
            .map(|sc| {
                let o = overrides_map.get(&sc.id);
                let override_chord = o.and_then(|o| o.chord.clone());
                let override_sequence = o.and_then(|o| o.sequence.clone());
                let listening = listening_id.as_deref() == Some(sc.id.as_str());
                let seq_listening = listening_seq
                    .as_ref()
                    .filter(|(lid, ..)| lid == sc.id.as_str())
                    .map(|(_, captured, total)| (captured.len(), *total));
                Self::shortcut_row_item(
                    cx, sc, is_dark,
                    override_chord, override_sequence,
                    seq_listening, listening,
                    listen_focus.clone(),
                )
            })
            .collect();

        let empty_filtered = rows.is_empty()
            && !list.is_empty()
            && !self.shortcuts_search_query.trim().is_empty();

        let conflict_alert: Option<AnyElement> = if !conflict_lines.is_empty() {
            Some(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .p_3()
                    .rounded(px(panel_radius.min(12.0)))
                    .bg(p.badge_info_bg)
                    .border_1()
                    .border_color(p.border)
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(p.badge_info_fg)
                            .child("Chord conflicts"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(p.badge_info_fg)
                            .children(
                                conflict_lines
                                    .into_iter()
                                    .map(|line| div().child(line).into_any_element()),
                            ),
                    )
                    .into_any_element(),
            )
        } else {
            None
        };

        let list_body: AnyElement = if empty_filtered {
            div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .py_10()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(p.content_title)
                        .child("No matching shortcuts"),
                )
                .into_any_element()
        } else {
            div()
                .flex()
                .flex_col()
                .gap_3()
                .children(rows)
                .into_any_element()
        };

        if let Some(ref g) = glyph_cfg {
            let radius = g.border_radius.min(12.0);
            div()
                .w_full()
                .flex()
                .justify_center()
                .child(
                    div()
                        .w_full()
                        .max_w(px(GLYPH_PANEL_CONTENT_MAX_W_PX))
                        .p_4()
                        .rounded(px(radius))
                        .bg(g.surface)
                        .border_1()
                        .border_color(g.border)
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(search_bar)
                        .child(create_btn)
                        .when_some(conflict_alert, |d, alert| d.child(alert))
                        .child(list_body),
                )
                .into_any_element()
        } else {
            div()
                .w_full()
                .p_4()
                .rounded(px(8.))
                .bg(theme::module_panel_bg(is_dark))
                .border_1()
                .border_color(theme::module_panel_stroke(is_dark))
                .flex()
                .flex_col()
                .gap_3()
                .child(search_bar)
                .child(create_btn)
                .when_some(conflict_alert, |d, alert| d.child(alert))
                .child(list_body)
                .into_any_element()
        }
    }
}
