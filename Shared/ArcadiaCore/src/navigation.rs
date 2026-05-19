use serde::{Deserialize, Serialize};

/// Describes how the UI host should size and wrap the page content area.
///
/// Added to [`NavigationPageDefinition`] so layout decisions derive from the
/// registry rather than from hardcoded `if active_page_id == …` chains in
/// the render path.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PageLayoutKind {
    /// Standard scrollable/padded content well (`w_full + p_6`).
    #[default]
    Standard,
    /// Edge-to-edge full-height page (flex_1 + h_full, no outer padding).
    FullHeight,
    /// Settings hub landing page — special canvas background and wider padding.
    SettingsHub,
}

use crate::config::modules::PYTHON_HOST_MODULE_NAME;
use crate::modules::python_registry;
use crate::services::{self, ServiceOwned, SERVICE_DEFINITIONS};

#[derive(Clone, Copy, Serialize)]
pub struct NavigationPageDefinition {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub glyph: &'static str,
    pub system_image: &'static str,
    /// Theme key for sidebar-selected fills and icon tint (`Desktop/src/gui/theme.rs`, `AppTheme` on iOS).
    pub accent: &'static str,
    /// When set, the page is shown only if this module is enabled (`MODULE_REGISTRY` name).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_module: Option<&'static str>,
    /// Content area layout contract — drives wrapper selection in the render path.
    #[serde(default)]
    pub layout_kind: PageLayoutKind,
    /// When true, the page handles Cmd+G natively (e.g. code editor Go-to-line) and the
    /// platform goto bar must not intercept that chord on this page.
    #[serde(default)]
    pub blocks_platform_goto: bool,
}

#[derive(Clone, Copy, Serialize)]
pub struct NavigationGroupDefinition {
    pub id: &'static str,
    pub label: &'static str,
    pub glyph: &'static str,
    pub system_image: &'static str,
    pub pages: &'static [&'static str],
    pub accent: &'static str,
}

#[derive(Serialize)]
pub struct NavigationRegistry {
    pub pages: Vec<NavigationPageDefinition>,
    pub groups: Vec<NavigationGroupDefinition>,
    pub global_pages: Vec<&'static str>,
    /// Pages rendered as compact controls in the surface's top bar (e.g. Logs, Extensions, Modules).
    /// Distinct from `global_pages` (sidebar), so each surface can place them appropriately.
    pub top_bar_pages: Vec<&'static str>,
    /// Sidebar "Settings" hub: tapping the parent reveals these pages (same IDs may appear in `top_bar_pages`).
    pub settings_hub_pages: Vec<&'static str>,
    /// Modules that have registered themselves on a service-host page (e.g. LAN Discovery → utility.services).
    pub services: Vec<crate::services::ServiceDefinition>,
    pub default_group: &'static str,
    pub default_page: &'static str,
}

/// Navigation mirrors sent over `surface.snapshot.extra.navigation_registry` (thin clients, mixed versions).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NavigationPageOwned {
    pub id: String,
    pub title: String,
    pub description: String,
    pub glyph: String,
    #[serde(rename = "system_image")]
    pub system_image: String,
    pub accent: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_module: Option<String>,
    #[serde(default)]
    pub layout_kind: PageLayoutKind,
    #[serde(default)]
    pub blocks_platform_goto: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NavigationGroupOwned {
    pub id: String,
    pub label: String,
    pub glyph: String,
    #[serde(rename = "system_image")]
    pub system_image: String,
    pub pages: Vec<String>,
    pub accent: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NavigationRegistryOwned {
    pub pages: Vec<NavigationPageOwned>,
    pub groups: Vec<NavigationGroupOwned>,
    #[serde(rename = "global_pages")]
    pub global_pages: Vec<String>,
    #[serde(rename = "top_bar_pages", default)]
    pub top_bar_pages: Vec<String>,
    #[serde(rename = "settings_hub_pages", default)]
    pub settings_hub_pages: Vec<String>,
    /// Service descriptors mirrored from the host so thin clients render the same Services rows.
    /// Drives service-host page visibility (see [`is_page_visible_with`]).
    #[serde(default)]
    pub services: Vec<ServiceOwned>,
    #[serde(rename = "default_group")]
    pub default_group: String,
    #[serde(rename = "default_page")]
    pub default_page: String,
}

impl NavigationRegistryOwned {
    pub fn from_static_registry() -> Self {
        Self {
            pages: PAGE_DEFINITIONS.iter().map(|p| p.into()).collect(),
            groups: GROUP_DEFINITIONS.iter().map(|g| g.into()).collect(),
            global_pages: GLOBAL_PAGE_IDS.iter().map(|s| (*s).to_string()).collect(),
            top_bar_pages: TOP_BAR_PAGE_IDS.iter().map(|s| (*s).to_string()).collect(),
            settings_hub_pages: SETTINGS_HUB_PAGE_IDS
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            services: SERVICE_DEFINITIONS.iter().map(|s| s.into()).collect(),
            default_group: DEFAULT_GROUP_ID.to_string(),
            default_page: DEFAULT_PAGE_ID.to_string(),
        }
    }

    pub fn services_for_page(&self, page_id: &str) -> Vec<&ServiceOwned> {
        self.services
            .iter()
            .filter(|s| s.page_id == page_id)
            .collect()
    }

    /// True when at least one service is registered to this page in this registry.
    pub fn page_has_services(&self, page_id: &str) -> bool {
        self.services.iter().any(|s| s.page_id == page_id)
    }

    /// Static registry merged with all extension-contributed pages (token settings + nav pages).
    pub fn with_extension_token_settings_merged() -> Self {
        let mut r = Self::from_static_registry();
        r.merge_extension_token_settings_pages();
        r.merge_python_nav_pages();
        r
    }

    pub fn merge_python_nav_pages(&mut self) {
        for decl in python_registry::list_nav_pages() {
            let page_id = extension_nav_page_id(&decl.extension_id);
            if self.pages.iter().any(|p| p.id == page_id) {
                continue;
            }
            self.pages.push(NavigationPageOwned {
                id: page_id.clone(),
                title: decl.title.clone(),
                description: decl.description.clone(),
                glyph: decl.glyph.clone(),
                system_image: decl.system_image.clone(),
                accent: decl.accent.clone(),
                required_module: Some(PYTHON_HOST_MODULE_NAME.to_string()),
                layout_kind: PageLayoutKind::Standard,
                blocks_platform_goto: false,
            });
            if let Some(group) = self.groups.iter_mut().find(|g| g.id == decl.group_id) {
                if !group.pages.contains(&page_id) {
                    group.pages.push(page_id);
                }
            }
        }
    }

    pub fn merge_extension_token_settings_pages(&mut self) {
        for (module_id, specs) in python_registry::standalone_extension_token_modules() {
            if specs.is_empty() {
                continue;
            }
            let id = extension_token_settings_page_id(&module_id);
            if self.pages.iter().any(|p| p.id == id) {
                continue;
            }
            self.pages
                .push(build_extension_token_settings_page_owned(&module_id));
            if !self.settings_hub_pages.iter().any(|p| p == &id) {
                self.settings_hub_pages.push(id);
            }
        }
    }
}

impl From<&NavigationPageDefinition> for NavigationPageOwned {
    fn from(p: &NavigationPageDefinition) -> Self {
        NavigationPageOwned {
            id: p.id.to_string(),
            title: p.title.to_string(),
            description: p.description.to_string(),
            glyph: p.glyph.to_string(),
            system_image: p.system_image.to_string(),
            accent: p.accent.to_string(),
            required_module: p.required_module.map(|s| s.to_string()),
            layout_kind: p.layout_kind,
            blocks_platform_goto: p.blocks_platform_goto,
        }
    }
}

impl From<&NavigationGroupDefinition> for NavigationGroupOwned {
    fn from(g: &NavigationGroupDefinition) -> Self {
        NavigationGroupOwned {
            id: g.id.to_string(),
            label: g.label.to_string(),
            glyph: g.glyph.to_string(),
            system_image: g.system_image.to_string(),
            pages: g.pages.iter().map(|s| (*s).to_string()).collect(),
            accent: g.accent.to_string(),
        }
    }
}


// ─── Navigation registry ────────────────────────────────────────────────────
//
// Pages, groups, and the placement frame are built from the extension
// collector: each module declares its pages via `Extension::nav_pages`; the
// app shell supplies the groups and placement frame. The collector returns
// `&'static` data (module statics), so no leaking is needed.

use std::sync::LazyLock;

struct NavData {
    pages: Vec<NavigationPageDefinition>,
    groups: Vec<NavigationGroupDefinition>,
    placement: crate::extension::NavPlacement,
}

static NAV_DATA: LazyLock<NavData> = LazyLock::new(|| {
    let providers = crate::extension::provider::default_providers();
    let collected = crate::extension::collector::collect(&providers)
        .expect("extension collector must produce a valid navigation set");
    NavData {
        pages: collected.nav_pages().iter().map(|p| **p).collect(),
        groups: collected.nav_groups().iter().map(|g| **g).collect(),
        placement: collected
            .nav_placement()
            .expect("the app shell must supply a navigation placement"),
    }
});

/// All navigation pages — contributed by every extension's `nav_pages()`.
pub static PAGE_DEFINITIONS: LazyLock<&'static [NavigationPageDefinition]> =
    LazyLock::new(|| NAV_DATA.pages.as_slice());

/// All navigation groups — contributed by the app shell.
pub static GROUP_DEFINITIONS: LazyLock<&'static [NavigationGroupDefinition]> =
    LazyLock::new(|| NAV_DATA.groups.as_slice());

/// Pages shown in the sidebar's global section.
pub static GLOBAL_PAGE_IDS: LazyLock<&'static [&'static str]> =
    LazyLock::new(|| NAV_DATA.placement.global_pages);
pub const LOGS_PAGE_ID: &str = "global.logs";
/// Pages rendered as compact top-bar controls.
pub static TOP_BAR_PAGE_IDS: LazyLock<&'static [&'static str]> =
    LazyLock::new(|| NAV_DATA.placement.top_bar_pages);
/// Parent row in the global sidebar is [`SETTINGS_HUB_ROOT_PAGE_ID`]; the
/// settings-hub pages nest under it. Logs is opened from the app-title context
/// menu on Desktop, not the top bar.
pub const SETTINGS_HUB_ROOT_PAGE_ID: &str = "global.settings";
/// Pages nested under the sidebar Settings hub.
pub static SETTINGS_HUB_PAGE_IDS: LazyLock<&'static [&'static str]> =
    LazyLock::new(|| NAV_DATA.placement.settings_hub_pages);
pub static DEFAULT_GROUP_ID: LazyLock<&'static str> =
    LazyLock::new(|| NAV_DATA.placement.default_group);
pub static DEFAULT_PAGE_ID: LazyLock<&'static str> =
    LazyLock::new(|| NAV_DATA.placement.default_page);

/// Settings hub pages for extensions with standalone `register_tokens`.
pub const EXTENSION_TOKEN_SETTINGS_PAGE_PREFIX: &str = "extension.tokens|";

/// Nav group pages declared dynamically by extensions via `register_nav_page`.
pub const EXTENSION_NAV_PAGE_PREFIX: &str = "extension.page|";

pub fn extension_nav_page_id(extension_id: &str) -> String {
    format!("{EXTENSION_NAV_PAGE_PREFIX}{extension_id}")
}

pub fn parse_extension_nav_page_id(page_id: &str) -> Option<&str> {
    page_id.strip_prefix(EXTENSION_NAV_PAGE_PREFIX)
}

pub fn extension_token_settings_page_id(module: &str) -> String {
    format!("{EXTENSION_TOKEN_SETTINGS_PAGE_PREFIX}{module}")
}

pub fn parse_extension_token_settings_page_id(page_id: &str) -> Option<&str> {
    page_id.strip_prefix(EXTENSION_TOKEN_SETTINGS_PAGE_PREFIX)
}

fn humanize_extension_module_id(id: &str) -> String {
    id.split(|c: char| c == '-' || c == '_')
        .filter(|s| !s.is_empty())
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().chain(c).collect(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn extension_token_settings_page_accent(module_id: &str) -> &'static str {
    const PALETTE: &[&str] = &[
        "amber", "emerald", "sky", "violet", "rose", "teal", "fuchsia",
    ];
    let mut h: usize = 0;
    for b in module_id.bytes() {
        h = h.wrapping_mul(31).wrapping_add(b as usize);
    }
    PALETTE[h % PALETTE.len()]
}

fn build_extension_token_settings_page_owned(module_id: &str) -> NavigationPageOwned {
    let title = humanize_extension_module_id(module_id);
    let description = python_registry::list_modules()
        .into_iter()
        .find(|(n, _, _, _, _, _, _)| n == module_id)
        .and_then(|(_, _, d, _, _, _, _)| {
            let t = d.trim();
            if t.is_empty() || t == "(not loaded)" {
                None
            } else {
                Some(t.replace('\n', " ").chars().take(180).collect::<String>())
            }
        })
        .unwrap_or_else(|| {
            format!("Token overrides for the {title} extension — same keys as register_tokens.")
        });

    NavigationPageOwned {
        id: extension_token_settings_page_id(module_id),
        title,
        description,
        glyph: {
            let icon_path = python_registry::resolve_extension_asset_path(module_id, "icon.svg");
            if icon_path.map(|p| p.exists()).unwrap_or(false) {
                format!("extension-icon/{module_id}")
            } else {
                "extensions".to_string()
            }
        },
        system_image: "slider.horizontal.3".to_string(),
        accent: extension_token_settings_page_accent(module_id).to_string(),
        required_module: Some(PYTHON_HOST_MODULE_NAME.to_string()),
        layout_kind: PageLayoutKind::Standard,
        blocks_platform_goto: false,
    }
}

pub fn page_by_id(page_id: &str) -> Option<&'static NavigationPageDefinition> {
    PAGE_DEFINITIONS.iter().find(|page| page.id == page_id)
}

pub fn group_by_id(group_id: &str) -> Option<&'static NavigationGroupDefinition> {
    GROUP_DEFINITIONS.iter().find(|group| group.id == group_id)
}

pub fn default_navigation_registry() -> NavigationRegistry {
    NavigationRegistry {
        pages: PAGE_DEFINITIONS.to_vec(),
        groups: GROUP_DEFINITIONS.to_vec(),
        global_pages: GLOBAL_PAGE_IDS.to_vec(),
        top_bar_pages: TOP_BAR_PAGE_IDS.to_vec(),
        settings_hub_pages: SETTINGS_HUB_PAGE_IDS.to_vec(),
        services: SERVICE_DEFINITIONS.to_vec(),
        default_group: *DEFAULT_GROUP_ID,
        default_page: *DEFAULT_PAGE_ID,
    }
}

pub fn default_navigation_registry_json() -> String {
    serde_json::to_string(&NavigationRegistryOwned::from_static_registry())
        .expect("navigation registry serialization should always succeed")
}

/// Resolve page visibility from the static registry. Service-host pages (any page that has at
/// least one entry in [`SERVICE_DEFINITIONS`]) become visible iff one of those services has its
/// `required_module` enabled — the page's own `required_module` is ignored. Other pages fall
/// back to their declared `required_module`.
pub fn is_page_visible_with<F>(page_id: &str, is_module_enabled: F) -> bool
where
    F: Fn(&str) -> bool,
{
    let Some(page) = page_by_id(page_id) else {
        return false;
    };
    if services::page_has_services(page_id) {
        return services::services_for_page(page_id)
            .iter()
            .any(|service| is_module_enabled(service.required_module));
    }
    match page.required_module {
        Some(module_name) => is_module_enabled(module_name),
        None => true,
    }
}

/// Same as [`is_page_visible_with`] but resolves the page (and its services) against an owned
/// registry — used by clients consuming a host's `surface.snapshot` payload.
pub fn is_page_visible_in_owned<F>(
    registry: &NavigationRegistryOwned,
    page_id: &str,
    is_module_enabled: F,
) -> bool
where
    F: Fn(&str) -> bool,
{
    let Some(page) = registry.pages.iter().find(|p| p.id == page_id) else {
        return false;
    };
    if registry.page_has_services(page_id) {
        return registry
            .services_for_page(page_id)
            .iter()
            .any(|service| is_module_enabled(&service.required_module));
    }
    match page.required_module.as_deref() {
        Some(module_name) => is_module_enabled(module_name),
        None => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_serializes_to_valid_json() {
        let json = default_navigation_registry_json();
        assert!(!json.is_empty());
        let v: serde_json::Value = serde_json::from_str(&json).expect("must be valid JSON");
        assert!(v.is_object());
        assert!(v["pages"].is_array());
        assert!(v["groups"].is_array());
    }

    #[test]
    fn registry_round_trips_through_json() {
        let original = NavigationRegistryOwned::from_static_registry();
        let json = serde_json::to_string(&original).unwrap();
        let back: NavigationRegistryOwned = serde_json::from_str(&json).unwrap();
        assert_eq!(original.pages.len(), back.pages.len());
        assert_eq!(original.groups.len(), back.groups.len());
        assert_eq!(original.settings_hub_pages, back.settings_hub_pages);
        assert_eq!(original.default_page, back.default_page);
        assert_eq!(original.default_group, back.default_group);
    }

    #[test]
    fn page_by_id_finds_shell() {
        let page = page_by_id("utility.shell").expect("utility.shell must exist");
        assert_eq!(page.id, "utility.shell");
        assert_eq!(
            page.required_module,
            Some(crate::config::modules::TERMINAL_MODULE_NAME)
        );
    }

    #[test]
    fn page_by_id_unknown_returns_none() {
        assert!(page_by_id("does.not.exist").is_none());
    }

    #[test]
    fn extension_token_settings_page_id_round_trips() {
        let id = extension_token_settings_page_id("googly-eyes");
        assert_eq!(
            parse_extension_token_settings_page_id(&id),
            Some("googly-eyes")
        );
        assert!(parse_extension_token_settings_page_id("global.settings").is_none());
    }

    #[test]
    fn group_by_id_finds_network() {
        let group = group_by_id("network").expect("network group must exist");
        assert!(group.pages.contains(&"network.nodes"));
    }

    #[test]
    fn group_by_id_unknown_returns_none() {
        assert!(group_by_id("ghost-group").is_none());
    }

    #[test]
    fn all_pages_with_required_module_exist_in_registry() {
        use crate::config::modules::ModulesConfig;
        for page in PAGE_DEFINITIONS {
            if let Some(module_name) = page.required_module {
                assert!(
                    ModulesConfig::manifest_for(module_name).is_some(),
                    "page '{}' requires module '{}' which is not in MODULE_REGISTRY",
                    page.id,
                    module_name
                );
            }
        }
    }

    #[test]
    fn all_group_pages_exist_in_page_definitions() {
        for group in GROUP_DEFINITIONS {
            for page_id in group.pages {
                assert!(
                    page_by_id(page_id).is_some(),
                    "group '{}' references page '{}' not in PAGE_DEFINITIONS",
                    group.id,
                    page_id
                );
            }
        }
    }

    #[test]
    fn default_page_exists() {
        assert!(
            page_by_id(DEFAULT_PAGE_ID).is_some(),
            "DEFAULT_PAGE_ID '{DEFAULT_PAGE_ID}' not in PAGE_DEFINITIONS"
        );
    }

    #[test]
    fn default_group_exists() {
        assert!(
            group_by_id(DEFAULT_GROUP_ID).is_some(),
            "DEFAULT_GROUP_ID '{DEFAULT_GROUP_ID}' not in GROUP_DEFINITIONS"
        );
    }

    #[test]
    fn all_global_page_ids_exist_in_definitions() {
        for page_id in GLOBAL_PAGE_IDS {
            assert!(
                page_by_id(page_id).is_some(),
                "GLOBAL_PAGE_IDS contains '{page_id}' not in PAGE_DEFINITIONS"
            );
        }
    }

    #[test]
    fn all_top_bar_page_ids_exist_in_definitions() {
        for page_id in TOP_BAR_PAGE_IDS {
            assert!(
                page_by_id(page_id).is_some(),
                "TOP_BAR_PAGE_IDS contains '{page_id}' not in PAGE_DEFINITIONS"
            );
        }
    }

    #[test]
    fn top_bar_pages_disjoint_from_global_pages() {
        for page_id in TOP_BAR_PAGE_IDS {
            assert!(
                !GLOBAL_PAGE_IDS.contains(page_id),
                "page '{page_id}' is in both TOP_BAR_PAGE_IDS and GLOBAL_PAGE_IDS"
            );
        }
    }

    #[test]
    fn top_bar_pages_round_trip_through_json() {
        let json = default_navigation_registry_json();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let arr = v["top_bar_pages"]
            .as_array()
            .expect("top_bar_pages must serialize as array");
        let ids: Vec<String> = arr
            .iter()
            .filter_map(|x| x.as_str().map(String::from))
            .collect();
        let expected: Vec<String> = TOP_BAR_PAGE_IDS.iter().map(|s| (*s).to_string()).collect();
        assert_eq!(
            ids, expected,
            "top_bar_pages JSON must match TOP_BAR_PAGE_IDS"
        );
    }

    #[test]
    fn all_settings_hub_page_ids_exist_in_definitions() {
        for page_id in SETTINGS_HUB_PAGE_IDS {
            assert!(
                page_by_id(page_id).is_some(),
                "SETTINGS_HUB_PAGE_IDS contains '{page_id}' not in PAGE_DEFINITIONS"
            );
        }
    }

    #[test]
    fn settings_hub_pages_round_trip_through_json() {
        let json = default_navigation_registry_json();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let arr = v["settings_hub_pages"]
            .as_array()
            .expect("settings_hub_pages must serialize as array");
        let ids: Vec<String> = arr
            .iter()
            .filter_map(|x| x.as_str().map(String::from))
            .collect();
        let expected: Vec<String> = SETTINGS_HUB_PAGE_IDS
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        assert_eq!(
            ids, expected,
            "settings_hub_pages JSON must match SETTINGS_HUB_PAGE_IDS"
        );
    }

    #[test]
    fn services_round_trip_through_json() {
        let json = default_navigation_registry_json();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let arr = v["services"]
            .as_array()
            .expect("services must serialize as array");
        assert!(
            !arr.is_empty(),
            "services array must include at least lan.discovery"
        );
        let ids: Vec<String> = arr
            .iter()
            .filter_map(|s| s["id"].as_str().map(String::from))
            .collect();
        assert!(ids.contains(&"lan.discovery".to_string()));
    }

    #[test]
    fn service_host_page_visibility_follows_required_module() {
        let visible_when =
            |module: &str| is_page_visible_with("utility.services", |name| name == module);
        assert!(visible_when(crate::config::modules::LAN_MODULE_NAME));
        assert!(!visible_when(crate::config::modules::TERMINAL_MODULE_NAME));
        assert!(!is_page_visible_with("utility.services", |_| false));
    }

    #[test]
    fn non_service_page_visibility_follows_declared_required_module() {
        assert!(is_page_visible_with("utility.shell", |name| {
            name == crate::config::modules::TERMINAL_MODULE_NAME
        }));
        assert!(!is_page_visible_with("utility.shell", |_| false));
        // global.settings has no required_module → always visible.
        assert!(is_page_visible_with("global.settings", |_| false));
    }

    #[test]
    fn unknown_page_id_is_never_visible() {
        assert!(!is_page_visible_with("does.not.exist", |_| true));
    }

    #[test]
    fn is_page_visible_in_owned_matches_static() {
        let registry = NavigationRegistryOwned::from_static_registry();
        let lan_only = |name: &str| name == crate::config::modules::LAN_MODULE_NAME;
        assert!(is_page_visible_in_owned(
            &registry,
            "utility.services",
            lan_only
        ));
        assert!(!is_page_visible_in_owned(
            &registry,
            "utility.services",
            |_| false
        ));
    }
}
