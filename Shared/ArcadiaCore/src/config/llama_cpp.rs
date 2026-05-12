use serde::{Deserialize, Serialize};

use crate::config::ConfigFile;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LlamaCppModelType {
    #[default]
    Generation,
    Vision,
    Embedding,
}

impl LlamaCppModelType {
    pub fn label(&self) -> &'static str {
        match self {
            LlamaCppModelType::Generation => "Generation",
            LlamaCppModelType::Vision => "Vision",
            LlamaCppModelType::Embedding => "Embedding",
        }
    }

    pub fn all() -> &'static [LlamaCppModelType] {
        &[
            LlamaCppModelType::Generation,
            LlamaCppModelType::Vision,
            LlamaCppModelType::Embedding,
        ]
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LlamaCppModel {
    pub id: String,
    pub name: String,
    pub model_type: LlamaCppModelType,
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
}
