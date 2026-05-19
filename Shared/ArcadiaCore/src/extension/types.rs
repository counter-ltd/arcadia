//! Contribution types for the extension system.
//!
//! Identity (`manifest`) is returned **owned** — a runtime `Modules/`-directory
//! extension must be able to produce it. Commands, navigation, permissions,
//! shortcuts, and services are returned as `&'static` slices of the existing
//! static types: only built-in modules contribute those, and runtime modules
//! feed them through dedicated conversion paths (as the Python host already
//! does for its commands and pages).

/// Where an extension came from. Built-ins are always loaded; runtime modules
/// follow the Python discover → stub → enable → body lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleSource {
    /// Compiled into the binary, registered via `inventory`.
    BuiltIn,
    /// Loaded at runtime from a file under `~/Arcadia/Modules/`.
    Loaded { path: std::path::PathBuf },
}

/// A named, versioned API surface a module offers to other modules.
/// Used by the collector to validate cross-module API dependencies, the same
/// way `required_modules` validates module dependencies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiContract {
    pub name: String,
    pub version: u32,
}

/// Owned mirror of [`crate::config::workspace::WorkspacePermissionDef`].
#[derive(Debug, Clone)]
pub struct OwnedWorkspacePermissionDef {
    pub id: String,
    pub title: String,
    pub description: String,
    pub default_granted: bool,
}

/// Owned mirror of [`crate::config::modules::ModuleManifest`], plus `api_exports`.
#[derive(Debug, Clone)]
pub struct OwnedModuleManifest {
    pub name: String,
    pub glyph: String,
    pub version: String,
    pub description: String,
    pub accent: String,
    pub required_modules: Vec<String>,
    pub required_permissions: Vec<String>,
    pub workspace_permissions: Vec<OwnedWorkspacePermissionDef>,
    pub supported_platforms: Vec<String>,
    /// Named+versioned APIs this module registers into the `ApiRegistry`.
    pub api_exports: Vec<ApiContract>,
}

/// Navigation structure owned by the app shell — the placement lists and
/// defaults that are not tied to any single module. Only the `ShellExtension`
/// returns this; module extensions contribute pages and leave it `None`.
///
/// Built-in data, so `&'static` slices: runtime modules contribute pages, not
/// the navigation frame.
#[derive(Debug, Clone, Copy)]
pub struct NavPlacement {
    /// Pages shown in the sidebar's global section.
    pub global_pages: &'static [&'static str],
    /// Pages rendered as compact top-bar controls.
    pub top_bar_pages: &'static [&'static str],
    /// Pages nested under the sidebar Settings hub.
    pub settings_hub_pages: &'static [&'static str],
    pub default_group: &'static str,
    pub default_page: &'static str,
}

/// An icon a module ships. `bytes` is raw SVG/PNG; `key` is the glyph lookup name.
#[derive(Clone)]
pub struct OwnedIcon {
    pub key: String,
    pub bytes: Vec<u8>,
}
