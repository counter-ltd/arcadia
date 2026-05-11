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
        // Style id renames for the same Python extension: terminal → flux → shell → terminal
        // (back to the original name to match the native `terminal` module). All old ids
        // collapse onto `terminal`.
        match self.active_style.as_str() {
            "flux" | "shell" => {
                self.active_style = "terminal".to_string();
                true
            }
            _ => false,
        }
    }
}
