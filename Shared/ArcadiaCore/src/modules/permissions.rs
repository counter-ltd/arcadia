//! Headless + GUI permission management (`permissions.toml`).

use crate::config::permissions::{
    is_known_permission_id, PermissionSubject, PermissionsConfig, PERMISSION_REGISTRY,
};
use crate::config::ConfigFile;
use crate::modules::{ExecutionContext, ModuleCommand};

pub const NAME: &str = "permissions";

fn parse_bool(s: &str) -> Option<bool> {
    match s.to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn list_cmd(_args: &[&str], _ctx: &ExecutionContext) -> String {
    let Ok(cfg) = PermissionsConfig::load_or_create() else {
        return "Error: could not load permissions.toml".to_string();
    };
    let mut lines = Vec::new();
    lines.push("permissions (catalog):".to_string());
    for p in PERMISSION_REGISTRY.iter() {
        lines.push(format!(
            "  {} — {} [default_global={}]",
            p.id, p.title, p.default_global
        ));
    }
    lines.push(String::new());
    lines.push("globals (effective):".to_string());
    for p in PERMISSION_REGISTRY.iter() {
        let on = cfg.global_allowed(p.id);
        lines.push(format!("  {} = {on}", p.id));
    }
    if !cfg.subjects.is_empty() {
        lines.push(String::new());
        lines.push("subject grants:".to_string());
        for (subj, m) in &cfg.subjects {
            for (pid, v) in m {
                lines.push(format!("  {subj} {pid} = {v}"));
            }
        }
    }
    lines.join("\n")
}

fn global_set(args: &[&str], _ctx: &ExecutionContext) -> String {
    let Some(perm) = args.first().copied() else {
        return "Usage: permissions.global-set <permission-id> <true|false>".to_string();
    };
    let Some(b) = args.get(1).copied().and_then(parse_bool) else {
        return "Usage: permissions.global-set <permission-id> <true|false>".to_string();
    };
    let Ok(mut cfg) = PermissionsConfig::load_or_create() else {
        return "Error: could not load permissions.toml".to_string();
    };
    match cfg.set_global(perm, b) {
        Ok(()) => {}
        Err(e) => return e,
    }
    if let Err(e) = cfg.save() {
        return format!("Save failed: {e}");
    }
    format!("global {perm} = {b}")
}

fn grant_set(args: &[&str], _ctx: &ExecutionContext) -> String {
    let Some(subj_raw) = args.first().copied() else {
        return "Usage: permissions.grant-set <module:name|python:id> <permission-id> <true|false>"
            .to_string();
    };
    let Some(perm) = args.get(1).copied() else {
        return "Usage: permissions.grant-set <subject> <permission-id> <true|false>".to_string();
    };
    let Some(b) = args.get(2).copied().and_then(parse_bool) else {
        return "Usage: permissions.grant-set <subject> <permission-id> <true|false>".to_string();
    };
    let Some(subject) = PermissionSubject::parse_storage_key(subj_raw) else {
        return format!(
            "Invalid subject '{subj_raw}'. Use module:<registry-name> or python:<extension-id>"
        );
    };
    let Ok(mut cfg) = PermissionsConfig::load_or_create() else {
        return "Error: could not load permissions.toml".to_string();
    };
    match cfg.set_subject_grant(&subject, perm, b) {
        Ok(()) => {}
        Err(e) => return e,
    }
    if let Err(e) = cfg.save() {
        return format!("Save failed: {e}");
    }
    format!("grant {} {} = {b}", subject.storage_key(), perm)
}

pub fn commands() -> &'static [ModuleCommand] {
    &[
        ModuleCommand {
            name: "list",
            description: "List permission catalog, global flags, and subject grants.",
            required_permissions: &[],
            run: list_cmd,
        },
        ModuleCommand {
            name: "global-set",
            description: "Set global permission: permissions.global-set <id> <true|false>",
            required_permissions: &[],
            run: global_set,
        },
        ModuleCommand {
            name: "grant-set",
            description:
                "Set per-subject grant: permissions.grant-set module:terminal shell.run true",
            required_permissions: &[],
            run: grant_set,
        },
    ]
}

/// Exposed for CLI `permit` / Desktop without duplicating parse logic.
pub fn apply_global_set(permission_id: &str, allowed: bool) -> Result<String, String> {
    let mut cfg = PermissionsConfig::load_or_create().map_err(|e| e.to_string())?;
    cfg.set_global(permission_id, allowed)?;
    cfg.save().map_err(|e| e.to_string())?;
    Ok(format!("global {permission_id} = {allowed}"))
}

pub fn apply_grant_set(
    subject: &PermissionSubject,
    permission_id: &str,
    granted: bool,
) -> Result<String, String> {
    let mut cfg = PermissionsConfig::load_or_create().map_err(|e| e.to_string())?;
    cfg.set_subject_grant(subject, permission_id, granted)?;
    cfg.save().map_err(|e| e.to_string())?;
    Ok(format!(
        "grant {} {} = {granted}",
        subject.storage_key(),
        permission_id
    ))
}

pub fn normalize_cli_subject(raw: &str) -> Result<PermissionSubject, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err("Subject cannot be empty".to_string());
    }
    if let Some(s) = PermissionSubject::parse_storage_key(raw) {
        return Ok(s);
    }
    // Bare name: try as registry module, else python extension id
    if crate::config::modules::ModulesConfig::manifest_for(raw).is_some() {
        return Ok(PermissionSubject::module(raw.to_string()));
    }
    Ok(PermissionSubject::python(raw.to_string()))
}

pub fn list_unknown_permission_ids(ids: &[&str]) -> Vec<String> {
    ids.iter()
        .copied()
        .filter(|id| !is_known_permission_id(id))
        .map(str::to_string)
        .collect()
}

#[derive(Default)]
pub struct PermissionsExtension;

impl crate::extension::Extension for PermissionsExtension {
    fn manifest(&self) -> crate::extension::OwnedModuleManifest {
        crate::extension::OwnedModuleManifest {
            name: NAME.to_string(),
            glyph: "permissions".to_string(),
            version: "0.1.0".to_string(),
            description: "Permission catalog, grants, and headless permit/list commands."
                .to_string(),
            accent: String::new(),
            required_modules: Vec::new(),
            required_permissions: Vec::new(),
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

crate::register_extension!(PermissionsExtension);
