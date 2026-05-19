//! Phase 2 — parity between the runtime extension collector and the legacy
//! static `MODULE_REGISTRY`.
//!
//! During the module → extension migration (Phase 3) modules move one at a time
//! from `MODULE_REGISTRY` into self-registered `Extension` impls. Two guards keep
//! that migration honest:
//!
//! - [`collector_matches_legacy_for_migrated_modules`] — for every module the
//!   collector *does* produce, its manifest must equal the legacy entry exactly.
//!   Green now (collector is empty); stays green as each module migrates. A
//!   migration that changes behaviour breaks this immediately.
//!
//! - [`collector_covers_all_legacy_modules`] — `#[ignore]`d. The migration's
//!   done-signal: when this passes, every legacy module is collector-backed and
//!   `MODULE_REGISTRY` can be deleted (Phase 4).

use arcadia_core::config::modules::{ModuleManifest, MODULE_REGISTRY};
use arcadia_core::extension::collector::collect;
use arcadia_core::extension::provider::default_providers;
use arcadia_core::extension::OwnedModuleManifest;

/// True when an owned manifest matches the legacy static manifest field-for-field.
fn manifests_match(owned: &OwnedModuleManifest, legacy: &ModuleManifest) -> Result<(), String> {
    let mut diffs = Vec::new();
    if owned.name != legacy.name {
        diffs.push(format!("name: {} != {}", owned.name, legacy.name));
    }
    if owned.glyph != legacy.glyph {
        diffs.push(format!("glyph: {} != {}", owned.glyph, legacy.glyph));
    }
    if owned.version != legacy.version {
        diffs.push(format!("version: {} != {}", owned.version, legacy.version));
    }
    if owned.description != legacy.description {
        diffs.push("description".to_string());
    }
    if owned.accent != legacy.accent {
        diffs.push(format!("accent: {} != {}", owned.accent, legacy.accent));
    }
    if owned.required_modules != legacy.required_modules {
        diffs.push(format!(
            "required_modules: {:?} != {:?}",
            owned.required_modules, legacy.required_modules
        ));
    }
    if owned.required_permissions != legacy.required_permissions {
        diffs.push(format!(
            "required_permissions: {:?} != {:?}",
            owned.required_permissions, legacy.required_permissions
        ));
    }
    if owned.supported_platforms != legacy.supported_platforms {
        diffs.push(format!(
            "supported_platforms: {:?} != {:?}",
            owned.supported_platforms, legacy.supported_platforms
        ));
    }
    let owned_ws: Vec<(&str, bool)> = owned
        .workspace_permissions
        .iter()
        .map(|w| (w.id.as_str(), w.default_granted))
        .collect();
    let legacy_ws: Vec<(&str, bool)> = legacy
        .workspace_permissions
        .iter()
        .map(|w| (w.id, w.default_granted))
        .collect();
    if owned_ws != legacy_ws {
        diffs.push(format!(
            "workspace_permissions: {owned_ws:?} != {legacy_ws:?}"
        ));
    }
    if diffs.is_empty() {
        Ok(())
    } else {
        Err(diffs.join("; "))
    }
}

#[test]
fn collector_matches_legacy_for_migrated_modules() {
    let providers = default_providers();
    let collected = collect(&providers).expect("collector must produce a valid extension set");

    for manifest in collected.manifests() {
        let legacy = MODULE_REGISTRY
            .iter()
            .find(|m| m.name == manifest.name)
            .unwrap_or_else(|| {
                panic!(
                    "collector produced module '{}' absent from MODULE_REGISTRY",
                    manifest.name
                )
            });
        if let Err(diff) = manifests_match(&manifest, legacy) {
            panic!(
                "migrated module '{}' diverges from its legacy manifest: {diff}",
                manifest.name
            );
        }
    }
}

/// Modules intentionally not yet migrated to the Extension system.
/// `animation` is held back deliberately — it is being reworked separately.
const MIGRATION_EXCLUSIONS: &[&str] = &["animation"];

#[test]
fn collector_covers_all_legacy_modules_except_exclusions() {
    let providers = default_providers();
    let collected = collect(&providers).expect("collector must produce a valid extension set");
    let collected_names: Vec<String> = collected.module_names();

    let missing: Vec<&str> = MODULE_REGISTRY
        .iter()
        .map(|m| m.name)
        .filter(|name| !collected_names.iter().any(|c| c == name))
        .filter(|name| !MIGRATION_EXCLUSIONS.contains(name))
        .collect();

    assert!(
        missing.is_empty(),
        "modules still only in MODULE_REGISTRY, not yet migrated: {missing:?}"
    );
}
