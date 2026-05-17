use openframe::{
    div, px, rgb, AnyElement, Context, FontWeight, InteractiveElement, IntoElement, MouseButton,
    ParentElement, StatefulInteractiveElement, Styled, Window,
};

use crate::gui::app::{ArcadiaRoot, VisualEditorTab};
use crate::gui::theme;
use arcadia_core::config::workspace::WorkspacesConfig;
use arcadia_core::config::ConfigFile;

impl ArcadiaRoot {
    pub(super) fn visual_editor_dashboard(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> AnyElement {
        let _ = window;
        let p = theme::theme_palette(cx, is_dark);
        let pal = theme::nav_accent_palette("sky", is_dark);
        let r = p.radius_md.min(12.0);

        let workspace_cards: Vec<AnyElement> = WorkspacesConfig::load_or_create()
            .map(|cfg| cfg.workspaces)
            .unwrap_or_default()
            .into_iter()
            .map(|ws| {
                let ws_path = ws.path.clone();
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
                            let id = this.visual_editor.next_id;
                            this.visual_editor.tabs.push(VisualEditorTab {
                                id,
                                title: format!("untitled-{id}"),
                                content: String::new(),
                                workspace_path: Some(ws_path.clone()),
                                file_path: None,
                                saved_content: String::new(),
                                block_positions: Vec::new(),
                            });
                            this.visual_editor.active_tab = this.visual_editor.tabs.len() - 1;
                            this.visual_editor.next_id += 1;
                            this.active_page_id = "editor.visual".to_string();
                            this.sync_settings_hub_expanded_from_active_page();
                            this.visual_editor.show_dashboard = false;
                            this.save_visual_editor_session();
                            this.visual_editor.focus.focus(window);
                            cx.notify();
                        }),
                    )
                    .into_any_element()
            })
            .collect();

        let tab_cards: Vec<AnyElement> = self
            .visual_editor.tabs
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
                    .gap_1()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, window, cx| {
                            this.visual_editor.active_tab = tab_idx;
                            this.visual_editor.show_dashboard = false;
                            this.active_page_id = "editor.visual".to_string();
                            this.sync_settings_hub_expanded_from_active_page();
                            this.visual_editor.focus.focus(window);
                            cx.notify();
                        }),
                    )
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
                    .child(div().text_xs().text_color(p.content_meta).child(subtitle))
                    .into_any_element()
            })
            .collect();

        let phantoms = || -> Vec<AnyElement> {
            (0..5)
                .map(|_| div().flex_1().min_w(px(220.)).into_any_element())
                .collect()
        };

        let tabs_body: AnyElement = if tab_cards.is_empty() {
            div()
                .text_sm()
                .text_color(p.content_meta)
                .child("No open documents.")
                .into_any_element()
        } else {
            div()
                .flex()
                .flex_row()
                .flex_wrap()
                .gap_4()
                .children(tab_cards)
                .children(phantoms())
                .into_any_element()
        };

        let workspaces_body: AnyElement = if workspace_cards.is_empty() {
            div()
                .text_sm()
                .text_color(p.content_meta)
                .child("No workspaces registered. Add one in Settings → Workspaces.")
                .into_any_element()
        } else {
            div()
                .flex()
                .flex_row()
                .flex_wrap()
                .gap_4()
                .children(workspace_cards)
                .children(phantoms())
                .into_any_element()
        };

        div()
            .id("visual-editor-dashboard")
            .w_full()
            .h_full()
            .overflow_y_scroll()
            .bg(if is_dark { rgb(0x1a1f29) } else { rgb(0xfafafa) })
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
                            .child("Open Documents"),
                    )
                    .child(tabs_body),
            )
            .into_any_element()
    }
}
