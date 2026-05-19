//! Generic host UI snapshot + batched patches (extend [`SurfacePatch`] for editors, settings, etc.).
//!
//! ## `SurfaceSnapshot.extra` schema (version **1**)
//!
//! Host **`extra`** is a JSON object with:
//! - **`schema_version`**: matches [`SURFACE_EXTRA_SCHEMA_VERSION`].
//! - **`navigation_registry`**: serialized [`NavigationRegistryOwned`].
//!
//! Extend with additional keys as needed; bump **`schema_version`** when semantics change.
//! Prefer new [`SurfacePatch`] variants over ad-hoc module verbs for mirrored UI state.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::config::modules::ModulesConfig;
use crate::config::ConfigFile;
use crate::modules::{ExecutionContext, ModuleCommand};
use crate::navigation::NavigationRegistryOwned;
use crate::surface_revision::current_surface_revision;

pub const NAME: &str = "surface";

/// Bump when `extra` layout or required fields change (see module docs).
pub const SURFACE_EXTRA_SCHEMA_VERSION: u32 = 1;

pub use crate::surface_revision::bump_surface_revision;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceSnapshot {
    pub modules: BTreeMap<String, bool>,
    #[serde(default)]
    pub revision: u64,
    #[serde(default)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum SurfacePatch {
    ModulesSet {
        #[serde(default)]
        client_id: Option<String>,
        name: String,
        enabled: bool,
    },
}

#[derive(Debug, Clone, Default)]
pub struct ParsedSurfaceSnapshot {
    pub modules: Vec<(String, bool)>,
    pub revision: u64,
    pub navigation_registry: Option<NavigationRegistryOwned>,
}

/// Lightweight parse for `surface.revision` JSON (`{"revision":…}`).
pub fn parse_surface_revision(payload: &str) -> Option<u64> {
    let v: serde_json::Value = serde_json::from_str(payload).ok()?;
    v.get("revision")?.as_u64()
}

pub fn parse_surface_snapshot(payload: &str) -> ParsedSurfaceSnapshot {
    serde_json::from_str::<SurfaceSnapshot>(payload)
        .map(|s| ParsedSurfaceSnapshot {
            modules: s.modules.into_iter().collect(),
            revision: s.revision,
            navigation_registry: navigation_registry_from_extra(&s.extra),
        })
        .unwrap_or_default()
}

fn navigation_registry_from_extra(extra: &serde_json::Value) -> Option<NavigationRegistryOwned> {
    extra
        .get("navigation_registry")
        .and_then(|v| serde_json::from_value::<NavigationRegistryOwned>(v.clone()).ok())
}

fn revision_json(_args: &[&str], _ctx: &ExecutionContext) -> String {
    serde_json::json!({
        "revision": current_surface_revision(),
    })
    .to_string()
}

fn snapshot(_args: &[&str], _ctx: &ExecutionContext) -> String {
    let Ok(cfg) = ModulesConfig::load_or_create() else {
        return "{}".to_string();
    };
    let revision = current_surface_revision();
    let navigation_registry =
        serde_json::to_value(&NavigationRegistryOwned::with_extension_token_settings_merged())
            .unwrap_or_else(|_| serde_json::json!({}));
    let snap = SurfaceSnapshot {
        modules: cfg.modules.clone(),
        revision,
        extra: serde_json::json!({
            "schema_version": SURFACE_EXTRA_SCHEMA_VERSION,
            "navigation_registry": navigation_registry,
        }),
    };
    serde_json::to_string(&snap).unwrap_or_else(|_| "{}".to_string())
}

fn patch(args: &[&str], _ctx: &ExecutionContext) -> String {
    let Some(payload) = args.first().copied() else {
        return "Usage: surface.patch '<json-array-of-patch-objects>'".to_string();
    };
    let patches: Vec<SurfacePatch> = match serde_json::from_str(payload) {
        Ok(p) => p,
        Err(e) => return format!("Invalid surface.patch JSON: {e}"),
    };
    let mut messages = Vec::new();
    for p in patches {
        match p {
            SurfacePatch::ModulesSet { name, enabled, .. } => {
                let mut cfg = match ModulesConfig::load_or_create() {
                    Ok(c) => c,
                    Err(e) => return format!("Error loading config: {e}"),
                };
                if let Err(e) = cfg.set_module_state(&name, enabled) {
                    return e;
                }
                if let Err(e) = cfg.save() {
                    return format!("Error saving config: {e}");
                }
                messages.push(format!(
                    "Module {name} {}",
                    if enabled { "enabled" } else { "disabled" }
                ));
            }
        }
    }
    if messages.is_empty() {
        return "No patches applied".to_string();
    }
    messages.join("\n")
}

/// Helpers for surfaces that don't depend on `serde_json` directly.
pub fn snapshot_module_rows(payload: &str) -> Vec<(String, bool)> {
    parse_surface_snapshot(payload).modules
}

pub fn patch_json_modules_set(name: &str, enabled: bool, client_id: Option<&str>) -> String {
    #[derive(Serialize)]
    struct Row<'a> {
        op: &'static str,
        name: &'a str,
        enabled: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        client_id: Option<&'a str>,
    }
    serde_json::to_string(&vec![Row {
        op: "modules_set",
        name,
        enabled,
        client_id,
    }])
    .unwrap_or_else(|_| "[]".to_string())
}

pub fn commands() -> &'static [ModuleCommand] {
    &[
        ModuleCommand {
            name: "snapshot",
            description:
                "JSON SurfaceSnapshot (modules, revision, extra.navigation_registry); revision advances when modules.toml saves",
            required_permissions: &["surface.read"],
            run: snapshot,
        },
        ModuleCommand {
            name: "revision",
            description: r#"JSON {"revision":u64} — cheap host generation counter for thin-client polls"#,
            required_permissions: &["surface.read"],
            run: revision_json,
        },
        ModuleCommand {
            name: "patch",
            description:
                "Apply SurfacePatch JSON array (modules_set + optional client_id); shared host state for multi-client",
            required_permissions: &["surface.control"],
            run: patch,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::navigation::NavigationRegistryOwned;

    #[test]
    fn parse_surface_revision_reads_counter() {
        let json = r#"{"revision":99}"#;
        assert_eq!(parse_surface_revision(json), Some(99));
        assert_eq!(parse_surface_revision("{}"), None);
    }

    #[test]
    fn parse_snapshot_valid_json() {
        let json = r#"{"modules":{"shell":true,"net":false},"revision":5,"extra":{}}"#;
        let parsed = parse_surface_snapshot(json);
        assert_eq!(parsed.revision, 5);
        assert!(parsed.modules.iter().any(|(n, e)| n == "shell" && *e));
        assert!(parsed.modules.iter().any(|(n, e)| n == "net" && !e));
    }

    #[test]
    fn parse_snapshot_returns_default_on_invalid() {
        let parsed = parse_surface_snapshot("not json");
        assert_eq!(parsed.revision, 0);
        assert!(parsed.modules.is_empty());
        assert!(parsed.navigation_registry.is_none());
    }

    #[test]
    fn parse_snapshot_extracts_navigation_registry() {
        let registry = NavigationRegistryOwned::from_static_registry();
        let registry_val = serde_json::to_value(&registry).unwrap();
        let snap = SurfaceSnapshot {
            modules: Default::default(),
            revision: 1,
            extra: serde_json::json!({ "schema_version": 1, "navigation_registry": registry_val }),
        };
        let json = serde_json::to_string(&snap).unwrap();
        let parsed = parse_surface_snapshot(&json);
        let nav = parsed
            .navigation_registry
            .expect("navigation_registry must deserialize");
        assert!(!nav.pages.is_empty());
        assert!(!nav.groups.is_empty());
    }

    #[test]
    fn patch_json_includes_all_fields() {
        let json = patch_json_modules_set("shell", true, Some("client-abc"));
        assert!(json.contains(r#""op":"modules_set""#));
        assert!(json.contains(r#""name":"shell""#));
        assert!(json.contains(r#""enabled":true"#));
        assert!(json.contains(r#""client_id":"client-abc""#));
    }

    #[test]
    fn patch_json_omits_client_id_when_none() {
        let json = patch_json_modules_set("net", false, None);
        assert!(!json.contains("client_id"));
    }

    #[test]
    fn parse_snapshot_round_trips_navigation_registry_like_snapshot_extra() {
        let registry = NavigationRegistryOwned::from_static_registry();
        let registry_val = serde_json::to_value(&registry).unwrap();
        let snap = SurfaceSnapshot {
            modules: BTreeMap::from([(
                crate::config::modules::SURFACE_MODULE_NAME.to_string(),
                true,
            )]),
            revision: 42,
            extra: serde_json::json!({
                "schema_version": 1,
                "navigation_registry": registry_val,
            }),
        };
        let json = serde_json::to_string(&snap).unwrap();
        let parsed = parse_surface_snapshot(&json);
        assert_eq!(parsed.revision, 42);
        let nav = parsed.navigation_registry.expect("navigation_registry");
        assert_eq!(nav.pages.len(), registry.pages.len());
        assert_eq!(nav.groups.len(), registry.groups.len());
    }

    #[test]
    fn surface_patch_modules_set_round_trips() {
        let json = patch_json_modules_set("shell", true, Some("abc"));
        let patches: Vec<SurfacePatch> = serde_json::from_str(&json).unwrap();
        assert_eq!(patches.len(), 1);
        match &patches[0] {
            SurfacePatch::ModulesSet {
                name,
                enabled,
                client_id,
            } => {
                assert_eq!(name, "shell");
                assert!(*enabled);
                assert_eq!(client_id.as_deref(), Some("abc"));
            }
        }
    }

    #[test]
    fn snapshot_module_rows_helper() {
        let json = r#"{"modules":{"shell":true},"revision":1,"extra":{}}"#;
        let rows = snapshot_module_rows(json);
        assert!(rows.iter().any(|(n, e)| n == "shell" && *e));
    }
}

#[derive(Default)]
pub struct SurfaceExtension;

impl crate::extension::Extension for SurfaceExtension {
    fn manifest(&self) -> crate::extension::OwnedModuleManifest {
        crate::extension::OwnedModuleManifest {
            name: NAME.to_string(),
            glyph: "surface".to_string(),
            version: "0.1.0".to_string(),
            description:
                "Generic UI snapshot (surface.snapshot) and patches (surface.patch); extend patches for new surfaces."
                    .to_string(),
            accent: String::new(),
            required_modules: Vec::new(),
            required_permissions: vec![
                "surface.read".to_string(),
                "surface.control".to_string(),
            ],
            workspace_permissions: Vec::new(),
            supported_platforms: Vec::new(),
            api_exports: Vec::new(),
        }
    }

    fn commands(&self) -> Vec<crate::extension::OwnedModuleCommand> {
        commands()
            .iter()
            .map(crate::extension::OwnedModuleCommand::from_static)
            .collect()
    }
}

crate::register_extension!(SurfaceExtension);
