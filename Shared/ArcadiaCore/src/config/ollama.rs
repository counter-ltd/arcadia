use serde::{Deserialize, Serialize};

use crate::config::ConfigFile;
use crate::ai_types::AiModelKind;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OllamaModel {
    pub id: String,
    pub name: String,
    pub model_tag: String,
    #[serde(default)]
    pub model_kind: OllamaModelKind,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OllamaModelKind {
    #[default]
    TextGeneration,
    ImageGeneration,
    Vision,
    Embedding,
}

impl OllamaModelKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::TextGeneration => "Text Generation",
            Self::ImageGeneration => "Image Generation",
            Self::Vision => "Vision",
            Self::Embedding => "Embedding",
        }
    }

    pub fn icon_key(&self) -> &'static str {
        match self {
            Self::TextGeneration => "type-text",
            Self::ImageGeneration => "type-image",
            Self::Vision => "type-vision",
            Self::Embedding => "type-embedding",
        }
    }

    pub fn as_ai_model_kind(&self) -> AiModelKind {
        match self {
            Self::TextGeneration => AiModelKind::TextGeneration,
            Self::ImageGeneration => AiModelKind::ImageGeneration,
            Self::Vision => AiModelKind::Vision,
            Self::Embedding => AiModelKind::Embedding,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OllamaConfig {
    #[serde(default = "default_endpoint")]
    pub endpoint: String,
    #[serde(default)]
    pub models: Vec<OllamaModel>,
}

fn default_endpoint() -> String {
    "http://localhost:11434".to_string()
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            endpoint: default_endpoint(),
            models: Vec::new(),
        }
    }
}

impl ConfigFile for OllamaConfig {
    fn file_name() -> &'static str {
        "ollama.toml"
    }

    fn merge_defaults(&mut self) -> bool {
        false
    }
}
