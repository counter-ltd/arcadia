use serde::{Deserialize, Serialize};

use crate::config::ConfigFile;
use crate::modules::ai_types::AiModelKind;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpenAiModel {
    pub id: String,
    // "display_name" preserved in TOML for backward compat; Rust API uses name.
    #[serde(alias = "display_name")]
    pub name: String,
    pub model_id: String,
    #[serde(default)]
    pub model_kind: OpenAiModelKind,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OpenAiModelKind {
    #[default]
    TextGeneration,
    ImageGeneration,
    Embedding,
}

impl OpenAiModelKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::TextGeneration => "Text Generation",
            Self::ImageGeneration => "Image Generation",
            Self::Embedding => "Embedding",
        }
    }

    pub fn icon_key(&self) -> &'static str {
        match self {
            Self::TextGeneration => "type-text",
            Self::ImageGeneration => "type-image",
            Self::Embedding => "type-embedding",
        }
    }

    pub fn as_ai_model_kind(&self) -> AiModelKind {
        match self {
            Self::TextGeneration => AiModelKind::TextGeneration,
            Self::ImageGeneration => AiModelKind::ImageGeneration,
            Self::Embedding => AiModelKind::Embedding,
        }
    }
}

/// A single OpenAI-compatible provider instance (distinct URL + API key + model list).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpenAiProvider {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub models: Vec<OpenAiModel>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpenAiConfig {
    #[serde(default)]
    pub providers: Vec<OpenAiProvider>,
    // Legacy fields from the old flat format (single api_key + base_url + models at top level).
    // Read-only for migration — never written back (skip_serializing).
    #[serde(default, skip_serializing, rename = "api_key")]
    legacy_api_key: String,
    #[serde(default, skip_serializing, rename = "base_url")]
    legacy_base_url: String,
    #[serde(default, skip_serializing, rename = "models")]
    legacy_models: Vec<OpenAiModel>,
}

impl Default for OpenAiConfig {
    fn default() -> Self {
        Self {
            providers: Vec::new(),
            legacy_api_key: String::new(),
            legacy_base_url: String::new(),
            legacy_models: Vec::new(),
        }
    }
}

impl ConfigFile for OpenAiConfig {
    fn file_name() -> &'static str {
        "openai.toml"
    }

    fn merge_defaults(&mut self) -> bool {
        // Migrate old flat config (single api_key + base_url + models) to first provider entry.
        if self.providers.is_empty()
            && (!self.legacy_api_key.is_empty() || !self.legacy_models.is_empty())
        {
            use std::time::{SystemTime, UNIX_EPOCH};
            let ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0);
            let base_url = if self.legacy_base_url.is_empty() {
                "https://api.openai.com".to_string()
            } else {
                self.legacy_base_url.clone()
            };
            self.providers.push(OpenAiProvider {
                id: format!("p_{ms}"),
                name: "OpenAI".to_string(),
                api_key: self.legacy_api_key.clone(),
                base_url,
                models: self.legacy_models.clone(),
            });
            return true;
        }
        false
    }
}
