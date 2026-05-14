use serde::{Deserialize, Serialize};

use crate::config::ConfigFile;
use crate::modules::ai_types::AiModelKind;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LlamaCppModelKind {
    #[default]
    #[serde(alias = "generation")]
    TextGeneration,
    ImageGeneration,
    Vision,
    Embedding,
}

impl LlamaCppModelKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::TextGeneration  => "Text Generation",
            Self::ImageGeneration => "Image Generation",
            Self::Vision          => "Vision",
            Self::Embedding       => "Embedding",
        }
    }

    pub fn all() -> &'static [LlamaCppModelKind] {
        &[
            LlamaCppModelKind::TextGeneration,
            LlamaCppModelKind::ImageGeneration,
            LlamaCppModelKind::Vision,
            LlamaCppModelKind::Embedding,
        ]
    }

    pub fn icon_key(&self) -> &'static str {
        match self {
            Self::TextGeneration  => "type-text",
            Self::ImageGeneration => "type-image",
            Self::Vision          => "type-vision",
            Self::Embedding       => "type-embedding",
        }
    }

    pub fn as_ai_model_kind(&self) -> AiModelKind {
        match self {
            Self::TextGeneration  => AiModelKind::TextGeneration,
            Self::ImageGeneration => AiModelKind::ImageGeneration,
            Self::Vision          => AiModelKind::Vision,
            Self::Embedding       => AiModelKind::Embedding,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LlamaCppModel {
    pub id: String,
    pub name: String,
    // "model_type" preserved in TOML for backward compat; Rust API uses model_kind.
    #[serde(rename = "model_type")]
    pub model_kind: LlamaCppModelKind,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mmproj_path: Option<String>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct LlamaCppConfig {
    pub models: Vec<LlamaCppModel>,
}

impl ConfigFile for LlamaCppConfig {
    fn file_name() -> &'static str {
        "llama-cpp.toml"
    }

    fn merge_defaults(&mut self) -> bool {
        false
    }
}
