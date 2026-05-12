use serde::{Deserialize, Serialize};

use crate::config::ConfigFile;

const FILE_NAME: &str = "ai.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub default_system_prompt: String,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            default_system_prompt: String::new(),
        }
    }
}

impl ConfigFile for AiConfig {
    fn file_name() -> &'static str {
        FILE_NAME
    }
}
