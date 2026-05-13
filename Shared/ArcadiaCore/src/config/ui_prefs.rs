use serde::{Deserialize, Serialize};

use crate::config::ConfigFile;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UiPrefsConfig {
    #[serde(default)]
    pub pinned_settings_pages: Vec<String>,
}

impl ConfigFile for UiPrefsConfig {
    fn file_name() -> &'static str {
        "ui-prefs.toml"
    }

    fn merge_defaults(&mut self) -> bool {
        false
    }
}
