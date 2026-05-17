use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::ConfigFile;

/// Sidecar file name holding free-canvas block positions, written next to the
/// edited `.py` files in the same directory.
const BLOCK_POSITIONS_SIDECAR: &str = ".arcadia-blocks.json";

fn sidecar_path(py_path: &str) -> Option<PathBuf> {
    Path::new(py_path)
        .parent()
        .map(|dir| dir.join(BLOCK_POSITIONS_SIDECAR))
}

fn file_key(py_path: &str) -> String {
    Path::new(py_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| py_path.to_string())
}

/// Load free-canvas (x, y) positions for the top-level blocks of `py_path` from
/// the directory's `.arcadia-blocks.json` sidecar. Empty when none stored.
pub fn load_block_positions(py_path: &str) -> Vec<(f32, f32)> {
    let Some(sidecar) = sidecar_path(py_path) else {
        return Vec::new();
    };
    let Ok(text) = std::fs::read_to_string(&sidecar) else {
        return Vec::new();
    };
    let Ok(map) = serde_json::from_str::<BTreeMap<String, Vec<(f32, f32)>>>(&text) else {
        return Vec::new();
    };
    map.get(&file_key(py_path)).cloned().unwrap_or_default()
}

/// Persist free-canvas positions for `py_path` into the directory sidecar,
/// merging with positions stored for sibling files.
pub fn save_block_positions(py_path: &str, positions: &[(f32, f32)]) {
    let Some(sidecar) = sidecar_path(py_path) else {
        return;
    };
    let mut map: BTreeMap<String, Vec<(f32, f32)>> = std::fs::read_to_string(&sidecar)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default();
    map.insert(file_key(py_path), positions.to_vec());
    if let Ok(json) = serde_json::to_string_pretty(&map) {
        let _ = std::fs::write(&sidecar, json);
    }
}

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
