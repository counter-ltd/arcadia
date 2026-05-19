//! Extension providers — sources the [`collector`](super::collector) draws from.
//!
//! A provider answers one question: "which extensions exist?" The collector merges
//! every provider's output into one set. This is the seam that lets a future
//! runtime loader plug in without touching core: it is just another provider.

use super::{Extension, ExtensionRegistration};

/// A source of extensions. Implementors must be cheap to call repeatedly —
/// the collector may re-`discover` on reload.
pub trait ExtensionProvider {
    /// Short identifier for diagnostics (e.g. `"inventory"`, `"modules-dir"`).
    fn id(&self) -> &'static str;

    /// Produce every extension this provider knows about.
    fn discover(&self) -> Vec<Box<dyn Extension>>;
}

/// Compiled-in extensions: walks the `inventory` registry of
/// [`ExtensionRegistration`] records submitted by built-in modules.
pub struct InventoryProvider;

impl ExtensionProvider for InventoryProvider {
    fn id(&self) -> &'static str {
        "inventory"
    }

    fn discover(&self) -> Vec<Box<dyn Extension>> {
        inventory::iter::<ExtensionRegistration>()
            .map(|reg| (reg.make)())
            .collect()
    }
}

/// Runtime-loaded modules from `~/Arcadia/Modules/`.
///
/// **Phase 1 stub.** The seam exists — `ModuleSource::Loaded`, owned trait returns,
/// `ModulesConfig::module_state` persistence — so the real loader (Phase 6) plugs
/// in here without core changes. Discovers nothing yet.
pub struct ModulesDirProvider;

impl ExtensionProvider for ModulesDirProvider {
    fn id(&self) -> &'static str {
        "modules-dir"
    }

    fn discover(&self) -> Vec<Box<dyn Extension>> {
        Vec::new()
    }
}

/// The providers consulted by [`collector::collect`](super::collector::collect),
/// in priority order. Built-ins first so a runtime module cannot shadow them.
pub fn default_providers() -> Vec<Box<dyn ExtensionProvider>> {
    vec![Box::new(InventoryProvider), Box::new(ModulesDirProvider)]
}
