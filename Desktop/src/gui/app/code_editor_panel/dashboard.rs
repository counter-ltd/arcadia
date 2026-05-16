use openframe::{
    div, font, px, rgb, AnyElement, Context, FontWeight, InteractiveElement, IntoElement,
    MouseButton, ParentElement, StatefulInteractiveElement, Styled, Window,
};

use super::line_render::{code_line_content, LineStyle};
use super::text::{detect_language, expand_tabs};
use crate::gui::app::{ArcadiaRoot, CodeEditorTab};
use crate::gui::assets::MONO_FONT_FAMILY;
use crate::gui::theme;
use arcadia_core::config::workspace::WorkspacesConfig;
use arcadia_core::config::ConfigFile;
use arcadia_core::modules::python_registry;

impl ArcadiaRoot {
    pub(super) fn code_editor_dashboard(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> AnyElement {
        let p = theme::theme_palette(cx, is_dark);

        // Measured monospace cell width at text_xs — preview lines render on the
        // same char grid as the editor.
        let preview_cw = {
            let font_size = window.rem_size() * 0.75;
            let ts = window.text_system();
            let fid = ts.resolve_font(&font(MONO_FONT_FAMILY));
            ts.ch_advance(fid, font_size).map(f32::from).unwrap_or(7.0)
        };
        let preview_indent_guide = if is_dark {
            rgb(0x2d3748)
        } else {
            rgb(0xd1d5db)
        };
        let preview_style = LineStyle {
            char_width: preview_cw,
            line_fg: p.content_title,
            sel_bg: p.content_title,
            cursor_bg: p.content_title,
            cursor_fg: p.content_title,
            indent_guide_color: preview_indent_guide,
            show_indent_guides: true,
            small: true,
            cursor_style: arcadia_core::config::code_editor::CursorStyle::Block,
        };

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
                // Preview runs the real syntax + decoration providers so
                // user-created extensions render here exactly as in the editor.
                let prev_lang = tab.language.clone().or_else(|| detect_language(&tab.title));
                let prev_src: String =
                    tab.content.lines().take(9).collect::<Vec<_>>().join("\n");
                let prev_hl = prev_lang
                    .as_deref()
                    .map(|l| python_registry::call_highlight_provider(l, &prev_src))
                    .unwrap_or_default();
                let prev_lines: Vec<(usize, String)> = {
                    let mut v = Vec::new();
                    let mut pos = 0usize;
                    for ln in prev_src.split('\n') {
                        v.push((pos, ln.to_string()));
                        pos += ln.len() + 1;
                    }
                    v
                };
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
                            .children(prev_lines.into_iter().enumerate().map(
                                move |(li, (start, ln))| {
                                    let deco = python_registry::call_decoration_providers(
                                        &expand_tabs(&ln),
                                        li,
                                    );
                                    code_line_content(
                                        &ln,
                                        start,
                                        &prev_hl,
                                        &deco,
                                        None,
                                        None,
                                        preview_style,
                                    )
                                },
                            )),
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
