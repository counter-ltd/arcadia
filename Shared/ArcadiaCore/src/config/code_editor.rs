use serde::{Deserialize, Serialize};

use crate::config::ConfigFile;

const FILE_NAME: &str = "code-editor.toml";

fn default_true() -> bool {
    true
}

/// Shape of the text caret in the code editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CursorStyle {
    /// Filled block covering the character (inverts it).
    #[default]
    Block,
    /// Thin vertical bar before the character.
    Bar,
    /// Line under the character.
    Underline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeEditorConfig {
    pub show_indentation_marks: bool,
    /// Manual override for monospace character advance width in pixels.
    /// `None` = auto-measure from font metrics at render time.
    #[serde(default)]
    pub char_width_override: Option<f32>,
    /// Copy the previous line's leading whitespace onto a new line.
    #[serde(default = "default_true")]
    pub auto_indent: bool,
    /// Insert the closing partner when an opening bracket/quote is typed.
    #[serde(default = "default_true")]
    pub auto_close_brackets: bool,
    /// Enable the per-tab undo/redo history.
    #[serde(default = "default_true")]
    pub undo_enabled: bool,
    /// Enable the editor line commands (duplicate/move/comment/indent…).
    #[serde(default = "default_true")]
    pub line_commands: bool,
    /// Shape of the text caret.
    #[serde(default)]
    pub cursor_style: CursorStyle,
}

impl Default for CodeEditorConfig {
    fn default() -> Self {
        Self {
            show_indentation_marks: false,
            char_width_override: None,
            auto_indent: true,
            auto_close_brackets: true,
            undo_enabled: true,
            line_commands: true,
            cursor_style: CursorStyle::Block,
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
