//! Extension collector — merges providers, validates, dependency-orders.
//!
//! Output is the runtime equivalent of the old static `MODULE_REGISTRY`. Where the
//! static array gave compile-time guarantees, the collector enforces them at
//! startup: duplicate names, missing dependencies, dependency cycles, and clashing
//! API exports are all hard errors.

use std::collections::{BTreeMap, BTreeSet};

use super::provider::ExtensionProvider;
use super::{Extension, OwnedModuleManifest};

/// A validation failure produced by [`collect`]. All variants are fatal —
/// the app cannot run with an inconsistent extension set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectError {
    /// Two extensions registered the same module name.
    DuplicateModule(String),
    /// `module` declares a dependency on `missing`, which no provider supplied.
    MissingDependency { module: String, missing: String },
    /// A dependency cycle involving these modules (sorted).
    DependencyCycle(Vec<String>),
    /// Two modules export the same `name@version` API contract.
    DuplicateApiExport { contract: String, modules: Vec<String> },
}

impl std::fmt::Display for CollectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollectError::DuplicateModule(name) => {
                write!(f, "duplicate module '{name}'")
            }
            CollectError::MissingDependency { module, missing } => {
                write!(f, "module '{module}' requires missing module '{missing}'")
            }
            CollectError::DependencyCycle(cycle) => {
                write!(f, "dependency cycle among modules: {}", cycle.join(", "))
            }
            CollectError::DuplicateApiExport { contract, modules } => write!(
                f,
                "API contract '{contract}' exported by multiple modules: {}",
                modules.join(", ")
            ),
        }
    }
}

/// The validated, dependency-ordered extension set.
pub struct CollectedExtensions {
    /// Extensions ordered so every module appears after its dependencies.
    extensions: Vec<Box<dyn Extension>>,
}

impl std::fmt::Debug for CollectedExtensions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CollectedExtensions")
            .field("modules", &self.module_names())
            .finish()
    }
}

impl CollectedExtensions {
    pub fn len(&self) -> usize {
        self.extensions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.extensions.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &dyn Extension> {
        self.extensions.iter().map(|b| b.as_ref())
    }

    /// Manifests in dependency order.
    pub fn manifests(&self) -> Vec<OwnedModuleManifest> {
        self.extensions.iter().map(|e| e.manifest()).collect()
    }

    /// Module names in dependency order.
    pub fn module_names(&self) -> Vec<String> {
        self.extensions.iter().map(|e| e.manifest().name).collect()
    }
}

/// Gather extensions from `providers`, validate, and dependency-order them.
pub fn collect(
    providers: &[Box<dyn ExtensionProvider>],
) -> Result<CollectedExtensions, CollectError> {
    let mut extensions: Vec<Box<dyn Extension>> = Vec::new();
    for provider in providers {
        extensions.extend(provider.discover());
    }

    let manifests: Vec<OwnedModuleManifest> =
        extensions.iter().map(|e| e.manifest()).collect();

    check_duplicates(&manifests)?;
    check_dependencies(&manifests)?;
    check_api_exports(&manifests)?;
    let order = topo_order(&manifests)?;

    // Reorder `extensions` to match the dependency order.
    let mut indexed: Vec<(usize, Box<dyn Extension>)> =
        extensions.into_iter().enumerate().collect();
    let rank: BTreeMap<&str, usize> = order
        .iter()
        .enumerate()
        .map(|(rank, name)| (name.as_str(), rank))
        .collect();
    indexed.sort_by_key(|(orig_idx, ext)| {
        rank.get(ext.manifest().name.as_str())
            .copied()
            .unwrap_or(*orig_idx)
    });
    let extensions = indexed.into_iter().map(|(_, ext)| ext).collect();

    Ok(CollectedExtensions { extensions })
}

fn check_duplicates(manifests: &[OwnedModuleManifest]) -> Result<(), CollectError> {
    let mut seen = BTreeSet::new();
    for m in manifests {
        if !seen.insert(m.name.clone()) {
            return Err(CollectError::DuplicateModule(m.name.clone()));
        }
    }
    Ok(())
}

fn check_dependencies(manifests: &[OwnedModuleManifest]) -> Result<(), CollectError> {
    let names: BTreeSet<&str> = manifests.iter().map(|m| m.name.as_str()).collect();
    for m in manifests {
        for dep in &m.required_modules {
            if !names.contains(dep.as_str()) {
                return Err(CollectError::MissingDependency {
                    module: m.name.clone(),
                    missing: dep.clone(),
                });
            }
        }
    }
    Ok(())
}

fn check_api_exports(manifests: &[OwnedModuleManifest]) -> Result<(), CollectError> {
    let mut owners: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for m in manifests {
        for api in &m.api_exports {
            let key = format!("{}@{}", api.name, api.version);
            owners.entry(key).or_default().push(m.name.clone());
        }
    }
    for (contract, mut modules) in owners {
        if modules.len() > 1 {
            modules.sort();
            return Err(CollectError::DuplicateApiExport { contract, modules });
        }
    }
    Ok(())
}

/// Kahn's algorithm. Returns module names so each appears after its dependencies.
fn topo_order(manifests: &[OwnedModuleManifest]) -> Result<Vec<String>, CollectError> {
    let mut deps: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for m in manifests {
        deps.entry(m.name.clone())
            .or_default()
            .extend(m.required_modules.iter().cloned());
    }

    let mut order = Vec::with_capacity(deps.len());
    // BTreeMap iteration is sorted → deterministic output.
    let mut resolved: BTreeSet<String> = BTreeSet::new();
    loop {
        let ready: Vec<String> = deps
            .iter()
            .filter(|(name, d)| {
                !resolved.contains(*name) && d.iter().all(|x| resolved.contains(x))
            })
            .map(|(name, _)| name.clone())
            .collect();
        if ready.is_empty() {
            break;
        }
        for name in ready {
            resolved.insert(name.clone());
            order.push(name);
        }
    }

    if order.len() != deps.len() {
        let mut cycle: Vec<String> = deps
            .keys()
            .filter(|name| !resolved.contains(*name))
            .cloned()
            .collect();
        cycle.sort();
        return Err(CollectError::DependencyCycle(cycle));
    }
    Ok(order)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extension::types::{ModuleSource, OwnedModuleManifest};
    use crate::extension::Extension;

    struct FakeExt(OwnedModuleManifest);

    impl Extension for FakeExt {
        fn manifest(&self) -> OwnedModuleManifest {
            self.0.clone()
        }
        fn source(&self) -> ModuleSource {
            ModuleSource::BuiltIn
        }
    }

    fn manifest(name: &str, deps: &[&str]) -> OwnedModuleManifest {
        OwnedModuleManifest {
            name: name.to_string(),
            glyph: String::new(),
            version: "0".to_string(),
            description: String::new(),
            accent: String::new(),
            required_modules: deps.iter().map(|d| d.to_string()).collect(),
            required_permissions: Vec::new(),
            workspace_permissions: Vec::new(),
            supported_platforms: Vec::new(),
            api_exports: Vec::new(),
        }
    }

    struct VecProvider(Vec<OwnedModuleManifest>);
    impl ExtensionProvider for VecProvider {
        fn id(&self) -> &'static str {
            "test"
        }
        fn discover(&self) -> Vec<Box<dyn Extension>> {
            self.0
                .iter()
                .map(|m| Box::new(FakeExt(m.clone())) as Box<dyn Extension>)
                .collect()
        }
    }

    fn run(manifests: Vec<OwnedModuleManifest>) -> Result<CollectedExtensions, CollectError> {
        let providers: Vec<Box<dyn ExtensionProvider>> = vec![Box::new(VecProvider(manifests))];
        collect(&providers)
    }

    #[test]
    fn orders_dependencies_before_dependents() {
        let c = run(vec![
            manifest("lan", &["net"]),
            manifest("net", &[]),
        ])
        .unwrap();
        let names = c.module_names();
        let net = names.iter().position(|n| n == "net").unwrap();
        let lan = names.iter().position(|n| n == "lan").unwrap();
        assert!(net < lan, "net must come before lan: {names:?}");
    }

    #[test]
    fn rejects_duplicate_module() {
        let err = run(vec![manifest("a", &[]), manifest("a", &[])]).unwrap_err();
        assert_eq!(err, CollectError::DuplicateModule("a".to_string()));
    }

    #[test]
    fn rejects_missing_dependency() {
        let err = run(vec![manifest("a", &["ghost"])]).unwrap_err();
        assert_eq!(
            err,
            CollectError::MissingDependency {
                module: "a".to_string(),
                missing: "ghost".to_string()
            }
        );
    }

    #[test]
    fn rejects_dependency_cycle() {
        let err = run(vec![manifest("a", &["b"]), manifest("b", &["a"])]).unwrap_err();
        assert!(matches!(err, CollectError::DependencyCycle(_)));
    }
}
