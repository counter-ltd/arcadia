use serde::{Deserialize, Serialize};

use crate::config::ConfigFile;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PersistedVisualTab {
    pub id: usize,
    pub title: String,
    #[serde(default)]
    pub file_path: Option<String>,
    #[serde(default)]
    pub workspace_path: Option<String>,
    /// Stored only for unsaved buffers (no `file_path`). File-backed tabs reload from disk.
    #[serde(default)]
    pub unsaved_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VisualEditorSession {
    #[serde(default)]
    pub active_tab: usize,
    #[serde(default)]
    pub next_id: usize,
    #[serde(default)]
    pub tabs: Vec<PersistedVisualTab>,
}

impl ConfigFile for VisualEditorSession {
    fn file_name() -> &'static str {
        "visual-editor-session.toml"
    }
}
