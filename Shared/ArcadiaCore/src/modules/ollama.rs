use crate::config::modules::AI_MODULE_NAME;
use crate::extension::{Extension, OwnedModuleManifest};

use super::ModuleCommand;

pub const NAME: &str = "ai-provider-ollama";

pub fn commands() -> &'static [ModuleCommand] {
    &[]
}

#[derive(Default)]
pub struct OllamaExtension;

impl Extension for OllamaExtension {
    fn manifest(&self) -> OwnedModuleManifest {
        OwnedModuleManifest {
            name: NAME.to_string(),
            glyph: "ollama".to_string(),
            version: "0.1.0".to_string(),
            description: "Ollama local inference provider for the AI chat module."
                .to_string(),
            accent: "cyan".to_string(),
            required_modules: vec![AI_MODULE_NAME.to_string()],
            required_permissions: Vec::new(),
            workspace_permissions: Vec::new(),
            supported_platforms: Vec::new(),
            api_exports: Vec::new(),
        }
    }
}

crate::register_extension!(OllamaExtension);
