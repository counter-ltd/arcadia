use serde::{Deserialize, Serialize};

use crate::config::ConfigFile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    pub active_style: String,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self { active_style: "default".to_string() }
    }
}

impl ConfigFile for AppearanceConfig {
    fn file_name() -> &'static str {
        "appearance.toml"
    }

    fn merge_defaults(&mut self) -> bool {
        // Style id renames: Terminal → Flux → Shell (same Python extension).
        match self.active_style.as_str() {
            "terminal" | "flux" => {
                self.active_style = "shell".to_string();
                true
            }
            _ => false,
        }
    }
}
