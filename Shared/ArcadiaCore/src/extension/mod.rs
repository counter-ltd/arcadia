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
pub mod types;

pub use types::{
    ApiContract, CommandHandler, ModuleSource, OwnedIcon, OwnedModuleCommand,
    OwnedModuleManifest, OwnedNavPage, OwnedPermission, OwnedService, OwnedShortcut,
    OwnedWorkspacePermissionDef,
};

/// A self-contained unit of app functionality. Implemented by built-in Rust
/// modules and by the host shims for runtime-loaded modules / Python extensions.
///
/// Every method except [`Extension::manifest`] has an empty default — a module
/// implements only the contribution kinds it actually provides.
pub trait Extension: Send + Sync {
    /// Identity, metadata, dependencies, and exported APIs. Required.
    fn manifest(&self) -> OwnedModuleManifest;

    /// Where this extension came from. Built-ins return [`ModuleSource::BuiltIn`].
    fn source(&self) -> ModuleSource {
        ModuleSource::BuiltIn
    }

    /// `module.verb` commands this extension dispatches.
    fn commands(&self) -> Vec<OwnedModuleCommand> {
        Vec::new()
    }

    /// Navigation pages this extension contributes.
    fn nav_pages(&self) -> Vec<OwnedNavPage> {
        Vec::new()
    }

    /// Long-running services advertised on the Services page.
    fn services(&self) -> Vec<OwnedService> {
        Vec::new()
    }

    /// Permission catalog entries this extension owns.
    fn permissions(&self) -> Vec<OwnedPermission> {
        Vec::new()
    }

    /// Keyboard shortcut contributions.
    fn shortcuts(&self) -> Vec<OwnedShortcut> {
        Vec::new()
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
