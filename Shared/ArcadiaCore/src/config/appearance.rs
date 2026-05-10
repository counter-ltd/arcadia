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
}
