use serde::{Deserialize, Serialize};

use crate::config::ConfigFile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    pub active_style: String,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            active_style: "default".to_string(),
        }
    }
}

impl ConfigFile for AppearanceConfig {
    fn file_name() -> &'static str {
        "appearance.toml"
    }

    fn merge_defaults(&mut self) -> bool {
        // Active-style renames for the terminal-theme extension (short-form ids stored in
        // appearance.toml, distinct from the full extension IDs in `config::modules`).
        // "flux" and "shell" are dead names — never reuse.
        match self.active_style.as_str() {
            "flux" | "shell" => {
                self.active_style = "terminal".to_string();
                true
            }
            _ => false,
        }
    }
}
