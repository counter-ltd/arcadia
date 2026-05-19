//! Extension system — modules as self-registering Rust extensions.
//!
//! An [`Extension`] owns *all* of its contributions: manifest, commands, navigation
//! pages, services, permissions, shortcuts, icons, and exported APIs. Core files
//! stop describing modules in static arrays; the [`collector`] gathers extensions
//! from every [`provider::ExtensionProvider`] and builds the runtime registries.
//!
//! Two extension kinds share this contract:
//! - **Built-in Rust modules** — compiled in, registered via `inventory` at link time.
//! - **Runtime modules** (future) — loaded from `~/Arcadia/Modules/` by
//!   [`provider::ModulesDirProvider`].
//!
//! The Python extension host is itself just another `Extension`.

pub mod api;
pub mod collector;
pub mod provider;
pub mod shell;
pub mod types;

pub use types::{
    ApiContract, ModuleSource, NavPlacement, OwnedIcon, OwnedModuleManifest,
    OwnedWorkspacePermissionDef,
};

use crate::config::permissions::PermissionDefinition;
use crate::modules::ModuleCommand;
use crate::navigation::{NavigationGroupDefinition, NavigationPageDefinition};
use crate::services::ServiceDefinition;
use crate::shortcuts::ShortcutDefinition;

/// A self-contained unit of app functionality. Implemented by built-in Rust
/// modules and by the host shims for runtime-loaded modules / Python extensions.
///
/// Every method except [`Extension::manifest`] has an empty default — a module
/// implements only the contribution kinds it actually provides.
///
/// Identity and commands are owned (a runtime module must produce them);
/// navigation, permissions, shortcuts, and services are `&'static` slices —
/// only built-in modules contribute those.
pub trait Extension: Send + Sync {
    /// Identity, metadata, dependencies, and exported APIs. Required.
    fn manifest(&self) -> OwnedModuleManifest;

    /// Where this extension came from. Built-ins return [`ModuleSource::BuiltIn`].
    fn source(&self) -> ModuleSource {
        ModuleSource::BuiltIn
    }

    /// Whether this extension is a toggleable module that belongs in the module
    /// registry. The app shell returns `false` — it contributes navigation and
    /// permissions but is not a user-facing module.
    fn is_registry_module(&self) -> bool {
        true
    }

    /// `module.verb` commands this extension dispatches.
    fn commands(&self) -> &'static [ModuleCommand] {
        &[]
    }

    /// Navigation pages this extension contributes.
    fn nav_pages(&self) -> &'static [NavigationPageDefinition] {
        &[]
    }

    /// Navigation groups this extension contributes (typically only the shell).
    fn nav_groups(&self) -> &'static [NavigationGroupDefinition] {
        &[]
    }

    /// Navigation frame — placement lists and defaults. Only the app shell
    /// returns `Some`; module extensions leave it `None`.
    fn nav_placement(&self) -> Option<NavPlacement> {
        None
    }

    /// Long-running services advertised on the Services page.
    fn services(&self) -> &'static [ServiceDefinition] {
        &[]
    }

    /// Permission catalog entries this extension owns.
    fn permissions(&self) -> &'static [PermissionDefinition] {
        &[]
    }

    /// Keyboard shortcut contributions.
    fn shortcuts(&self) -> &'static [ShortcutDefinition] {
        &[]
    }

    /// Icons/glyphs this extension ships.
    fn icons(&self) -> Vec<OwnedIcon> {
        Vec::new()
    }

    /// Called once after the collector has ordered all extensions by dependency.
    /// Runs only for enabled extensions. Replaces per-module init in `load_all`.
    fn init(&self) {}

    /// Called on disable / shutdown. Must release any registered API objects.
    fn shutdown(&self) {}
}

/// Link-time registration record. Built-in modules submit one of these via
/// `inventory::submit!`; [`provider::InventoryProvider`] iterates them.
///
/// `make` is a constructor rather than a `&'static dyn Extension` so the trait
/// object is owned (`Box`), uniform with runtime-loaded extensions.
pub struct ExtensionRegistration {
    pub make: fn() -> Box<dyn Extension>,
}

inventory::collect!(ExtensionRegistration);

/// Registers a built-in module as an [`Extension`]. Place once in a module's `mod.rs`:
///
/// ```ignore
/// arcadia_core::register_extension!(LanExtension);
/// ```
#[macro_export]
macro_rules! register_extension {
    ($ty:ty) => {
        $crate::extension::__inventory_submit! {
            $crate::extension::ExtensionRegistration {
                make: || ::std::boxed::Box::new(<$ty>::default()),
            }
        }
    };
}

#[doc(hidden)]
pub use inventory::submit as __inventory_submit;
