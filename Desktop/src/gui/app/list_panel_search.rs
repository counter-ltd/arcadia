//! Search field shared by Modules and Extensions list panels.

use openframe::{px, text_input, Context, IntoElement, Styled, Window};

use crate::gui::app::ArcadiaRoot;
use crate::gui::theme;

#[derive(Clone, Copy)]
pub(crate) enum ListPanelSearchKind {
    Modules,
    Extensions,
    Permissions,
    Shortcuts,
    Workspaces,
    Rules,
    Skills,
}

pub(crate) fn list_panel_row_matches(q_lower: &str, primary: &str, extras: &[&str]) -> bool {
    if q_lower.is_empty() {
        return true;
    }
    primary.to_ascii_lowercase().contains(q_lower)
        || extras
            .iter()
            .any(|s| s.to_ascii_lowercase().contains(q_lower))
}

impl ArcadiaRoot {
    pub(crate) fn list_panel_search_bar(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
        kind: ListPanelSearchKind,
    ) -> impl IntoElement {
        let (text, focus) = match kind {
            ListPanelSearchKind::Modules => {
                (&self.modules_search_query, &self.modules_search_focus)
            }
            ListPanelSearchKind::Extensions => {
                (&self.extensions_search_query, &self.extensions_search_focus)
            }
            ListPanelSearchKind::Permissions => (
                &self.permissions_search_query,
                &self.permissions_search_focus,
            ),
            ListPanelSearchKind::Shortcuts => {
                (&self.shortcuts_search_query, &self.shortcuts_search_focus)
            }
            ListPanelSearchKind::Workspaces => {
                (&self.workspace_search_query, &self.workspace_search_focus)
            }
            ListPanelSearchKind::Rules => (&self.rules_search_query, &self.rules_search_focus),
            ListPanelSearchKind::Skills => (&self.skills_search_query, &self.skills_search_focus),
        };
        let placeholder = match kind {
            ListPanelSearchKind::Modules => "Search modules…",
            ListPanelSearchKind::Extensions => "Search extensions…",
            ListPanelSearchKind::Permissions => "Search permissions…",
            ListPanelSearchKind::Shortcuts => "Search shortcuts…",
            ListPanelSearchKind::Workspaces => "Search workspaces…",
            ListPanelSearchKind::Rules => "Search rules…",
            ListPanelSearchKind::Skills => "Search skills…",
        };

        let input_bg = theme::glyph_snapshot(cx)
            .map(|g| g.surface)
            .unwrap_or_else(|| theme::ui_surface(cx, is_dark));
        let input_border = theme::glyph_snapshot(cx)
            .map(|g| g.border)
            .unwrap_or_else(|| theme::ui_border(cx, is_dark));
        let radius = theme::ui_radius(cx);
        let title_c = theme::module_title_text(is_dark);
        let meta_c = theme::module_meta_text(is_dark);

        let weak = cx.weak_entity();

        text_input(
            match kind {
                ListPanelSearchKind::Modules => "list-search-modules",
                ListPanelSearchKind::Extensions => "list-search-extensions",
                ListPanelSearchKind::Permissions => "list-search-permissions",
                ListPanelSearchKind::Shortcuts => "list-search-shortcuts",
                ListPanelSearchKind::Workspaces => "list-search-workspaces",
                ListPanelSearchKind::Rules => "list-search-rules",
                ListPanelSearchKind::Skills => "list-search-skills",
            },
            window,
            weak,
            text,
            placeholder,
            focus,
            title_c,
            meta_c,
            move |this, new_text, cx| {
                let buf = match kind {
                    ListPanelSearchKind::Modules => &mut this.modules_search_query,
                    ListPanelSearchKind::Extensions => &mut this.extensions_search_query,
                    ListPanelSearchKind::Permissions => &mut this.permissions_search_query,
                    ListPanelSearchKind::Shortcuts => &mut this.shortcuts_search_query,
                    ListPanelSearchKind::Workspaces => &mut this.workspace_search_query,
                    ListPanelSearchKind::Rules => &mut this.rules_search_query,
                    ListPanelSearchKind::Skills => &mut this.skills_search_query,
                };
                *buf = new_text;
                cx.notify();
            },
        )
        .w_full()
        .px_3()
        .py_2()
        .rounded(px(radius))
        .bg(input_bg)
        .border_1()
        .border_color(input_border)
        .text_sm()
    }
}
