use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::config::ConfigFile;

/// Canonicalize `path` for scope comparison.
///
/// If the path itself does not exist, fall back to canonicalizing the parent
/// and joining the file name — this lets us check scope on paths that are
/// about to be created (e.g. new file in a workspace).
fn canonicalize_for_scope(path: &str) -> Option<PathBuf> {
    let p = Path::new(path);
    if let Ok(c) = std::fs::canonicalize(p) {
        return Some(c);
    }
    let parent = p.parent()?;
    let name = p.file_name()?;
    std::fs::canonicalize(parent).ok().map(|c| c.join(name))
}

/// True if `candidate` is within `workspace_path` after canonicalization of both.
///
/// Safe against `..` traversal and symlink escape — never use raw `starts_with`
/// on string paths for security decisions.
fn path_within_workspace(candidate: &str, workspace_path: &str) -> bool {
    let Some(canon_ws) = canonicalize_for_scope(workspace_path) else {
        return false;
    };
    let Some(canon_candidate) = canonicalize_for_scope(candidate) else {
        return false;
    };
    canon_candidate.starts_with(&canon_ws)
}

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

/// First workspace whose canonicalized path covers `path`.
pub fn workspace_for_path(path: &str) -> Option<WorkspaceEntry> {
    list_workspaces()
        .into_iter()
        .find(|w| path_within_workspace(path, &w.path))
}

pub fn workspace_has_permission(workspace_id: &str, permission_id: &str) -> bool {
    list_workspaces()
        .iter()
        .find(|w| w.id == workspace_id)
        .map(|w| w.has_permission(permission_id))
        .unwrap_or(false)
}

/// True if any registered workspace covers `path` and has `permission_id` granted.
///
/// Path comparison uses canonicalize to defeat `..` traversal and symlinks.
pub fn any_workspace_grants(path: &str, permission_id: &str) -> bool {
    list_workspaces()
        .iter()
        .any(|w| w.has_permission(permission_id) && path_within_workspace(path, &w.path))
}

pub fn new_workspace_id() -> String {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("ws_{ms}")
}
