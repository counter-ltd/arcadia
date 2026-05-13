use serde::{Deserialize, Serialize};

use crate::config::ConfigFile;

const FILE_NAME: &str = "ai.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    #[serde(default = "default_system_prompt")]
    pub default_system_prompt: String,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: i32,
}

fn default_max_tokens() -> i32 {
    512
}

fn default_system_prompt() -> String {
    "You are a helpful assistant integrated into Arcadia. \
     Be concise and accurate. When working with files, prefer showing diffs \
     over repeating entire file contents."
        .to_string()
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            default_system_prompt: default_system_prompt(),
            max_tokens: default_max_tokens(),
        }
    }
}

impl ConfigFile for AiConfig {
    fn file_name() -> &'static str {
        FILE_NAME
    }

    fn merge_defaults(&mut self) -> bool {
        // Add field migrations here following the pattern in config/modules.rs.
        // Return true if any field was changed so the caller saves the updated config.
        false
    }
}
