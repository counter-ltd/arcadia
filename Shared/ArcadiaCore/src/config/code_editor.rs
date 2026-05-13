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
