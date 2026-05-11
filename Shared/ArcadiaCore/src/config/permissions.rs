//! Static permission catalog and persisted global + per-subject grants.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io;

use super::modules::{
    ModulesConfig, MODULE_REGISTRY, TERMINAL_MODULE_NAME, TERMINAL_MOTD_MODULE_NAME,
};
use crate::config::{write_config_toml, ConfigFile};

const FILE_NAME: &str = "permissions.toml";
pub const SCHEMA_VERSION: u32 = 2;

/// Stable permission ids (see product plan / AGENTS).
#[derive(Debug, Clone, Copy)]
pub struct PermissionDefinition {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    /// Default for global toggle when key missing from disk.
    pub default_global: bool,
}

pub const PERMISSION_REGISTRY: &[PermissionDefinition] = &[
    PermissionDefinition {
        id: "network.lan",
        title: "LAN access",
        description: "Discovery, peer I/O, and LAN module commands (multicast, pairing, etc.).",
        default_global: false,
    },
    PermissionDefinition {
        id: "session.remote_route",
        title: "Remote routing",
        description: "Run commands on a peer host via --net:as lan:… (Arcadia remote control).",
        default_global: false,
    },
    PermissionDefinition {
        id: "shell.run",
        title: "Shell commands",
        description: "Spawn local subprocesses (shell.execute).",
        default_global: false,
    },
    PermissionDefinition {
        id: "shell.bridge",
        title: "Shell bridge",
        description: "Host/runtime bridge (shell.internal).",
        default_global: false,
    },
    PermissionDefinition {
        id: "surface.read",
        title: "Surface read",
        description: "Read UI mirror state (surface.snapshot, surface.revision).",
        default_global: true,
    },
    PermissionDefinition {
        id: "surface.control",
        title: "Surface control",
        description: "Mutate mirrored UI / module toggles (surface.patch).",
        default_global: true,
    },
    PermissionDefinition {
        id: "late.outbound",
        title: "Late.sh network",
        description: "Connect and interact with late.sh (WebSocket, credentials, chat).",
        default_global: false,
    },
    PermissionDefinition {
        id: "python.host",
        title: "Python host",
        description: "Load or reload extensions from disk (python-host.reload).",
        default_global: false,
    },
    PermissionDefinition {
        id: "python.extension_toggle",
        title: "Extension enable",
        description: "Enable or disable Python extensions.",
        default_global: false,
    },
    PermissionDefinition {
        id: "input.capture",
        title: "Input capture",
        description: "Reserved for future input capture / injection policy (global gate).",
        default_global: false,
    },
    PermissionDefinition {
        id: "tray.create",
        title: "Tray / menu-bar icons",
        description: "Create and update menu-bar (macOS) or system-tray (Windows/Linux) icons.",
        default_global: false,
    },
    PermissionDefinition {
        id: "cursor.global_position",
        title: "Global cursor position",
        description: "Read the OS-global mouse cursor position even when Arcadia is not focused.",
        default_global: false,
    },
    PermissionDefinition {
        id: "overlay.hud",
        title: "HUD overlay window",
        description: "Create and control the shared always-on-top transparent overlay window (non-interactive / pass-through in v1).",
        default_global: false,
    },
    PermissionDefinition {
        id: "overlay.system_ui",
        title: "Overlay system-UI tier",
        description: "Allow overlay.set-stacking system_ui (higher stacking tier; best-effort per OS).",
        default_global: false,
    },
];

pub fn permission_definition(id: &str) -> Option<&'static PermissionDefinition> {
    PERMISSION_REGISTRY.iter().find(|p| p.id == id)
}

pub fn is_known_permission_id(id: &str) -> bool {
    permission_definition(id).is_some()
}

/// Registry / modules.toml key for a native module (e.g. `terminal`, `lan`).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PermissionSubject {
    Module { name: String },
    Python { extension_id: String },
}

impl PermissionSubject {
    pub fn module(name: impl Into<String>) -> Self {
        Self::Module { name: name.into() }
    }

    pub fn python(extension_id: impl Into<String>) -> Self {
        Self::Python {
            extension_id: extension_id.into(),
        }
    }

    /// Serialize key for `permissions.toml` (`module:terminal`, `python:foo`).
    pub fn storage_key(&self) -> String {
        match self {
            PermissionSubject::Module { name } => format!("module:{name}"),
            PermissionSubject::Python { extension_id } => format!("python:{extension_id}"),
        }
    }

    pub fn parse_storage_key(key: &str) -> Option<Self> {
        if let Some(name) = key.strip_prefix("module:") {
            if name.is_empty() {
                return None;
            }
            return Some(Self::Module {
                name: name.to_string(),
            });
        }
        if let Some(id) = key.strip_prefix("python:") {
            if id.is_empty() {
                return None;
            }
            return Some(Self::Python {
                extension_id: id.to_string(),
            });
        }
        None
    }
}

/// Map dispatch token prefix (`shell`, `lan`, …) to modules.toml registry key for grants UI.
pub fn registry_module_for_command_prefix(prefix: &str) -> Option<&'static str> {
    match prefix {
        "shell" => Some(TERMINAL_MODULE_NAME),
        "shell-motd" => Some(TERMINAL_MOTD_MODULE_NAME),
        _ => ModulesConfig::manifest_for(prefix).map(|m| m.name),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionsConfig {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    /// Global kill-switches: permission id → allowed.
    #[serde(default)]
    pub globals: BTreeMap<String, bool>,
    /// Per subject (`module:…` / `python:…`): permission id → granted for that subject.
    #[serde(default)]
    pub subjects: BTreeMap<String, BTreeMap<String, bool>>,
}

fn default_schema_version() -> u32 {
    0
}

impl Default for PermissionsConfig {
    fn default() -> Self {
        let globals: BTreeMap<String, bool> = PERMISSION_REGISTRY
            .iter()
            .map(|p| (p.id.to_string(), p.default_global))
            .collect();
        let mut subjects = BTreeMap::new();
        let mut surface_grants = BTreeMap::new();
        surface_grants.insert("surface.read".to_string(), true);
        surface_grants.insert("surface.control".to_string(), true);
        subjects.insert(
            PermissionSubject::module(crate::config::modules::SURFACE_MODULE_NAME.to_string())
                .storage_key(),
            surface_grants,
        );
        Self {
            schema_version: SCHEMA_VERSION,
            globals,
            subjects,
        }
    }
}

impl PermissionsConfig {
    pub fn global_allowed(&self, permission_id: &str) -> bool {
        let Some(def) = permission_definition(permission_id) else {
            return false;
        };
        *self
            .globals
            .get(permission_id)
            .unwrap_or(&def.default_global)
    }

    pub fn set_global(&mut self, permission_id: &str, allowed: bool) -> Result<(), String> {
        if !is_known_permission_id(permission_id) {
            return Err(format!("Unknown permission: {permission_id}"));
        }
        self.globals.insert(permission_id.to_string(), allowed);
        Ok(())
    }

    /// Subject must explicitly grant (`true`); missing or `false` denies.
    pub fn subject_grants(&self, subject: &PermissionSubject, permission_id: &str) -> bool {
        if !is_known_permission_id(permission_id) {
            return false;
        }
        let key = subject.storage_key();
        self.subjects
            .get(&key)
            .and_then(|m| m.get(permission_id))
            .copied()
            .unwrap_or(false)
    }

    pub fn set_subject_grant(
        &mut self,
        subject: &PermissionSubject,
        permission_id: &str,
        granted: bool,
    ) -> Result<(), String> {
        if !is_known_permission_id(permission_id) {
            return Err(format!("Unknown permission: {permission_id}"));
        }
        let key = subject.storage_key();
        self.subjects
            .entry(key)
            .or_default()
            .insert(permission_id.to_string(), granted);
        Ok(())
    }

    /// Global on and subject explicitly granted.
    pub fn effective_allowed(&self, subject: &PermissionSubject, permission_id: &str) -> bool {
        self.global_allowed(permission_id) && self.subject_grants(subject, permission_id)
    }

    /// Any declared permission missing subject grant or globally off.
    pub fn missing_grants_for_module_enable(&self, registry_module_name: &str) -> Vec<String> {
        let Some(manifest) = ModulesConfig::manifest_for(registry_module_name) else {
            return Vec::new();
        };
        self.missing_grants_for_declared(
            &PermissionSubject::module(registry_module_name),
            manifest.required_permissions,
        )
    }

    pub fn missing_grants_for_declared(
        &self,
        subject: &PermissionSubject,
        required: &[&str],
    ) -> Vec<String> {
        required
            .iter()
            .filter(|pid| !self.effective_allowed(subject, pid))
            .map(|s| (*s).to_string())
            .collect()
    }

    pub fn grant_all_for_subject(&mut self, subject: &PermissionSubject, permission_ids: &[&str]) {
        for pid in permission_ids {
            if is_known_permission_id(pid) {
                let _ = self.set_subject_grant(subject, pid, true);
            }
        }
    }

    /// Turn on global + subject grant for each id (first-enable / modal Accept).
    pub fn ensure_effective_grants(
        &mut self,
        subject: &PermissionSubject,
        permission_ids: &[String],
    ) -> Result<(), String> {
        for pid in permission_ids {
            if !is_known_permission_id(pid) {
                continue;
            }
            self.set_global(pid, true)?;
            self.set_subject_grant(subject, pid, true)?;
        }
        Ok(())
    }

    /// Idempotent seed: for every currently-enabled native module, ensure its declared
    /// `required_permissions` are granted globally and to its own subject. Only fills keys
    /// that are absent from the map — values the user explicitly set (including to `false`)
    /// are left alone.
    ///
    /// The original migration (schema 0 → 1) only ran once and silently missed users who
    /// enabled a module via a non-modal path (CLI, file edit, or an older build). Schema
    /// 1 → 2 re-runs the seed so meta-commands like `python-host.extension-enable` work in
    /// those cases instead of being rejected at dispatch time.
    fn migrate_seed_from_modules_if_needed(&mut self) -> bool {
        if self.schema_version >= SCHEMA_VERSION {
            return false;
        }
        let Ok(modules) = ModulesConfig::load_or_create() else {
            self.schema_version = SCHEMA_VERSION;
            return true;
        };
        for manifest in MODULE_REGISTRY.iter() {
            if !modules.modules.get(manifest.name).copied().unwrap_or(false) {
                continue;
            }
            let subj = PermissionSubject::module(manifest.name);
            let subj_key = subj.storage_key();
            for pid in manifest.required_permissions {
                if !is_known_permission_id(pid) {
                    continue;
                }
                if !self.globals.contains_key(*pid) {
                    self.globals.insert((*pid).to_string(), true);
                }
                let entry = self.subjects.entry(subj_key.clone()).or_default();
                if !entry.contains_key(*pid) {
                    entry.insert((*pid).to_string(), true);
                }
            }
        }
        self.schema_version = SCHEMA_VERSION;
        true
    }
}

impl ConfigFile for PermissionsConfig {
    fn file_name() -> &'static str {
        FILE_NAME
    }

    fn merge_defaults(&mut self) -> bool {
        let mut changed = false;
        if self.migrate_seed_from_modules_if_needed() {
            changed = true;
        }

        for p in PERMISSION_REGISTRY {
            if !self.globals.contains_key(p.id) {
                self.globals.insert(p.id.to_string(), p.default_global);
                changed = true;
            }
        }

        let known: std::collections::HashSet<&str> =
            PERMISSION_REGISTRY.iter().map(|p| p.id).collect();
        self.globals.retain(|k, _| known.contains(k.as_str()));
        for m in self.subjects.values_mut() {
            m.retain(|k, _| known.contains(k.as_str()));
        }
        self.subjects.retain(|_, m| !m.is_empty());

        changed
    }

    fn save(&self) -> io::Result<()> {
        write_config_toml(Self::file_name(), self)
    }
}
