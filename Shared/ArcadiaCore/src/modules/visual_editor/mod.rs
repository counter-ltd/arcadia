use super::ModuleCommand;

pub mod blocks;
pub mod codegen;
pub mod palette;
pub mod parse;

pub const NAME: &str = "visual-editor";

pub fn commands() -> &'static [ModuleCommand] {
    &[]
}

#[derive(Default)]
pub struct VisualEditorExtension;

impl crate::extension::Extension for VisualEditorExtension {
    fn manifest(&self) -> crate::extension::OwnedModuleManifest {
        crate::extension::OwnedModuleManifest {
            name: NAME.to_string(),
            glyph: "blocks".to_string(),
            version: "0.1.0".to_string(),
            description:
                "Scratch-style visual block editor for Python, with per-file tabs in the sidebar."
                    .to_string(),
            accent: String::new(),
            required_modules: Vec::new(),
            required_permissions: Vec::new(),
            workspace_permissions: Vec::new(),
            supported_platforms: Vec::new(),
            api_exports: Vec::new(),
        }
    }
}

crate::register_extension!(VisualEditorExtension);
