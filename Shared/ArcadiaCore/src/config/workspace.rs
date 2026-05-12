use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::config::ConfigFile;

#[derive(Debug, Clone, Copy)]
pub struct WorkspacePermissionDef {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub default_granted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceEntry {
    pub id: String,
    pub label: String,
    pub path: String,
    #[serde(default)]
    pub granted_permissions: Vec<String>,
}

impl WorkspaceEntry {
    pub fn has_permission(&self, permission_id: &str) -> bool {
        self.granted_permissions.iter().any(|p| p == permission_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkspacesConfig {
    #[serde(default)]
    pub workspaces: Vec<WorkspaceEntry>,
}

impl ConfigFile for WorkspacesConfig {
    fn file_name() -> &'static str {
        "workspace.toml"
    }
}

pub fn list_workspaces() -> Vec<WorkspaceEntry> {
    WorkspacesConfig::load_or_create()
        .map(|c| c.workspaces)
        .unwrap_or_default()
}

/// First workspace whose path is a prefix of `path`.
pub fn workspace_for_path(path: &str) -> Option<WorkspaceEntry> {
    list_workspaces()
        .into_iter()
        .find(|w| path.starts_with(&w.path))
}

pub fn workspace_has_permission(workspace_id: &str, permission_id: &str) -> bool {
    list_workspaces()
        .iter()
        .find(|w| w.id == workspace_id)
        .map(|w| w.has_permission(permission_id))
        .unwrap_or(false)
}

/// True if any registered workspace covers `path` and has `permission_id` granted.
pub fn any_workspace_grants(path: &str, permission_id: &str) -> bool {
    list_workspaces()
        .iter()
        .any(|w| path.starts_with(&w.path) && w.has_permission(permission_id))
}

pub fn new_workspace_id() -> String {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("ws_{ms}")
}
