use crate::extension::{Extension, OwnedModuleManifest};

use super::ModuleCommand;

pub const NAME: &str = "code-editor";

pub fn commands() -> &'static [ModuleCommand] {
    &[]
}

#[derive(Default)]
pub struct CodeEditorExtension;

impl Extension for CodeEditorExtension {
    fn manifest(&self) -> OwnedModuleManifest {
        OwnedModuleManifest {
            name: NAME.to_string(),
            glyph: "file-code".to_string(),
            version: "0.1.0".to_string(),
            description: "Code editor with per-file tabs in the sidebar.".to_string(),
            accent: String::new(),
            required_modules: Vec::new(),
            required_permissions: Vec::new(),
            workspace_permissions: Vec::new(),
            supported_platforms: Vec::new(),
            api_exports: Vec::new(),
        }
    }

    fn nav_pages(&self) -> &'static [crate::navigation::NavigationPageDefinition] {
        use crate::navigation::{NavigationPageDefinition, PageLayoutKind};
        static PAGES: &[NavigationPageDefinition] = &[
            NavigationPageDefinition {
                id: "editor.main",
                title: "Editor",
                description: "Open and edit files. Each open file appears as a tab in the sidebar.",
                glyph: "file-code",
                system_image: "doc.text",
                accent: "sky",
                required_module: Some(NAME),
                layout_kind: PageLayoutKind::FullHeight,
                blocks_platform_goto: true,
            },
            NavigationPageDefinition {
                id: "editor.settings",
                title: "Editor",
                description: "Code editor preferences — indentation, display, and formatting options.",
                glyph: "file-code",
                system_image: "doc.text",
                accent: "sky",
                required_module: Some(NAME),
                layout_kind: PageLayoutKind::Standard,
                blocks_platform_goto: false,
            },
        ];
        PAGES
    }

    fn shortcuts(&self) -> &'static [crate::shortcuts::ShortcutDefinition] {
        use crate::shortcuts::{
            ShortcutActionStatic, ShortcutDefinition, ShortcutScopeStatic, ShortcutTriggerStatic,
            ShortcutVisibility,
        };
        // One editor command shortcut, page-scoped to the editor; fires while the
        // editor text area holds focus. Owner stays "arcadia" to match the legacy
        // registry exactly.
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
        static SHORTCUTS: &[ShortcutDefinition] = &[
            editor_shortcut!("editor:save", "Editor — save file", "s", false, false, true, "editor.save"),
            editor_shortcut!("editor:select-line", "Editor — select line", "l", false, false, true, "editor.select_line"),
            editor_shortcut!("editor:duplicate-line", "Editor — duplicate line", "d", false, true, true, "editor.duplicate_line"),
            editor_shortcut!("editor:delete-line", "Editor — delete line", "k", false, true, true, "editor.delete_line"),
            editor_shortcut!("editor:move-line-up", "Editor — move line up", "up", true, false, false, "editor.move_line_up"),
            editor_shortcut!("editor:move-line-down", "Editor — move line down", "down", true, false, false, "editor.move_line_down"),
            editor_shortcut!("editor:indent", "Editor — indent selection", "]", false, false, true, "editor.indent"),
            editor_shortcut!("editor:outdent", "Editor — outdent selection", "[", false, false, true, "editor.outdent"),
            editor_shortcut!("editor:toggle-comment", "Editor — toggle line comment", "/", false, false, true, "editor.toggle_comment"),
            editor_shortcut!("editor:undo", "Editor — undo", "z", false, false, true, "editor.undo"),
            editor_shortcut!("editor:redo", "Editor — redo", "z", false, true, true, "editor.redo"),
        ];
        SHORTCUTS
    }
}

crate::register_extension!(CodeEditorExtension);
