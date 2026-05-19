//! `ApiRegistry` — typed, versioned service locator for modules extending modules.
//!
//! A module that wants to be extensible registers an API object under a
//! `name@version` key; other modules retrieve it instead of `use`-ing the module
//! directly. This kills compile-time inter-module coupling and gives runtime
//! modules a boundary-safe way to reach built-in capabilities.
//!
//! Keyed by `(name, version)` — string, not `TypeId` — so the same dispatch story
//! works for runtime modules. The compiled-in `TypeId` fast path is Phase 5.

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

/// API objects are stored type-erased; callers downcast on retrieval.
type AnyApi = Arc<dyn Any + Send + Sync>;

/// Process-wide registry of module-provided APIs.
pub struct ApiRegistry {
    entries: Mutex<HashMap<(String, u32), AnyApi>>,
}

impl ApiRegistry {
    fn new() -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
        }
    }

    /// Register `api` under `name@version`. Replaces any existing entry for that
    /// exact key (re-registration on module reload is expected).
    pub fn register<T: Any + Send + Sync + 'static>(
        &self,
        name: &str,
        version: u32,
        api: Arc<T>,
    ) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.insert((name.to_string(), version), api);
        }
    }

    /// Retrieve the API registered under `name@version`, downcast to `T`.
    /// `None` if absent or the stored type does not match.
    pub fn get<T: Any + Send + Sync + 'static>(
        &self,
        name: &str,
        version: u32,
    ) -> Option<Arc<T>> {
        let entries = self.entries.lock().ok()?;
        let any = entries.get(&(name.to_string(), version))?.clone();
        any.downcast::<T>().ok()
    }

    /// Remove an entry — used when a module is disabled / unloaded.
    pub fn unregister(&self, name: &str, version: u32) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.remove(&(name.to_string(), version));
        }
    }
}

/// The shared process-wide registry.
pub fn registry() -> &'static ApiRegistry {
    static REGISTRY: OnceLock<ApiRegistry> = OnceLock::new();
    REGISTRY.get_or_init(ApiRegistry::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_get_round_trips() {
        let reg = ApiRegistry::new();
        reg.register("counter", 1, Arc::new(42u32));
        assert_eq!(reg.get::<u32>("counter", 1).map(|a| *a), Some(42));
    }

    #[test]
    fn version_is_part_of_the_key() {
        let reg = ApiRegistry::new();
        reg.register("counter", 1, Arc::new(1u32));
        reg.register("counter", 2, Arc::new(2u32));
        assert_eq!(reg.get::<u32>("counter", 1).map(|a| *a), Some(1));
        assert_eq!(reg.get::<u32>("counter", 2).map(|a| *a), Some(2));
    }

    #[test]
    fn wrong_type_returns_none() {
        let reg = ApiRegistry::new();
        reg.register("x", 1, Arc::new(1u32));
        assert!(reg.get::<String>("x", 1).is_none());
    }

    #[test]
    fn unregister_removes_entry() {
        let reg = ApiRegistry::new();
        reg.register("x", 1, Arc::new(1u32));
        reg.unregister("x", 1);
        assert!(reg.get::<u32>("x", 1).is_none());
    }
}
