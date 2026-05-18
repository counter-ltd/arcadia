//! Static shortcut definitions compiled into core.

use crate::config::modules::{GOTO_MODULE_NAME, TERMINAL_MODULE_NAME};

use super::model::{
    ShortcutActionStatic, ShortcutDefinition, ShortcutScopeStatic, ShortcutTriggerStatic,
    ShortcutTriggerStaticChord, ShortcutVisibility,
};

/// One editor command shortcut. `key`/`alt`/`shift`/`platform` form the chord
/// (`platform` is Cmd on macOS). Page-scoped to the editor; fires while the
/// editor text area holds focus.
macro_rules! editor_shortcut {
    ($id:literal, $label:literal, $key:literal, $alt:literal, $shift:literal, $platform:literal, $ctl:literal) => {
        ShortcutDefinition {
            id: $id,
            label: $label,
            owner: "arcadia",
            required_registry_module: None,
            scope: ShortcutScopeStatic::Pages(&["editor.main"]),
            visibility: ShortcutVisibility::Both,
            priority: 80,
            consumes: true,
            bypass_text_focus: true,
            system_wide: false,
            triggers: &[ShortcutTriggerStatic::Chord {
                key: $key,
                control: false,
                alt: $alt,
                shift: $shift,
                platform: $platform,
                function: false,
            }],
            actions: &[ShortcutActionStatic::UiControl { control_id: $ctl }],
        }
    };
}

pub static SHORTCUT_DEFINITIONS: &[ShortcutDefinition] = &[
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
        id: "terminal:toggle-shell-mode",
        label: "Toggle shell execution mode (generic vs internal)",
        owner: "terminal",
        required_registry_module: Some(TERMINAL_MODULE_NAME),
        scope: ShortcutScopeStatic::Pages(&["utility.shell"]),
        visibility: ShortcutVisibility::Both,
        priority: 90,
        consumes: true,
        bypass_text_focus: false,
        system_wide: false,
        triggers: &[ShortcutTriggerStatic::Chord {
            key: "tab",
            control: false,
            alt: false,
            shift: true,
            platform: false,
            function: false,
        }],
        actions: &[ShortcutActionStatic::UiControl {
            control_id: "terminal.toggle_shell_mode",
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
    // ── Editor command shortcuts (page-scoped, fire over editor text focus) ──
    editor_shortcut!("editor:save", "Editor — save file", "s", false, false, true, "editor.save"),
    editor_shortcut!(
        "editor:select-line",
        "Editor — select line",
        "l",
        false,
        false,
        true,
        "editor.select_line"
    ),
    editor_shortcut!(
        "editor:duplicate-line",
        "Editor — duplicate line",
        "d",
        false,
        true,
        true,
        "editor.duplicate_line"
    ),
    editor_shortcut!(
        "editor:delete-line",
        "Editor — delete line",
        "k",
        false,
        true,
        true,
        "editor.delete_line"
    ),
    editor_shortcut!(
        "editor:move-line-up",
        "Editor — move line up",
        "up",
        true,
        false,
        false,
        "editor.move_line_up"
    ),
    editor_shortcut!(
        "editor:move-line-down",
        "Editor — move line down",
        "down",
        true,
        false,
        false,
        "editor.move_line_down"
    ),
    editor_shortcut!(
        "editor:indent",
        "Editor — indent selection",
        "]",
        false,
        false,
        true,
        "editor.indent"
    ),
    editor_shortcut!(
        "editor:outdent",
        "Editor — outdent selection",
        "[",
        false,
        false,
        true,
        "editor.outdent"
    ),
    editor_shortcut!(
        "editor:toggle-comment",
        "Editor — toggle line comment",
        "/",
        false,
        false,
        true,
        "editor.toggle_comment"
    ),
    editor_shortcut!("editor:undo", "Editor — undo", "z", false, false, true, "editor.undo"),
    editor_shortcut!(
        "editor:redo",
        "Editor — redo",
        "z",
        false,
        true,
        true,
        "editor.redo"
    ),
    ShortcutDefinition {
        id: "goto:open",
        label: "Open goto bar",
        owner: "goto",
        required_registry_module: Some(GOTO_MODULE_NAME),
        scope: ShortcutScopeStatic::ArcadiaWide,
        visibility: ShortcutVisibility::Both,
        priority: 90,
        consumes: true,
        bypass_text_focus: false,
        system_wide: false,
        triggers: &[ShortcutTriggerStatic::Chord {
            key: "g",
            control: false,
            alt: false,
            shift: false,
            platform: true,
            function: false,
        }],
        actions: &[ShortcutActionStatic::UiControl {
            control_id: "goto.open_page",
        }],
    },
];
