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

    fn is_registry_module(&self) -> bool {
        false
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

    fn permissions(&self) -> &'static [crate::config::permissions::PermissionDefinition] {
        use crate::config::permissions::{PermissionDefinition, SystemGrant};
        static PERMS: &[PermissionDefinition] = &[
            PermissionDefinition {
                id: "input.capture",
                title: "Input capture",
                description: "Reserved for future input capture / injection policy (global gate).",
                default_global: false,
                system_grant: None,
            },
            PermissionDefinition {
                id: "system.accessibility",
                title: "Accessibility (AX)",
                description: "Read and drive other apps' UI via the macOS Accessibility API — focused-app menu extent, status-item boundaries, assistive control. Needs the OS Accessibility grant.",
                default_global: false,
                system_grant: Some(SystemGrant::Accessibility),
            },
            PermissionDefinition {
                id: "system.screen_recording",
                title: "Screen Recording",
                description: "Capture screen contents and other apps' window geometry. Needs the OS Screen Recording grant.",
                default_global: false,
                system_grant: Some(SystemGrant::ScreenRecording),
            },
            PermissionDefinition {
                id: "system.camera",
                title: "Camera",
                description: "Capture video from the camera. Needs the OS Camera grant.",
                default_global: false,
                system_grant: Some(SystemGrant::Camera),
            },
            PermissionDefinition {
                id: "system.microphone",
                title: "Microphone",
                description: "Capture audio from the microphone. Needs the OS Microphone grant.",
                default_global: false,
                system_grant: Some(SystemGrant::Microphone),
            },
            PermissionDefinition {
                id: "system.input_monitoring",
                title: "Input Monitoring",
                description: "Observe keyboard and mouse input system-wide. Needs the OS Input Monitoring grant.",
                default_global: false,
                system_grant: Some(SystemGrant::InputMonitoring),
            },
            PermissionDefinition {
                id: "system.location",
                title: "Location",
                description: "Read the device's location. Needs the OS Location Services grant.",
                default_global: false,
                system_grant: Some(SystemGrant::Location),
            },
            PermissionDefinition {
                id: "system.automation",
                title: "Automation",
                description: "Send Apple Events to control other applications. Needs the OS Automation grant.",
                default_global: false,
                system_grant: Some(SystemGrant::Automation),
            },
            PermissionDefinition {
                id: "system.full_disk_access",
                title: "Full Disk Access",
                description: "Read files in OS-protected locations system-wide. Needs the OS Full Disk Access grant.",
                default_global: false,
                system_grant: Some(SystemGrant::FullDiskAccess),
            },
            PermissionDefinition {
                id: "system.contacts",
                title: "Contacts",
                description: "Read the system address book. Needs the OS Contacts grant.",
                default_global: false,
                system_grant: Some(SystemGrant::Contacts),
            },
            PermissionDefinition {
                id: "system.calendars",
                title: "Calendars",
                description: "Read and write calendar events. Needs the OS Calendars grant.",
                default_global: false,
                system_grant: Some(SystemGrant::Calendars),
            },
            PermissionDefinition {
                id: "system.photos",
                title: "Photos",
                description: "Read the system photo library. Needs the OS Photos grant.",
                default_global: false,
                system_grant: Some(SystemGrant::Photos),
            },
            PermissionDefinition {
                id: "system.reminders",
                title: "Reminders",
                description: "Read and write reminders. Needs the OS Reminders grant.",
                default_global: false,
                system_grant: Some(SystemGrant::Reminders),
            },
            PermissionDefinition {
                id: "system.bluetooth",
                title: "Bluetooth",
                description: "Communicate with Bluetooth devices. Needs the OS Bluetooth grant.",
                default_global: false,
                system_grant: Some(SystemGrant::Bluetooth),
            },
        ];
        PERMS
    }

    fn shortcuts(&self) -> &'static [crate::shortcuts::ShortcutDefinition] {
        use crate::shortcuts::{
            ShortcutActionStatic, ShortcutDefinition, ShortcutScopeStatic, ShortcutTriggerStatic,
            ShortcutTriggerStaticChord, ShortcutVisibility,
        };
        static SHORTCUTS: &[ShortcutDefinition] = &[
            ShortcutDefinition {
                id: "arcadia:dismiss-overlays",
                label: "Dismiss menus and modal overlays",
                owner: "arcadia",
                required_registry_module: None,
                scope: ShortcutScopeStatic::ArcadiaWide,
                visibility: ShortcutVisibility::Both,
                priority: 100,
                consumes: true,
                bypass_text_focus: true,
                system_wide: false,
                triggers: &[ShortcutTriggerStatic::Chord {
                    key: "escape",
                    control: false,
                    alt: false,
                    shift: false,
                    platform: false,
                    function: false,
                }],
                actions: &[ShortcutActionStatic::UiControl {
                    control_id: "arcadia.dismiss_overlays",
                }],
            },
            ShortcutDefinition {
                id: "arcadia:os-global-modules",
                label: "OS-global — Modules (requires consent in Shortcuts settings)",
                owner: "arcadia",
                required_registry_module: None,
                scope: ShortcutScopeStatic::ArcadiaWide,
                visibility: ShortcutVisibility::GlobalPrefsOnly,
                priority: 8,
                consumes: false,
                bypass_text_focus: false,
                system_wide: true,
                triggers: &[ShortcutTriggerStatic::Chord {
                    key: "m",
                    control: true,
                    alt: true,
                    shift: true,
                    platform: true,
                    function: false,
                }],
                actions: &[ShortcutActionStatic::Navigate {
                    page_id: "global.modules",
                }],
            },
            ShortcutDefinition {
                id: "demo:sequence-modules",
                label: "Sample chord sequence — Modules",
                owner: "arcadia",
                required_registry_module: None,
                scope: ShortcutScopeStatic::ArcadiaWide,
                visibility: ShortcutVisibility::GlobalPrefsOnly,
                priority: 5,
                consumes: true,
                bypass_text_focus: false,
                system_wide: false,
                triggers: &[ShortcutTriggerStatic::Sequence(&[
                    ShortcutTriggerStaticChord {
                        key: "comma",
                        control: false,
                        alt: false,
                        shift: false,
                        platform: true,
                        function: false,
                    },
                    ShortcutTriggerStaticChord {
                        key: "m",
                        control: false,
                        alt: false,
                        shift: false,
                        platform: true,
                        function: false,
                    },
                ])],
                actions: &[ShortcutActionStatic::Navigate {
                    page_id: "global.modules",
                }],
            },
            ShortcutDefinition {
                id: "arcadia:command-bar",
                label: "Open command bar",
                owner: "arcadia",
                required_registry_module: None,
                scope: ShortcutScopeStatic::ArcadiaWide,
                visibility: ShortcutVisibility::Both,
                priority: 95,
                consumes: true,
                bypass_text_focus: false,
                system_wide: false,
                triggers: &[ShortcutTriggerStatic::Chord {
                    key: "~",
                    control: false,
                    alt: false,
                    shift: false,
                    platform: false,
                    function: false,
                }],
                actions: &[ShortcutActionStatic::UiControl {
                    control_id: "arcadia.toggle_command_bar",
                }],
            },
        ];
        SHORTCUTS
    }
}

crate::register_extension!(ShellExtension);
