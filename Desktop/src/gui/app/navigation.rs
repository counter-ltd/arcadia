use arcadia_core::navigation::{self, NavigationGroupOwned, NavigationPageOwned};
use openframe::{div, rgb, Context, Div, FontWeight, ParentElement, Styled, Window};

use super::ArcadiaRoot;

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
        if let Some(nav) = &self.remote_nav {
            nav.pages
                .iter()
                .find(|p| p.id == page_id)
                .map(NavPageRef::Remote)
        } else {
            navigation::page_by_id(page_id).map(NavPageRef::Static)
        }
    }

    pub(crate) fn effective_group(&self, group_id: &str) -> Option<NavGroupRef<'_>> {
        self.visible_groups_effective()
            .into_iter()
            .find(|g| g.id() == group_id)
    }

    pub(crate) fn global_page_ids_effective(&self) -> Vec<&str> {
        if let Some(nav) = &self.remote_nav {
            nav.global_pages.iter().map(|s| s.as_str()).collect()
        } else {
            navigation::GLOBAL_PAGE_IDS.iter().copied().collect()
        }
    }

    pub(crate) fn top_bar_page_ids_effective(&self) -> Vec<&str> {
        if let Some(nav) = &self.remote_nav {
            nav.top_bar_pages.iter().map(|s| s.as_str()).collect()
        } else {
            navigation::TOP_BAR_PAGE_IDS.iter().copied().collect()
        }
    }

    pub(crate) fn settings_hub_page_ids_effective(&self) -> Vec<&str> {
        if let Some(nav) = &self.remote_nav {
            if nav.settings_hub_pages.is_empty() {
                navigation::SETTINGS_HUB_PAGE_IDS.iter().copied().collect()
            } else {
                nav.settings_hub_pages.iter().map(|s| s.as_str()).collect()
            }
        } else {
            navigation::SETTINGS_HUB_PAGE_IDS.iter().copied().collect()
        }
    }

    /// Expand the Settings hub when the active page is the hub root or a nested registry target.
    pub(crate) fn sync_settings_hub_expanded_from_active_page(&mut self) {
        let active = self.active_page_id.as_str();
        if active == navigation::SETTINGS_HUB_ROOT_PAGE_ID
            || self
                .settings_hub_page_ids_effective()
                .iter()
                .any(|p| *p == active)
        {
            self.settings_hub_expanded = true;
        }
    }

    pub(crate) fn visible_groups_effective(&self) -> Vec<NavGroupRef<'_>> {
        let all: Vec<NavGroupRef<'_>> = if let Some(nav) = &self.remote_nav {
            nav.groups.iter().map(NavGroupRef::Remote).collect()
        } else {
            navigation::GROUP_DEFINITIONS
                .iter()
                .map(NavGroupRef::Static)
                .collect()
        };
        all.into_iter()
            .filter(|g| g.page_ids().iter().any(|pid| self.is_page_visible(pid)))
            .collect()
    }

    pub(crate) fn effective_default_page(&self) -> &str {
        self.remote_nav
            .as_ref()
            .map(|n| n.default_page.as_str())
            .unwrap_or(navigation::DEFAULT_PAGE_ID)
    }

    pub(crate) fn render_active_content(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
        active_page: Option<NavPageRef<'_>>,
        is_dark: bool,
    ) -> Div {
        #[cfg(feature = "gui")]
        if self.active_page_id.as_str() == "utility.shell" {
            return div()
                .flex_1()
                .h_full()
                .min_h_0()
                .p_2()
                .child(self.shell_panel(window, cx));
        }
        if self.active_page_id.as_str() == "global.modules" {
            return div().w_full().p_6().child(self.modules_panel(cx, is_dark));
        }
        if self.active_page_id.as_str() == "network.nodes" {
            return div()
                .w_full()
                .p_6()
                .child(self.lan_nodes_panel(cx, is_dark));
        }
        // TODO: dispatch via page registry; see anti-pattern #3 in AGENTS.md.
        if self.active_page_id.as_str() == "utility.services" {
            return div()
                .w_full()
                .p_6()
                .child(self.services_panel(cx, is_dark));
        }
        if self.active_page_id.as_str() == "late.now_playing" {
            return self.render_late_now_playing(window, cx, is_dark);
        }
        if self.active_page_id.as_str() == "late.settings" {
            return div().w_full().p_6().child(self.late_settings_panel(cx, is_dark));
        }
        if self.active_page_id.as_str() == "python.settings" {
            return div().w_full().p_6().child(self.python_settings_panel(cx, is_dark));
        }
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
                    .text_color(if is_dark {
                        rgb(0xe5e7eb)
                    } else {
                        rgb(0x1f2937)
                    })
                    .child(active_page.map_or_else(|| "Page".to_string(), |page| page.title().to_string())),
            )
            .child(
                div()
                    .text_base()
                    .text_color(if is_dark {
                        rgb(0x9ca3af)
                    } else {
                        rgb(0x6b7280)
                    })
                    .child(active_page.map_or_else(
                        || "Page definition not found.".to_string(),
                        |page| page.description().to_string(),
                    )),
            )
    }

    pub fn is_page_visible(&self, page_id: &str) -> bool {
        if let Some(remote) = &self.remote_nav {
            return navigation::is_page_visible_in_owned(remote, page_id, |name| {
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
            } else if let Some(nav) = &self.remote_nav {
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
        }
    }

}
