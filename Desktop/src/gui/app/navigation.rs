use arcadia_core::config::modules::{ModulesConfig, AI_MODULE_NAME};
use arcadia_core::config::permissions::{PermissionSubject, PermissionsConfig};
use arcadia_core::config::ConfigFile as _;
use arcadia_core::modules::ai::any_ai_provider_enabled;
use arcadia_core::modules::python_registry;
use arcadia_core::navigation::{self, NavigationGroupOwned, NavigationPageOwned};
use openframe::prelude::FluentBuilder as _;
use openframe::{
    div, px, Context, Div, FontWeight, InteractiveElement, IntoElement, MouseButton, ParentElement,
    SharedString, StatefulInteractiveElement, Styled, Tooltip, Window,
};

use super::ArcadiaRoot;
use crate::gui::theme::{self, render_icon};

#[derive(Clone, Copy)]
pub(crate) enum NavPageRef<'a> {
    Static(&'static navigation::NavigationPageDefinition),
    Remote(&'a NavigationPageOwned),
}

#[derive(Clone, Copy)]
pub(crate) enum NavGroupRef<'a> {
    Static(&'static navigation::NavigationGroupDefinition),
    Remote(&'a NavigationGroupOwned),
}

impl NavPageRef<'_> {
    pub fn id(&self) -> &str {
        match self {
            NavPageRef::Static(p) => p.id,
            NavPageRef::Remote(p) => p.id.as_str(),
        }
    }

    pub fn title(&self) -> &str {
        match self {
            NavPageRef::Static(p) => p.title,
            NavPageRef::Remote(p) => p.title.as_str(),
        }
    }

    pub fn description(&self) -> &str {
        match self {
            NavPageRef::Static(p) => p.description,
            NavPageRef::Remote(p) => p.description.as_str(),
        }
    }

    pub fn glyph(&self) -> &str {
        match self {
            NavPageRef::Static(p) => p.glyph,
            NavPageRef::Remote(p) => p.glyph.as_str(),
        }
    }

    pub fn accent(&self) -> &str {
        match self {
            NavPageRef::Static(p) => p.accent,
            NavPageRef::Remote(p) => p.accent.as_str(),
        }
    }

    #[allow(dead_code)]
    pub fn required_module(&self) -> Option<&str> {
        match self {
            NavPageRef::Static(p) => p.required_module,
            NavPageRef::Remote(p) => p.required_module.as_deref(),
        }
    }
}

impl NavGroupRef<'_> {
    pub fn id(&self) -> &str {
        match self {
            NavGroupRef::Static(g) => g.id,
            NavGroupRef::Remote(g) => g.id.as_str(),
        }
    }

    pub fn label(&self) -> &str {
        match self {
            NavGroupRef::Static(g) => g.label,
            NavGroupRef::Remote(g) => g.label.as_str(),
        }
    }

    pub fn glyph(&self) -> &str {
        match self {
            NavGroupRef::Static(g) => g.glyph,
            NavGroupRef::Remote(g) => g.glyph.as_str(),
        }
    }

    #[allow(dead_code)]
    pub fn system_image(&self) -> &str {
        match self {
            NavGroupRef::Static(g) => g.system_image,
            NavGroupRef::Remote(g) => g.system_image.as_str(),
        }
    }

    pub fn accent(&self) -> &str {
        match self {
            NavGroupRef::Static(g) => g.accent,
            NavGroupRef::Remote(g) => g.accent.as_str(),
        }
    }

    pub fn page_ids(&self) -> Vec<&str> {
        match self {
            NavGroupRef::Static(g) => g.pages.iter().copied().collect(),
            NavGroupRef::Remote(g) => g.pages.iter().map(|s| s.as_str()).collect(),
        }
    }
}

impl ArcadiaRoot {
    pub(crate) fn page_ref(&self, page_id: &str) -> Option<NavPageRef<'_>> {
        if let Some(nav) = self.navigation_registry() {
            if let Some(p) = nav.pages.iter().find(|p| p.id == page_id) {
                return Some(NavPageRef::Remote(p));
            }
        }
        navigation::page_by_id(page_id).map(NavPageRef::Static)
    }

    pub(crate) fn effective_group(&self, group_id: &str) -> Option<NavGroupRef<'_>> {
        self.visible_groups_effective()
            .into_iter()
            .find(|g| g.id() == group_id)
    }

    pub(crate) fn global_page_ids_effective(&self) -> Vec<&str> {
        self.navigation_registry()
            .map(|nav| nav.global_pages.iter().map(|s| s.as_str()).collect())
            .unwrap_or_else(|| navigation::GLOBAL_PAGE_IDS.iter().copied().collect())
    }

    pub(crate) fn top_bar_page_ids_effective(&self) -> Vec<&str> {
        let mut v: Vec<&str> = self
            .navigation_registry()
            .map(|nav| nav.top_bar_pages.iter().map(|s| s.as_str()).collect())
            .unwrap_or_else(|| navigation::TOP_BAR_PAGE_IDS.iter().copied().collect());
        v.retain(|&id| id != navigation::LOGS_PAGE_ID);
        v
    }

    pub(crate) fn settings_hub_page_ids_effective(&self) -> Vec<&str> {
        let mut ids: Vec<&str> = self
            .navigation_registry()
            .map(|nav| {
                if nav.settings_hub_pages.is_empty() {
                    navigation::SETTINGS_HUB_PAGE_IDS.iter().copied().collect()
                } else {
                    nav.settings_hub_pages.iter().map(|s| s.as_str()).collect()
                }
            })
            .unwrap_or_else(|| navigation::SETTINGS_HUB_PAGE_IDS.iter().copied().collect());
        ids.sort_by_key(|id| navigation::page_by_id(id).map(|p| p.title).unwrap_or(""));
        ids
    }

    /// Expand or collapse the Settings hub based on whether the active page is a settings page.
    pub(crate) fn sync_settings_hub_expanded_from_active_page(&mut self) {
        let active = self.active_page_id.as_str();
        let in_settings = active == navigation::SETTINGS_HUB_ROOT_PAGE_ID
            || self
                .settings_hub_page_ids_effective()
                .iter()
                .any(|p| *p == active);
        if in_settings != self.settings_hub_expanded {
            self.settings_hub_expanded = in_settings;
            self.start_settings_expand_anim(in_settings);
        }
        self.sync_pill_expand_from_active_page();
    }

    pub(crate) fn sync_pill_expand_from_active_page(&mut self) {
        let active = self.active_page_id.clone();
        let on_settings = active.as_str() == navigation::SETTINGS_HUB_ROOT_PAGE_ID
            || self
                .settings_hub_page_ids_effective()
                .iter()
                .any(|p| *p == active.as_str());
        for &pill_id in &["notification.main", "extensions.settings", "global.modules"] {
            let should_expand = on_settings || active.as_str() == pill_id;
            let current = self.pill_expanded.get(pill_id).copied().unwrap_or(false);
            if should_expand != current {
                self.pill_expanded.insert(pill_id.to_string(), should_expand);
                self.start_pill_expand_anim(pill_id, should_expand);
            }
        }
    }

    pub(crate) fn visible_groups_effective(&self) -> Vec<NavGroupRef<'_>> {
        let all: Vec<NavGroupRef<'_>> = if let Some(nav) = self.navigation_registry() {
            nav.groups.iter().map(NavGroupRef::Remote).collect()
        } else {
            Vec::new()
        };
        let mut visible: Vec<NavGroupRef<'_>> = all
            .into_iter()
            .filter(|g| g.page_ids().iter().any(|pid| self.is_page_visible(pid)))
            .collect();
        visible.sort_by_key(|g| g.label().to_string());
        visible
    }

    pub(crate) fn effective_default_page(&self) -> &str {
        self.navigation_registry()
            .map(|n| n.default_page.as_str())
            .unwrap_or_else(|| {
                if self.remote_navigation_required() {
                    "__thin.nav_waiting__"
                } else {
                    navigation::DEFAULT_PAGE_ID
                }
            })
    }

    pub(crate) fn render_thin_client_waiting_panel(
        &mut self,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> Div {
        let p = theme::theme_palette(cx, is_dark);
        div()
            .w_full()
            .flex_1()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_4()
            .p_8()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(p.content_title)
                    .child("Waiting for host navigation"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(p.content_body)
                    .child(
                        "navigation_from_host_only is set in thin-client.toml but the host snapshot has no navigation_registry yet.",
                    ),
            )
            .child(
                div()
                    .px_4()
                    .py_2()
                    .rounded(px(8.))
                    .cursor_pointer()
                    .bg(p.accent)
                    .text_color(p.on_accent)
                    .child("Reload from host")
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.reload_modules();
                            this.ensure_valid_navigation_selection();
                            cx.notify();
                        }),
                    ),
            )
    }

    pub(crate) fn render_active_content(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> Div {
        if self.thin_client_nav_waiting_host() {
            return self.render_thin_client_waiting_panel(cx, is_dark);
        }
        #[cfg(feature = "gui")]
        if self.active_page_id.as_str() == "utility.shell" {
            return div()
                .flex_1()
                .h_full()
                .min_h_0()
                .child(self.shell_panel(window, cx));
        }
        #[cfg(all(feature = "ios-gui", not(feature = "gui")))]
        if self.active_page_id.as_str() == "utility.shell" {
            return div()
                .flex_1()
                .h_full()
                .min_h_0()
                .child(self.ios_execute_shell_panel(window, cx));
        }
        // Full-height / custom-layout pages — handled before the standard padded dispatcher.
        if self.active_page_id.as_str() == "late.now_playing" {
            return self.render_late_now_playing(window, cx, is_dark);
        }
        if self.active_page_id.as_str() == "ai.chat" {
            let diff_open = self.ai_diff_panel_open && !self.ai_pending_edits.is_empty();
            let chat = self.ai_chat_panel(window, cx, is_dark);
            let mut row = div()
                .flex_1()
                .h_full()
                .min_h_0()
                .flex()
                .flex_row()
                .child(chat);
            if diff_open {
                let diff = self.ai_diff_panel(cx, is_dark);
                row = row.child(diff);
            }
            return row;
        }

        // Dynamic extension-token settings pages (page IDs aren't in PAGE_DEFINITIONS).
        let active = self.active_page_id.clone();
        if let Some(mid) = navigation::parse_extension_token_settings_page_id(&active) {
            return div()
                .w_full()
                .p_6()
                .child(self.extension_token_settings_panel_for_module(window, cx, is_dark, mid));
        }
        // Dynamic extension nav pages declared via arcadia.register_nav_page().
        if let Some(ext_id) = navigation::parse_extension_nav_page_id(&active) {
            return div()
                .w_full()
                .p_6()
                .child(self.extension_nav_panel(window, cx, is_dark, ext_id));
        }

        // Standard padded panels — add new pages here as new match arms.
        // All arms share the same div().w_full().p_6() wrapper.
        {
            let panel: Option<openframe::AnyElement> = match self.active_page_id.as_str() {
                "global.modules" => {
                    Some(self.modules_panel(window, cx, is_dark).into_any_element())
                }
                "network.nodes" => Some(self.lan_nodes_panel(cx, is_dark).into_any_element()),
                "utility.services" => Some(self.services_panel(cx, is_dark).into_any_element()),
                "late.settings" => Some(
                    self.late_settings_panel(window, cx, is_dark)
                        .into_any_element(),
                ),
                "extensions.settings" => Some(
                    self.python_settings_panel(window, cx, is_dark)
                        .into_any_element(),
                ),
                "global.appearance" => Some(
                    self.appearance_panel(window, cx, is_dark)
                        .into_any_element(),
                ),
                "global.permissions" => Some(
                    self.permissions_panel(window, cx, is_dark)
                        .into_any_element(),
                ),
                "global.shortcuts" => {
                    Some(self.shortcuts_panel(window, cx, is_dark).into_any_element())
                }
                "editor.settings" => Some(
                    self.code_editor_settings_panel(window, cx, is_dark)
                        .into_any_element(),
                ),
                "global.workspaces" => Some(self.workspace_panel(window, cx, is_dark)),
                "ai.settings" => Some(
                    self.ai_settings_panel(window, cx, is_dark)
                        .into_any_element(),
                ),
                "ai.models" => Some(self.ai_models_panel(window, cx, is_dark).into_any_element()),
                "ai.rules" => Some(self.ai_rules_panel(window, cx, is_dark).into_any_element()),
                "ai.skills" => Some(self.ai_skills_panel(window, cx, is_dark).into_any_element()),
                "notification.main" => Some(
                    self.notification_panel(window, cx, is_dark)
                        .into_any_element(),
                ),
                "notification.settings" => Some(
                    self.notification_settings_panel(window, cx, is_dark)
                        .into_any_element(),
                ),
                navigation::SETTINGS_HUB_ROOT_PAGE_ID => Some(
                    self.settings_hub_landing_panel(cx, is_dark)
                        .into_any_element(),
                ),
                _ => None,
            };
            if let Some(content) = panel {
                if self.active_page_id.as_str() == navigation::SETTINGS_HUB_ROOT_PAGE_ID {
                    let hub_bg = theme::theme_palette(cx, is_dark).canvas;
                    return div()
                        .w_full()
                        .h_full()
                        .bg(hub_bg)
                        .p_8()
                        .child(content);
                }
                return div().w_full().p_6().child(content);
            }
        }
        if self.active_page_id.as_str() == "editor.main" {
            let active_idx = self
                .active_code_editor_tab
                .min(self.code_editor_tabs.len().saturating_sub(1));
            let ws_path = self
                .code_editor_tabs
                .get(active_idx)
                .and_then(|t| t.workspace_path.clone())
                .unwrap_or_default();
            let show_explorer = self.code_editor_explorer_open && !ws_path.is_empty();
            let sidebar_bg = theme::explorer_sidebar_bg(is_dark);
            let border_color = theme::explorer_border(is_dark);
            let text_color = theme::explorer_text(is_dark);
            let dim_color = theme::explorer_dim(is_dark);
            let hover_bg = theme::explorer_hover_bg(is_dark);
            let mut row = div().w_full().h_full().flex().flex_row();
            if show_explorer {
                // Collect flat entry list with depth via recursive walk
                let mut flat: Vec<(String, String, bool, usize)> = Vec::new(); // (full_path, name, is_dir, depth)
                collect_explorer_entries(
                    &ws_path,
                    0,
                    &self.code_editor_explorer_expanded,
                    false,
                    &mut flat,
                );

                let mut sidebar = div()
                    .w(px(220.))
                    .h_full()
                    .flex_shrink_0()
                    .bg(sidebar_bg)
                    .border_r_1()
                    .border_color(border_color)
                    .flex()
                    .flex_col()
                    .overflow_hidden();

                // Header
                sidebar = sidebar.child(
                    div()
                        .px_3()
                        .py_2()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(text_color)
                        .border_b_1()
                        .border_color(border_color)
                        .child(ws_path.rsplit('/').next().unwrap_or(&ws_path).to_string()),
                );

                let active_file_path = self
                    .code_editor_tabs
                    .get(active_idx)
                    .and_then(|t| t.file_path.clone());
                let active_row_bg = theme::explorer_active_row_bg(is_dark);
                let active_row_text = theme::explorer_active_row_text(is_dark);

                if flat.is_empty() {
                    sidebar = sidebar.child(
                        div()
                            .px_3()
                            .py_2()
                            .text_xs()
                            .text_color(dim_color)
                            .child("No files found."),
                    );
                } else {
                    for (full_path, name, is_dir, depth) in flat {
                        let full_path2 = full_path.clone();
                        let is_expanded = self.code_editor_explorer_expanded.contains(&full_path);
                        let is_active_file =
                            !is_dir && active_file_path.as_deref() == Some(full_path.as_str());
                        let folder_icon = if is_expanded { "folder-open" } else { "folder" };
                        let file_icon = file_icon_for(&name);
                        let indent_px = px(8. + depth as f32 * 16.);
                        let row_text = if is_active_file {
                            active_row_text
                        } else {
                            text_color
                        };

                        let entry_row = div()
                            .pl(indent_px)
                            .pr_2()
                            .py(px(3.))
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_1p5()
                            .text_xs()
                            .text_color(row_text)
                            .cursor_pointer()
                            .when(is_active_file, |s| s.bg(active_row_bg))
                            .hover(move |s| s.bg(hover_bg));

                        let entry_row = if is_dir {
                            let chevron = if is_expanded {
                                "chevron-down"
                            } else {
                                "chevron-right"
                            };
                            entry_row
                                .child(theme::render_icon(chevron).size_3().text_color(dim_color))
                                .child(
                                    theme::render_icon(folder_icon)
                                        .size_3()
                                        .text_color(text_color),
                                )
                                .child(name)
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |this, _, _, cx| {
                                        if this.code_editor_explorer_expanded.contains(&full_path2)
                                        {
                                            this.code_editor_explorer_expanded.remove(&full_path2);
                                        } else {
                                            this.code_editor_explorer_expanded
                                                .insert(full_path2.clone());
                                        }
                                        cx.notify();
                                    }),
                                )
                        } else {
                            entry_row
                                .child(div().size_3()) // spacer aligns with dir chevron
                                .child(theme::render_icon(&file_icon).size_3().text_color(row_text))
                                .child(name)
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |this, _, _, cx| {
                                        // Switch to existing tab if already open.
                                        if let Some(existing) =
                                            this.code_editor_tabs.iter().position(|t| {
                                                t.file_path.as_deref() == Some(full_path2.as_str())
                                            })
                                        {
                                            this.active_code_editor_tab = existing;
                                            this.code_editor_show_dashboard = false;
                                            cx.notify();
                                            return;
                                        }
                                        if let Ok(text) = std::fs::read_to_string(&full_path2) {
                                            let lang =
                                                crate::gui::app::code_editor_panel::detect_language(
                                                    &full_path2,
                                                );
                                            let title = std::path::Path::new(&full_path2)
                                                .file_name()
                                                .map(|n| n.to_string_lossy().to_string())
                                                .unwrap_or_else(|| full_path2.clone());
                                            let idx = this
                                                .active_code_editor_tab
                                                .min(this.code_editor_tabs.len().saturating_sub(1));
                                            if let Some(tab) = this.code_editor_tabs.get_mut(idx) {
                                                tab.saved_content = text.clone();
                                                tab.content = text;
                                                tab.file_path = Some(full_path2.clone());
                                                tab.title = title;
                                                tab.language = lang;
                                                tab.cursor = 0;
                                                tab.selection_anchor = None;
                                                tab.highlight_dirty = true;
                                            }
                                            this.save_editor_session();
                                            cx.notify();
                                        }
                                    }),
                                )
                        };
                        sidebar = sidebar.child(entry_row);
                    }
                }
                row = row.child(sidebar);
            }
            return row.child(
                div()
                    .flex_1()
                    .h_full()
                    .overflow_hidden()
                    .child(self.code_editor_panel(window, cx, is_dark)),
            );
        }
        if self.active_page_id.as_str() == "editor.visual" {
            let active_idx = self
                .active_visual_editor_tab
                .min(self.visual_editor_tabs.len().saturating_sub(1));
            let ws_path = self
                .visual_editor_tabs
                .get(active_idx)
                .and_then(|t| t.workspace_path.clone())
                .unwrap_or_default();
            let show_explorer = self.visual_editor_explorer_open && !ws_path.is_empty();
            let sidebar_bg = theme::explorer_sidebar_bg(is_dark);
            let border_color = theme::explorer_border(is_dark);
            let text_color = theme::explorer_text(is_dark);
            let dim_color = theme::explorer_dim(is_dark);
            let hover_bg = theme::explorer_hover_bg(is_dark);
            let mut row = div().w_full().h_full().flex().flex_row();
            if show_explorer {
                let mut flat: Vec<(String, String, bool, usize)> = Vec::new();
                collect_explorer_entries(
                    &ws_path,
                    0,
                    &self.visual_editor_explorer_expanded,
                    true,
                    &mut flat,
                );

                let mut sidebar = div()
                    .w(px(220.))
                    .h_full()
                    .flex_shrink_0()
                    .bg(sidebar_bg)
                    .border_r_1()
                    .border_color(border_color)
                    .flex()
                    .flex_col()
                    .overflow_hidden();

                sidebar = sidebar.child(
                    div()
                        .px_3()
                        .py_2()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(text_color)
                        .border_b_1()
                        .border_color(border_color)
                        .child(ws_path.rsplit('/').next().unwrap_or(&ws_path).to_string()),
                );

                let active_file_path = self
                    .visual_editor_tabs
                    .get(active_idx)
                    .and_then(|t| t.file_path.clone());
                let active_row_bg = theme::explorer_active_row_bg(is_dark);
                let active_row_text = theme::explorer_active_row_text(is_dark);

                if flat.is_empty() {
                    sidebar = sidebar.child(
                        div()
                            .px_3()
                            .py_2()
                            .text_xs()
                            .text_color(dim_color)
                            .child("No Python files found."),
                    );
                } else {
                    for (full_path, name, is_dir, depth) in flat {
                        let full_path2 = full_path.clone();
                        let is_expanded =
                            self.visual_editor_explorer_expanded.contains(&full_path);
                        let is_active_file =
                            !is_dir && active_file_path.as_deref() == Some(full_path.as_str());
                        let folder_icon = if is_expanded { "folder-open" } else { "folder" };
                        let file_icon = file_icon_for(&name);
                        let indent_px = px(8. + depth as f32 * 16.);
                        let row_text = if is_active_file {
                            active_row_text
                        } else {
                            text_color
                        };

                        let entry_row = div()
                            .pl(indent_px)
                            .pr_2()
                            .py(px(3.))
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_1p5()
                            .text_xs()
                            .text_color(row_text)
                            .cursor_pointer()
                            .when(is_active_file, |s| s.bg(active_row_bg))
                            .hover(move |s| s.bg(hover_bg));

                        let entry_row = if is_dir {
                            let chevron = if is_expanded {
                                "chevron-down"
                            } else {
                                "chevron-right"
                            };
                            entry_row
                                .child(theme::render_icon(chevron).size_3().text_color(dim_color))
                                .child(
                                    theme::render_icon(folder_icon)
                                        .size_3()
                                        .text_color(text_color),
                                )
                                .child(name)
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |this, _, _, cx| {
                                        if this
                                            .visual_editor_explorer_expanded
                                            .contains(&full_path2)
                                        {
                                            this.visual_editor_explorer_expanded
                                                .remove(&full_path2);
                                        } else {
                                            this.visual_editor_explorer_expanded
                                                .insert(full_path2.clone());
                                        }
                                        cx.notify();
                                    }),
                                )
                        } else {
                            entry_row
                                .child(div().size_3())
                                .child(theme::render_icon(&file_icon).size_3().text_color(row_text))
                                .child(name)
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |this, _, _, cx| {
                                        if let Some(existing) =
                                            this.visual_editor_tabs.iter().position(|t| {
                                                t.file_path.as_deref() == Some(full_path2.as_str())
                                            })
                                        {
                                            this.active_visual_editor_tab = existing;
                                            this.visual_editor_show_dashboard = false;
                                            cx.notify();
                                            return;
                                        }
                                        if let Ok(text) = std::fs::read_to_string(&full_path2) {
                                            let title = std::path::Path::new(&full_path2)
                                                .file_name()
                                                .map(|n| n.to_string_lossy().to_string())
                                                .unwrap_or_else(|| full_path2.clone());
                                            let idx = this
                                                .active_visual_editor_tab
                                                .min(this.visual_editor_tabs.len().saturating_sub(1));
                                            if let Some(tab) = this.visual_editor_tabs.get_mut(idx) {
                                                tab.saved_content = text.clone();
                                                tab.content = text;
                                                tab.file_path = Some(full_path2.clone());
                                                tab.title = title;
                                            }
                                            this.save_visual_editor_session();
                                            cx.notify();
                                        }
                                    }),
                                )
                        };
                        sidebar = sidebar.child(entry_row);
                    }
                }
                row = row.child(sidebar);
            }
            let canvas = div()
                .flex_1()
                .h_full()
                .overflow_hidden()
                .child(self.visual_editor_panel(window, cx, is_dark));
            let mut row = row.child(canvas);
            if self.visual_editor_palette_open {
                let palette = self.visual_editor_palette_panel(cx, is_dark);
                row = row.child(palette);
            }
            return row;
        }
        {
            let active_page = self
                .active_page_if_visible()
                .or_else(|| self.page_ref(self.effective_default_page()));
            div()
                .w_full()
                .p_6()
                .flex()
                .flex_col()
                .items_center()
                .gap_3()
                .py_16()
                .child(
                    div()
                        .text_3xl()
                        .font_weight(FontWeight::BOLD)
                        .child(self.title.clone()),
                )
                .child(
                    div()
                        .text_2xl()
                        .text_color(theme::ui_text(cx, is_dark))
                        .child(
                            active_page.map_or_else(
                                || "Page".to_string(),
                                |page| page.title().to_string(),
                            ),
                        ),
                )
                .child(
                    div()
                        .text_base()
                        .text_color(theme::ui_subtext(cx, is_dark))
                        .child(active_page.map_or_else(
                            || "Page definition not found.".to_string(),
                            |page| page.description().to_string(),
                        )),
                )
        }
    }

    /// Hub root (`global.settings`): grid of all nested settings pages from the registry.
    fn settings_hub_landing_panel(&mut self, cx: &mut Context<Self>, is_dark: bool) -> Div {
        let p = theme::theme_palette(cx, is_dark);
        let hub = self.page_ref(navigation::SETTINGS_HUB_ROOT_PAGE_ID);
        let perm_cfg = PermissionsConfig::load_or_create().ok();

        let mut hub_page_ids = self.settings_hub_page_ids_effective();
        hub_page_ids.sort_by_key(|id| {
            self.page_ref(id)
                .map(|p| p.title().to_string())
                .unwrap_or_default()
        });

        let tiles: Vec<_> = hub_page_ids
            .into_iter()
            .filter_map(|page_id| {
                if !self.is_page_visible(page_id) {
                    return None;
                }
                let page = self.page_ref(page_id)?;
                let page_id_owned = page_id.to_string();
                let page_id_pin = page_id_owned.clone();
                let pal = theme::nav_accent_palette(page.accent(), is_dark);
                let r = p.radius_md.min(12.0);
                let is_pinned = self.pinned_settings_pages.iter().any(|p| p == page_id);
                let pin_glyph = if is_pinned { "pin-fill" } else { "pin" };
                let pin_color = if is_pinned {
                    pal.icon_active
                } else {
                    p.content_meta
                };
                // Collect (perm_id, granted) pairs for this page's module.
                // Extension token pages declare their own permissions separately from
                // the python-host module that gates their visibility.
                let perm_rows: Vec<(String, bool)> =
                    if let Some(ext_id) = navigation::parse_extension_token_settings_page_id(page_id) {
                        let declared = python_registry::extension_declared_permissions(ext_id);
                        let subj = PermissionSubject::python(ext_id);
                        declared
                            .into_iter()
                            .map(|pid| {
                                let granted = perm_cfg
                                    .as_ref()
                                    .map(|pc| pc.effective_allowed(&subj, &pid))
                                    .unwrap_or(false);
                                (pid, granted)
                            })
                            .collect()
                    } else {
                        page.required_module()
                            .and_then(|m| ModulesConfig::manifest_for(m))
                            .map(|manifest| {
                                let subj = PermissionSubject::module(manifest.name);
                                manifest
                                    .required_permissions
                                    .iter()
                                    .map(|pid| {
                                        let granted = perm_cfg
                                            .as_ref()
                                            .map(|pc| pc.effective_allowed(&subj, pid))
                                            .unwrap_or(false);
                                        ((*pid).to_string(), granted)
                                    })
                                    .collect()
                            })
                            .unwrap_or_default()
                    };
                let has_perms = !perm_rows.is_empty();
                let badge_id = SharedString::from(format!("hub-perm-badge:{}", page_id));
                let icon_col = p.content_meta;
                Some(
                    div()
                        .relative()
                        .flex_1()
                        .min_w(px(220.))
                        .cursor_pointer()
                        .p_4()
                        .rounded(px(r))
                        .bg(p.panel_bg)
                        .border_1()
                        .border_color(pal.icon_idle)
                        .hover(move |s| s.bg(p.row_bg).border_color(pal.icon_active))
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(
                            div()
                                .flex()
                                .gap_3()
                                .items_center()
                                .child(
                                    render_icon(page.glyph())
                                        .size_6()
                                        .text_color(pal.icon_active),
                                )
                                .child(
                                    div()
                                        .text_base()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(p.content_title)
                                        .flex_1()
                                        .child(page.title().to_string()),
                                )
                                .child(
                                    div()
                                        .cursor_pointer()
                                        .opacity(if is_pinned { 1.0 } else { 0.35 })
                                        .child(
                                            render_icon(pin_glyph).size_4().text_color(pin_color),
                                        )
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(move |this, _, _, cx| {
                                                cx.stop_propagation();
                                                this.toggle_settings_pin(&page_id_pin);
                                                cx.notify();
                                            }),
                                        ),
                                ),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(p.content_body)
                                .child(page.description().to_string()),
                        )
                        .when(has_perms, move |d| {
                            d.child(
                                div()
                                    .id(badge_id)
                                    .absolute()
                                    .bottom(px(8.))
                                    .right(px(8.))
                                    .size_4()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .cursor_default()
                                    .child(
                                        render_icon("permissions")
                                            .size_4()
                                            .text_color(icon_col),
                                    )
                                    .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                        cx.stop_propagation();
                                    })
                                    .tooltip(move |_window, cx| {
                                        Tooltip::build(cx, |t| {
                                            perm_rows.iter().fold(t, |t, (pid, granted)| {
                                                let color = if *granted {
                                                    openframe::rgb(0x4ade80)
                                                } else {
                                                    openframe::rgb(0xf87171)
                                                };
                                                t.child(
                                                    openframe::div()
                                                        .text_color(color)
                                                        .child(pid.clone()),
                                                )
                                            })
                                        })
                                    }),
                            )
                        })
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, _, cx| {
                                this.active_page_id = page_id_owned.clone();
                                if page_id_owned.as_str() == "global.modules" {
                                    this.reload_modules();
                                }
                                this.sync_settings_hub_expanded_from_active_page();
                                cx.notify();
                            }),
                        )
                        .into_any_element(),
                )
            })
            .collect();

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_6()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_3xl()
                            .font_weight(FontWeight::BOLD)
                            .text_color(p.content_title)
                            .child(
                                hub.map(|h| h.title().to_string())
                                    .unwrap_or_else(|| "Settings".into()),
                            ),
                    )
                    .child(
                        div().text_sm().text_color(p.content_meta).child(
                            hub.map(|h| h.description().to_string())
                                .unwrap_or_else(|| "Choose a settings area.".into()),
                        ),
                    ),
            )
            .child(if tiles.is_empty() {
                div()
                    .text_sm()
                    .text_color(p.content_meta)
                    .child("No settings pages are available right now.")
                    .into_any_element()
            } else {
                {
                    // Phantom spacers keep last-row tiles the same width as full rows.
                    // 5 spacers covers up to 6 columns; they are invisible and non-interactive.
                    let phantoms: Vec<_> = (0..5)
                        .map(|_| div().flex_1().min_w(px(220.)).into_any_element())
                        .collect();
                    div()
                        .flex()
                        .flex_row()
                        .flex_wrap()
                        .gap_4()
                        .children(tiles)
                        .children(phantoms)
                        .into_any_element()
                }
            })
    }

    pub fn is_page_visible(&self, page_id: &str) -> bool {
        if page_id == "ai.models" {
            return self.is_module_enabled(AI_MODULE_NAME)
                && any_ai_provider_enabled(&self.module_rows);
        }
        if let Some(nav) = self.navigation_registry() {
            return navigation::is_page_visible_in_owned(nav, page_id, |name| {
                self.is_module_enabled(name)
            });
        }
        navigation::is_page_visible_with(page_id, |name| self.is_module_enabled(name))
    }

    pub fn active_page_if_visible(&self) -> Option<NavPageRef<'_>> {
        if self.is_page_visible(self.active_page_id.as_str()) {
            self.page_ref(self.active_page_id.as_str())
        } else {
            None
        }
    }

    pub fn ensure_valid_navigation_selection(&mut self) {
        let group_fix = {
            let visible_groups = self.visible_groups_effective();
            let group_is_visible = visible_groups
                .iter()
                .any(|group| group.id() == self.active_group_id.as_str());
            if group_is_visible {
                None
            } else if let Some(group) = visible_groups.first() {
                Some(group.id().to_string())
            } else if let Some(nav) = self.navigation_registry() {
                Some(nav.default_group.clone())
            } else {
                Some(navigation::DEFAULT_GROUP_ID.to_string())
            }
        };
        if let Some(g) = group_fix {
            self.active_group_id = g;
        }

        let page_fix = {
            let visible_groups = self.visible_groups_effective();
            let active_group = visible_groups
                .iter()
                .find(|group| group.id() == self.active_group_id.as_str())
                .or_else(|| visible_groups.first());
            if self.is_page_visible(self.active_page_id.as_str()) {
                None
            } else if let Some(group) = active_group {
                group
                    .page_ids()
                    .into_iter()
                    .find(|page_id| self.is_page_visible(page_id))
                    .map(|s| s.to_string())
            } else {
                None
            }
        };
        if let Some(p) = page_fix {
            self.active_page_id = p;
            self.sync_settings_hub_expanded_from_active_page();
        }
    }
}

fn collect_explorer_entries(
    dir: &str,
    depth: usize,
    expanded: &std::collections::HashSet<String>,
    py_only: bool,
    out: &mut Vec<(String, String, bool, usize)>,
) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<(String, bool)> = rd
        .flatten()
        .map(|e| {
            let is_dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
            (e.file_name().to_string_lossy().to_string(), is_dir)
        })
        .filter(|(name, _)| !name.starts_with('.'))
        .filter(|(name, is_dir)| *is_dir || !py_only || name.ends_with(".py"))
        .collect();
    entries.sort_by(|(a, a_dir), (b, b_dir)| b_dir.cmp(a_dir).then(a.cmp(b)));
    for (name, is_dir) in entries {
        let full = format!("{}/{}", dir, name);
        out.push((full.clone(), name, is_dir, depth));
        if is_dir && expanded.contains(&full) {
            collect_explorer_entries(&full, depth + 1, expanded, py_only, out);
        }
    }
}

fn file_icon_for(name: &str) -> String {
    use arcadia_core::modules::python_registry;
    if let Some(icon) = python_registry::call_file_icon_providers(name) {
        return icon;
    }
    let ext = name.rsplit('.').next().unwrap_or("");
    match ext {
        "rs" | "js" | "ts" | "jsx" | "tsx" | "py" | "go" | "java" | "c" | "cpp" | "h" | "hpp"
        | "cs" | "rb" | "php" | "swift" | "kt" | "sh" | "bash" | "zsh" | "html" | "css"
        | "scss" | "sass" | "json" | "toml" | "yaml" | "yml" | "xml" | "sql" | "lua" | "r"
        | "dart" | "ex" | "exs" => "file-code".to_string(),
        "md" | "txt" | "rst" | "org" | "tex" | "log" => "file-text".to_string(),
        _ => "file".to_string(),
    }
}
