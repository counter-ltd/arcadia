use serde::{Deserialize, Serialize};

use crate::config::ConfigFile;
use crate::modules::ai_types::AiModelKind;

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
    Vision,
    Embedding,
}

impl OllamaModelKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::TextGeneration => "Text Generation",
            Self::Vision         => "Vision",
            Self::Embedding      => "Embedding",
        }
    }

    pub fn as_ai_model_kind(&self) -> AiModelKind {
        match self {
            Self::TextGeneration => AiModelKind::TextGeneration,
            Self::Vision         => AiModelKind::Vision,
            Self::Embedding      => AiModelKind::Embedding,
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
}
