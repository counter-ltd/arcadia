//! The Arcadia app shell, modelled as an [`Extension`].
//!
//! The shell owns the navigation frame and the app-level contributions that
//! belong to no single module: the global pages (Settings, Logs, Modules,
//! Appearance, Permissions, Shortcuts), the Services host page, every
//! navigation group, the placement lists, and — in later sub-steps — the
//! `arcadia`-owned shortcuts and app-level permissions.
//!
//! It is a built-in extension like any other, so the collector gathers its
//! contributions through the same path. It is not a toggleable module and is
//! deliberately absent from `MODULE_REGISTRY`.

use crate::extension::{Extension, NavPlacement, OwnedModuleManifest};
use crate::navigation::{NavigationGroupDefinition, NavigationPageDefinition, PageLayoutKind};

/// Manifest name of the shell extension. Not a `MODULE_REGISTRY` module.
pub const SHELL_NAME: &str = "arcadia-shell";

/// Pages owned by the app shell — those with no `required_module`.
static SHELL_PAGES: &[NavigationPageDefinition] = &[
    NavigationPageDefinition {
        id: "utility.services",
        title: "Services",
        description: "Long-running module services advertised by Arcadia (LAN discovery, etc.).",
        glyph: "services",
        system_image: "antenna.radiowaves.left.and.right",
        accent: "amber",
        required_module: None,
        layout_kind: PageLayoutKind::Standard,
        blocks_platform_goto: false,
    },
    NavigationPageDefinition {
        id: "global.logs",
        title: "Logs",
        description: "Recent logs and activity stream appear here.",
        glyph: "logs",
        system_image: "doc.text.magnifyingglass",
        accent: "sky",
        required_module: None,
        layout_kind: PageLayoutKind::Standard,
        blocks_platform_goto: false,
    },
    NavigationPageDefinition {
        id: "global.settings",
        title: "Settings",
        description: "App preferences and configuration controls appear here.",
        glyph: "settings",
        system_image: "gearshape",
        accent: "indigo",
        required_module: None,
        layout_kind: PageLayoutKind::SettingsHub,
        blocks_platform_goto: false,
    },
    NavigationPageDefinition {
        id: "global.modules",
        title: "Modules",
        description: "Manage global module availability and dependency requirements.",
        glyph: "modules",
        system_image: "switch.2",
        accent: "fuchsia",
        required_module: None,
        layout_kind: PageLayoutKind::Standard,
        blocks_platform_goto: false,
    },
    NavigationPageDefinition {
        id: "global.appearance",
        title: "Appearance",
        description: "Theme and display preferences. Placeholder — detailed controls will land here.",
        glyph: "appearance",
        system_image: "paintpalette",
        accent: "indigo",
        required_module: None,
        layout_kind: PageLayoutKind::Standard,
        blocks_platform_goto: false,
    },
    NavigationPageDefinition {
        id: "global.permissions",
        title: "Permissions",
        description: "Global capability toggles and per-module or per-extension grants.",
        glyph: "permissions",
        system_image: "lock.shield",
        accent: "indigo",
        required_module: None,
        layout_kind: PageLayoutKind::Standard,
        blocks_platform_goto: false,
    },
    NavigationPageDefinition {
        id: "global.shortcuts",
        title: "Shortcuts",
        description: "Keyboard, pointer, and OS-global shortcuts; conflicts and overrides.",
        glyph: "shortcuts",
        system_image: "keyboard",
        accent: "indigo",
        required_module: None,
        layout_kind: PageLayoutKind::Standard,
        blocks_platform_goto: false,
    },
];

/// Every navigation group. Groups reference pages by id; the pages themselves
/// are contributed by whichever extension owns them.
static SHELL_GROUPS: &[NavigationGroupDefinition] = &[
    NavigationGroupDefinition {
        id: "utilities",
        label: "Utilities",
        glyph: "tools",
        system_image: "wrench.and.screwdriver",
        pages: &["utility.shell", "utility.services"],
        accent: "amber",
    },
    NavigationGroupDefinition {
        id: "network",
        label: "Network",
        glyph: "network",
        system_image: "network",
        pages: &["network.nodes"],
        accent: "cyan",
    },
    NavigationGroupDefinition {
        id: "social",
        label: "Social",
        glyph: "chat",
        system_image: "bubble.left.and.bubble.right.fill",
        pages: &["late.now_playing", "late.experimental"],
        accent: "teal",
    },
    NavigationGroupDefinition {
        id: "code",
        label: "Code",
        glyph: "file-code",
        system_image: "doc.text",
        pages: &["editor.main", "editor.visual"],
        accent: "sky",
    },
    NavigationGroupDefinition {
        id: "ai",
        label: "AI",
        glyph: "ai-provider",
        system_image: "sparkles",
        pages: &["ai.chat", "ai.models", "ai.rules", "ai.skills"],
        accent: "violet",
    },
];

/// The shell extension. Always present; not user-toggleable.
#[derive(Default)]
pub struct ShellExtension;

impl Extension for ShellExtension {
    fn manifest(&self) -> OwnedModuleManifest {
        OwnedModuleManifest {
            name: SHELL_NAME.to_string(),
            glyph: "settings".to_string(),
            version: "0.1.0".to_string(),
            description: "Arcadia app shell — navigation frame and global pages.".to_string(),
            accent: String::new(),
            required_modules: Vec::new(),
            required_permissions: Vec::new(),
            workspace_permissions: Vec::new(),
            supported_platforms: Vec::new(),
            api_exports: Vec::new(),
        }
    }

    fn nav_pages(&self) -> &'static [NavigationPageDefinition] {
        SHELL_PAGES
    }

    fn nav_groups(&self) -> &'static [NavigationGroupDefinition] {
        SHELL_GROUPS
    }

    fn nav_placement(&self) -> Option<NavPlacement> {
        Some(NavPlacement {
            global_pages: &["global.settings"],
            top_bar_pages: &["extensions.settings", "global.modules", "notification.main"],
            settings_hub_pages: &[
                "global.permissions",
                "global.shortcuts",
                "global.appearance",
                "global.workspaces",
                "late.settings",
                "editor.settings",
                "ai.settings",
                "notification.settings",
            ],
            default_group: "utilities",
            default_page: "global.settings",
        })
    }
}

crate::register_extension!(ShellExtension);
