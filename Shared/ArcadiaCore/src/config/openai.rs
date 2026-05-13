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
            Self::TextGeneration  => "Text Generation",
            Self::ImageGeneration => "Image Generation",
            Self::Embedding       => "Embedding",
        }
    }

    pub fn as_ai_model_kind(&self) -> AiModelKind {
        match self {
            Self::TextGeneration  => AiModelKind::TextGeneration,
            Self::ImageGeneration => AiModelKind::ImageGeneration,
            Self::Embedding       => AiModelKind::Embedding,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpenAiConfig {
    // Stored as plaintext in openai.toml. Future: migrate to OS keychain.
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub models: Vec<OpenAiModel>,
}

impl Default for OpenAiConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: "https://api.openai.com".to_string(),
            models: Vec::new(),
        }
    }
}

impl ConfigFile for OpenAiConfig {
    fn file_name() -> &'static str {
        "openai.toml"
    }

    fn merge_defaults(&mut self) -> bool {
        false
    }
}
