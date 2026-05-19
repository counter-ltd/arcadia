use crate::config::modules::AI_MODULE_NAME;
use crate::extension::{Extension, OwnedModuleManifest};

use super::ModuleCommand;

pub const NAME: &str = "ai-provider-openai";

pub fn commands() -> &'static [ModuleCommand] {
    &[]
}

#[derive(Default)]
pub struct OpenAiExtension;

impl Extension for OpenAiExtension {
    fn manifest(&self) -> OwnedModuleManifest {
        OwnedModuleManifest {
            name: NAME.to_string(),
            glyph: "openai".to_string(),
            version: "0.1.0".to_string(),
            description: "OpenAI API provider for the AI chat module.".to_string(),
            accent: "emerald".to_string(),
            required_modules: vec![AI_MODULE_NAME.to_string()],
            required_permissions: Vec::new(),
            workspace_permissions: Vec::new(),
            supported_platforms: Vec::new(),
            api_exports: Vec::new(),
        }
    }
}

crate::register_extension!(OpenAiExtension);
