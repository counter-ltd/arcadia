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
}

crate::register_extension!(CodeEditorExtension);
