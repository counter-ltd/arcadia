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
use arcadia_core::config::permissions::{PermissionDefinition, PERMISSION_REGISTRY};
use arcadia_core::services::SERVICE_DEFINITIONS;
use arcadia_core::shortcuts::SHORTCUT_DEFINITIONS;
use arcadia_core::extension::collector::collect;
use arcadia_core::extension::provider::default_providers;
use arcadia_core::extension::OwnedModuleManifest;
use arcadia_core::navigation::{
    GROUP_DEFINITIONS, GLOBAL_PAGE_IDS, PAGE_DEFINITIONS, SETTINGS_HUB_PAGE_IDS,
    TOP_BAR_PAGE_IDS,
};
use std::collections::BTreeMap;

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

/// Extensions that are not `MODULE_REGISTRY` modules — the app shell owns
/// navigation-frame contributions but is not a toggleable module.
const NON_MODULE_EXTENSIONS: &[&str] = &["arcadia-shell"];

#[test]
fn collector_matches_legacy_for_migrated_modules() {
    let providers = default_providers();
    let collected = collect(&providers).expect("collector must produce a valid extension set");

    for manifest in collected.manifests() {
        // The app shell is an extension but not a MODULE_REGISTRY module.
        if NON_MODULE_EXTENSIONS.contains(&manifest.name.as_str()) {
            continue;
        }
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

/// Modules intentionally not migrated to the Extension system. Empty — every
/// legacy module is now collector-backed. Kept as the hook for any future
/// deliberate exclusion.
const MIGRATION_EXCLUSIONS: &[&str] = &[];

#[test]
fn collector_covers_all_legacy_modules() {
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

#[test]
fn collector_nav_matches_legacy() {
    let providers = default_providers();
    let collected = collect(&providers).expect("collector must produce a valid extension set");

    // Pages — same set, identical field-for-field (compared via serialized form).
    let got_pages: BTreeMap<String, serde_json::Value> = collected
        .nav_pages()
        .iter()
        .map(|p| (p.id.to_string(), serde_json::to_value(p).unwrap()))
        .collect();
    let want_pages: BTreeMap<String, serde_json::Value> = PAGE_DEFINITIONS
        .iter()
        .map(|p| (p.id.to_string(), serde_json::to_value(p).unwrap()))
        .collect();
    assert_eq!(
        got_pages, want_pages,
        "collector nav pages diverge from PAGE_DEFINITIONS"
    );

    // Groups — same set, identical.
    let got_groups: BTreeMap<String, serde_json::Value> = collected
        .nav_groups()
        .iter()
        .map(|g| (g.id.to_string(), serde_json::to_value(g).unwrap()))
        .collect();
    let want_groups: BTreeMap<String, serde_json::Value> = GROUP_DEFINITIONS
        .iter()
        .map(|g| (g.id.to_string(), serde_json::to_value(g).unwrap()))
        .collect();
    assert_eq!(
        got_groups, want_groups,
        "collector nav groups diverge from GROUP_DEFINITIONS"
    );

    // Placement frame.
    let placement = collected
        .nav_placement()
        .expect("the shell extension must supply a nav placement");
    assert_eq!(placement.global_pages, GLOBAL_PAGE_IDS, "global_pages");
    assert_eq!(placement.top_bar_pages, TOP_BAR_PAGE_IDS, "top_bar_pages");
    assert_eq!(
        placement.settings_hub_pages, SETTINGS_HUB_PAGE_IDS,
        "settings_hub_pages"
    );
}

#[test]
fn collector_permissions_match_legacy() {
    let providers = default_providers();
    let collected = collect(&providers).expect("collector must produce a valid extension set");

    let key = |p: &PermissionDefinition| {
        (
            p.title.to_string(),
            p.description.to_string(),
            p.default_global,
            p.system_grant,
        )
    };
    let got: BTreeMap<String, _> = collected
        .permissions()
        .iter()
        .map(|p| (p.id.to_string(), key(p)))
        .collect();
    let want: BTreeMap<String, _> = PERMISSION_REGISTRY
        .iter()
        .map(|p| (p.id.to_string(), key(p)))
        .collect();
    assert_eq!(
        got, want,
        "collector permissions diverge from PERMISSION_REGISTRY"
    );
}

#[test]
fn collector_shortcuts_match_legacy() {
    let providers = default_providers();
    let collected = collect(&providers).expect("collector must produce a valid extension set");

    let got: BTreeMap<String, String> = collected
        .shortcuts()
        .iter()
        .map(|s| (s.id.to_string(), format!("{s:?}")))
        .collect();
    let want: BTreeMap<String, String> = SHORTCUT_DEFINITIONS
        .iter()
        .map(|s| (s.id.to_string(), format!("{s:?}")))
        .collect();
    assert_eq!(
        got, want,
        "collector shortcuts diverge from SHORTCUT_DEFINITIONS"
    );
}

#[test]
fn collector_services_match_legacy() {
    let providers = default_providers();
    let collected = collect(&providers).expect("collector must produce a valid extension set");

    // ServiceDefinition serializes its metadata; runtime control fn pointers are
    // `#[serde(skip)]`, so this compares the descriptor fields.
    let got: BTreeMap<String, serde_json::Value> = collected
        .services()
        .iter()
        .map(|s| (s.id.to_string(), serde_json::to_value(s).unwrap()))
        .collect();
    let want: BTreeMap<String, serde_json::Value> = SERVICE_DEFINITIONS
        .iter()
        .map(|s| (s.id.to_string(), serde_json::to_value(s).unwrap()))
        .collect();
    assert_eq!(
        got, want,
        "collector services diverge from SERVICE_DEFINITIONS"
    );
}
