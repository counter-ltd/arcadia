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
}

crate::register_extension!(CodeEditorExtension);
