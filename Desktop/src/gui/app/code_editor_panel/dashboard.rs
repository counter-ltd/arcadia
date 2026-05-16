use openframe::{
    div, px, rgb, AnyElement, Context, FontWeight, InteractiveElement, IntoElement, MouseButton,
    ParentElement, StatefulInteractiveElement, Styled, Window,
};

use crate::gui::app::{ArcadiaRoot, CodeEditorTab};
use crate::gui::theme;
use arcadia_core::config::workspace::WorkspacesConfig;
use arcadia_core::config::ConfigFile;

impl ArcadiaRoot {
    pub(super) fn code_editor_dashboard(
        &mut self,
        _window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> AnyElement {
        let p = theme::theme_palette(cx, is_dark);

        let pal = theme::nav_accent_palette("emerald", is_dark);
        let r = p.radius_md.min(12.0);

        let workspace_cards: Vec<AnyElement> = WorkspacesConfig::load_or_create()
            .map(|cfg| cfg.workspaces)
            .unwrap_or_default()
            .into_iter()
            .map(|ws| {
                let ws_path_clone = ws.path.clone();
                div()
                    .flex_1()
                    .min_w(px(220.))
                    .cursor_pointer()
                    .p_4()
                    .rounded(px(r))
                    .bg(p.panel_bg)
                    .border_1()
                    .border_color(p.panel_border)
                    .hover(move |s| s.bg(p.row_bg).border_color(pal.row_hover))
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .gap_3()
                            .items_center()
                            .child(
                                theme::render_icon("folder-open")
                                    .size_6()
                                    .text_color(pal.icon_active),
                            )
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(p.content_title)
                                    .flex_1()
                                    .child(ws.label.clone()),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(p.content_meta)
                            .max_w(px(248.))
                            .child(ws.path.clone()),
                    )
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, window, cx| {
                            let id = this.code_editor_next_id;
                            this.code_editor_tabs.push(CodeEditorTab {
                                id,
                                title: format!("untitled-{id}"),
                                content: String::new(),
                                cursor: 0,
                                selection_anchor: None,
                                language: None,
                                hl_spans: vec![],
                                decorations: vec![],
                                highlight_dirty: true,
                                workspace_path: Some(ws_path_clone.clone()),
                                file_path: None,
                                saved_content: String::new(),
                                cached_lines: vec![],
                                cached_line_byte_starts: vec![],
                            });
                            this.active_code_editor_tab = this.code_editor_tabs.len() - 1;
                            this.code_editor_next_id += 1;
                            this.active_page_id = "editor.main".to_string();
                            this.sync_settings_hub_expanded_from_active_page();
                            this.code_editor_show_dashboard = false;
                            this.save_editor_session();
                            this.code_editor_focus.focus(window);
                            cx.notify();
                        }),
                    )
                    .into_any_element()
            })
            .collect();

        let editor_bg_preview = if is_dark {
            rgb(0x141820)
        } else {
            rgb(0xf0f2f5)
        };
        let editor_cards: Vec<AnyElement> = self
            .code_editor_tabs
            .iter()
            .enumerate()
            .map(|(tab_idx, tab)| {
                let title = tab.title.clone();
                let subtitle = tab
                    .file_path
                    .as_deref()
                    .or(tab.workspace_path.as_deref())
                    .unwrap_or("unsaved")
                    .to_string();
                let is_dirty = tab.content != tab.saved_content;
                let preview_lines: Vec<String> = tab
                    .content
                    .lines()
                    .take(9)
                    .map(|l| {
                        let s: String = l.chars().take(48).collect();
                        s
                    })
                    .collect();
                let fh_card = self.code_editor_focus.clone();
                div()
                    .flex_1()
                    .min_w(px(220.))
                    .cursor_pointer()
                    .rounded(px(r))
                    .bg(p.panel_bg)
                    .border_1()
                    .border_color(p.panel_border)
                    .hover(move |s| s.bg(p.row_bg).border_color(pal.row_hover))
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, window, cx| {
                            this.active_code_editor_tab = tab_idx;
                            this.code_editor_show_dashboard = false;
                            this.active_page_id = "editor.main".to_string();
                            this.sync_settings_hub_expanded_from_active_page();
                            fh_card.focus(window);
                            cx.notify();
                        }),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_h(px(100.))
                            .bg(editor_bg_preview)
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
                                    .child(if line.is_empty() {
                                        " ".to_string()
                                    } else {
                                        line
                                    })
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
                                    .flex()
                                    .flex_row()
                                    .gap_1()
                                    .items_center()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(p.content_title)
                                            .flex_1()
                                            .child(title),
                                    )
                                    .child(if is_dirty {
                                        div()
                                            .w(px(6.))
                                            .h(px(6.))
                                            .rounded_full()
                                            .bg(pal.icon_active)
                                            .into_any_element()
                                    } else {
                                        div().into_any_element()
                                    }),
                            )
                            .child(div().text_xs().text_color(p.content_meta).child(subtitle)),
                    )
                    .into_any_element()
            })
            .collect();

        let editors_body: AnyElement = if editor_cards.is_empty() {
            div()
                .text_sm()
                .text_color(p.content_meta)
                .child("No open editors.")
                .into_any_element()
        } else {
            let phantoms: Vec<_> = (0..5)
                .map(|_| div().flex_1().min_w(px(220.)).into_any_element())
                .collect();
            div()
                .flex()
                .flex_row()
                .flex_wrap()
                .gap_4()
                .children(editor_cards)
                .children(phantoms)
                .into_any_element()
        };

        let workspaces_body: AnyElement = if workspace_cards.is_empty() {
            div()
                .text_sm()
                .text_color(p.content_meta)
                .child("No workspaces registered. Add one in Settings → Workspaces.")
                .into_any_element()
        } else {
            let phantoms: Vec<_> = (0..5)
                .map(|_| div().flex_1().min_w(px(220.)).into_any_element())
                .collect();
            div()
                .flex()
                .flex_row()
                .flex_wrap()
                .gap_4()
                .children(workspace_cards)
                .children(phantoms)
                .into_any_element()
        };

        return div()
            .id("editor-empty-state")
            .w_full()
            .h_full()
            .overflow_y_scroll()
            .bg(if is_dark {
                rgb(0x1a1f29)
            } else {
                rgb(0xfafafa)
            })
            .p_8()
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
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(p.content_title)
                            .child("Workspaces"),
                    )
                    .child(workspaces_body),
            )
            .child(div().w_full().h(px(1.)).bg(p.panel_border))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .text_xl()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(p.content_title)
                            .child("Open Editors"),
                    )
                    .child(editors_body),
            )
            .into_any_element();
    }
}
