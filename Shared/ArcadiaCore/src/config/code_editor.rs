use serde::{Deserialize, Serialize};

use crate::config::ConfigFile;

const FILE_NAME: &str = "code-editor.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeEditorConfig {
    pub show_indentation_marks: bool,
    /// Manual override for monospace character advance width in pixels.
    /// `None` = auto-measure from font metrics at render time.
    #[serde(default)]
    pub char_width_override: Option<f32>,
}

impl Default for CodeEditorConfig {
    fn default() -> Self {
        Self {
            show_indentation_marks: false,
            char_width_override: None,
        }
    }
}

impl ConfigFile for CodeEditorConfig {
    fn file_name() -> &'static str {
        FILE_NAME
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PersistedTab {
    pub id: usize,
    pub title: String,
    #[serde(default)]
    pub file_path: Option<String>,
    #[serde(default)]
    pub workspace_path: Option<String>,
    #[serde(default)]
    pub cursor: usize,
    /// Stored only for unsaved buffers (no `file_path`). File-backed tabs reload from disk.
    #[serde(default)]
    pub unsaved_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CodeEditorSession {
    #[serde(default)]
    pub active_tab: usize,
    #[serde(default)]
    pub next_id: usize,
    #[serde(default)]
    pub tabs: Vec<PersistedTab>,
}

impl ConfigFile for CodeEditorSession {
    fn file_name() -> &'static str {
        "code-editor-session.toml"
    }
}
