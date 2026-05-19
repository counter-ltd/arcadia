//! Owned contribution types for the extension system.
//!
//! Built-in Rust modules have `&'static` data; future `Modules/`-directory modules
//! (loaded at runtime) cannot. The `Extension` trait therefore returns **owned**
//! values — built-ins clone their statics, runtime modules produce them directly.
//! Retrofitting owned returns later would touch every module, so the contract is
//! owned from the start.

use std::sync::Arc;

use crate::modules::{ExecutionContext, ModuleCommand};
use crate::navigation::PageLayoutKind;

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

/// Command handler. A built-in passes a plain `fn` coerced into the `Arc`;
/// a runtime module supplies a closure that bridges to its loaded code.
pub type CommandHandler =
    Arc<dyn Fn(&[&str], &ExecutionContext) -> String + Send + Sync>;

/// Owned mirror of [`crate::modules::ModuleCommand`].
#[derive(Clone)]
pub struct OwnedModuleCommand {
    pub name: String,
    pub description: String,
    pub required_permissions: Vec<String>,
    pub run: CommandHandler,
}

impl OwnedModuleCommand {
    /// Wrap a built-in module's static [`ModuleCommand`] as an owned command.
    /// The `run` function pointer is reused as-is — only the metadata is cloned.
    pub fn from_static(cmd: &ModuleCommand) -> Self {
        let run = cmd.run;
        OwnedModuleCommand {
            name: cmd.name.to_string(),
            description: cmd.description.to_string(),
            required_permissions: cmd
                .required_permissions
                .iter()
                .map(|p| p.to_string())
                .collect(),
            run: Arc::new(move |args, ctx| run(args, ctx)),
        }
    }
}

/// Owned mirror of [`crate::navigation::NavigationPageDefinition`].
#[derive(Debug, Clone)]
pub struct OwnedNavPage {
    pub id: String,
    pub title: String,
    pub description: String,
    pub glyph: String,
    pub system_image: String,
    pub accent: String,
    pub required_module: Option<String>,
    pub layout_kind: PageLayoutKind,
    pub blocks_platform_goto: bool,
}

/// Owned mirror of [`crate::services::ServiceDefinition`] metadata. Runtime control
/// hooks (start/stop) are addressed by command token, not `fn` pointers, so this
/// type stays serializable and boundary-safe.
#[derive(Debug, Clone)]
pub struct OwnedService {
    pub id: String,
    pub page_id: String,
    pub title: String,
    pub description: String,
    pub required_module: String,
    pub glyph: String,
    pub system_image: String,
    pub accent: String,
}

/// Owned mirror of [`crate::config::permissions::PermissionDefinition`]. `system_grant`
/// is carried as its serialized key — the catalog resolves it to a `SystemGrant`.
#[derive(Debug, Clone)]
pub struct OwnedPermission {
    pub id: String,
    pub title: String,
    pub description: String,
    pub default_global: bool,
    pub system_grant: Option<String>,
}

/// Owned shortcut contribution. The JSON spec mirrors the Python extension
/// `register_shortcut_json` payload so both extension systems share a format.
#[derive(Debug, Clone)]
pub struct OwnedShortcut {
    pub json: String,
}

/// An icon a module ships. `bytes` is raw SVG/PNG; `key` is the glyph lookup name.
#[derive(Clone)]
pub struct OwnedIcon {
    pub key: String,
    pub bytes: Vec<u8>,
}
