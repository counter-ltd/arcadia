use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io;

use crate::config::workspace::WorkspacePermissionDef;
use crate::config::{write_config_toml, ConfigFile};
use crate::platform::PlatformInfo;

/// OS id from [`crate::platform::PlatformInfo::name`]: `macos`, `windows`, `linux`, `ios`, `unknown`.
pub fn runtime_platform_id() -> &'static str {
    crate::platform::current().name()
}

/// `current` is typically [`runtime_platform_id`]. Empty `supported_platforms` = all platforms.
pub fn supports_platform(supported_platforms: &[&str], current: &str) -> bool {
    supported_platforms.is_empty() || supported_platforms.iter().any(|p| *p == current)
}

/// Empty slice = all platforms; otherwise host must match an entry.
pub fn supports_runtime_platform(supported_platforms: &[&str]) -> bool {
    supports_platform(supported_platforms, runtime_platform_id())
}

/// Same as [`supports_runtime_platform`] for owned strings (e.g. Python extension declarations).
pub fn supports_runtime_platform_owned(supported_platforms: &[String]) -> bool {
    if supported_platforms.is_empty() {
        return true;
    }
    let cur = runtime_platform_id();
    supported_platforms.iter().any(|p| p == cur)
}

const LEGACY_LAN_MODULE_NAME: &str = "lan-module";
const LEGACY_TERMINAL_MODULE_NAME: &str = "shell";
const LEGACY_TERMINAL_MOTD_MODULE_NAME: &str = "shell-motd";

/// Earlier ids for the same terminal-styling Python extension. They all collapse to
/// `terminal-theme` so `extension_state[<id>] = true` survives the rename instead of
/// silently resetting the extension to disabled.
/// Dead names — never reuse. Referenced by `config::extension_tokens` to migrate token files.
pub(crate) const LEGACY_TERMINAL_THEME_EXTENSION_IDS: &[&str] =
    &["tui-style", "shell-theme", "flux-theme"];
pub(crate) const TERMINAL_THEME_EXTENSION_ID: &str = "terminal-theme";
pub const ANIMATION_MODULE_NAME: &str = "animation";
pub const LAN_MODULE_NAME: &str = "lan";
pub const LATE_MODULE_NAME: &str = "late";
pub const NET_MODULE_NAME: &str = "net";
pub const PYTHON_HOST_MODULE_NAME: &str = "python-host";
pub const PERMISSIONS_MODULE_NAME: &str = "permissions";
pub const SURFACE_MODULE_NAME: &str = "surface";
pub const REMOTE_SESSION_MODULE_NAME: &str = "remote-session";
pub const TERMINAL_MODULE_NAME: &str = "terminal";
pub const TERMINAL_MOTD_MODULE_NAME: &str = "terminal-motd";
pub const TRAY_MODULE_NAME: &str = "tray";
pub const CURSOR_MODULE_NAME: &str = "cursor";
pub const KEYBOARD_MODULE_NAME: &str = "keyboard";
pub const AUDIO_MODULE_NAME: &str = "audio";
pub const OVERLAY_MODULE_NAME: &str = "overlay";
pub const WORKSPACE_MODULE_NAME: &str = "workspace";
pub const CODE_EDITOR_MODULE_NAME: &str = "code-editor";
pub const VISUAL_EDITOR_MODULE_NAME: &str = "visual-editor";
pub const AI_MODULE_NAME: &str = "ai";
pub const AI_LLAMA_CPP_MODULE_NAME: &str = "ai-provider-llama-cpp";
pub const AI_OLLAMA_MODULE_NAME: &str = "ai-provider-ollama";
pub const AI_OPENAI_MODULE_NAME: &str = "ai-provider-openai";
pub const AI_RULES_MODULE_NAME: &str = "ai-rules";
pub const AI_SKILLS_MODULE_NAME: &str = "ai-skills";
pub const AI_EXEC_CLAUDE_MODULE_NAME: &str = "ai-provider-exec-claude";
pub const AI_EXEC_CODEX_MODULE_NAME: &str = "ai-provider-exec-codex";
pub const AI_EXEC_GEMINI_MODULE_NAME: &str = "ai-provider-exec-gemini";
pub const AI_EXEC_AIDER_MODULE_NAME: &str = "ai-provider-exec-aider";
pub const AI_APFEL_MODULE_NAME: &str = "ai-provider-apfel";
pub const AI_IMAGE_PLAYGROUND_MODULE_NAME: &str = "ai-provider-image-playground";
pub const NOTIFICATION_MODULE_NAME: &str = "notification";
pub const GOTO_MODULE_NAME: &str = "goto";
pub const WASM_HOST_MODULE_NAME: &str = "wasm-host";
const FILE_NAME: &str = "modules.toml";

#[derive(Debug, Clone, Copy)]
pub struct ModuleManifest {
    pub name: &'static str,
    pub glyph: &'static str,
    pub version: &'static str,
    pub description: &'static str,
    /// Accent palette key (e.g. `"emerald"`, `"amber"`). Empty string = use the caller's default.
    pub accent: &'static str,
    pub required_modules: &'static [&'static str],
    /// Permissions that must be globally and per-subject granted for full use (first-enable flow).
    pub required_permissions: &'static [&'static str],
    /// Workspace-scoped permissions this module contributes. The workspace panel iterates all
    /// enabled modules and shows these as per-workspace grant toggles.
    pub workspace_permissions: &'static [WorkspacePermissionDef],
    /// Empty = all platforms. Otherwise whitelist of [`runtime_platform_id`] values.
    pub supported_platforms: &'static [&'static str],
}

// ─── Module registry ────────────────────────────────────────────────────────
//
// MODULE_REGISTRY is built from the extension collector: every module declares
// its own manifest via its `Extension` impl. The owned manifest strings are
// leaked into `&'static` so the registry keeps the original `ModuleManifest`
// type and every consumer is unchanged. The leak is a one-time process-lifetime
// cost — the registry never goes away.

use std::sync::LazyLock;

fn leak_str(s: String) -> &'static str {
    s.leak()
}

fn leak_strs(v: Vec<String>) -> &'static [&'static str] {
    let leaked: Vec<&'static str> = v.into_iter().map(leak_str).collect();
    leaked.leak()
}

fn leak_workspace_permissions(
    v: Vec<crate::extension::OwnedWorkspacePermissionDef>,
) -> &'static [WorkspacePermissionDef] {
    let leaked: Vec<WorkspacePermissionDef> = v
        .into_iter()
        .map(|w| WorkspacePermissionDef {
            id: leak_str(w.id),
            title: leak_str(w.title),
            description: leak_str(w.description),
            default_granted: w.default_granted,
        })
        .collect();
    leaked.leak()
}

fn leak_manifest(o: crate::extension::OwnedModuleManifest) -> ModuleManifest {
    ModuleManifest {
        name: leak_str(o.name),
        glyph: leak_str(o.glyph),
        version: leak_str(o.version),
        description: leak_str(o.description),
        accent: leak_str(o.accent),
        required_modules: leak_strs(o.required_modules),
        required_permissions: leak_strs(o.required_permissions),
        workspace_permissions: leak_workspace_permissions(o.workspace_permissions),
        supported_platforms: leak_strs(o.supported_platforms),
    }
}

/// Single source of truth for modules and their metadata — built once from
/// every self-registered extension. Replaces the former hand-written array.
pub static MODULE_REGISTRY: LazyLock<Vec<ModuleManifest>> = LazyLock::new(|| {
    let providers = crate::extension::provider::default_providers();
    let collected = crate::extension::collector::collect(&providers)
        .expect("extension collector must produce a valid module set");
    collected
        .module_manifests()
        .into_iter()
        .map(leak_manifest)
        .collect()
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulesConfig {
    pub modules: BTreeMap<String, bool>,
    /// Persistent enable state for Python extensions discovered under `~/Arcadia/Extensions/`.
    /// Keyed by extension id (directory name or file stem, kebab-cased). Defaults to `false`
    /// for newly-discovered extensions — the user must explicitly enable each one through the
    /// Extensions settings page so that side-effectful `main.py` bodies do not run (and OS
    /// permission prompts do not fire) until the user opts in.
    #[serde(default)]
    pub extension_state: BTreeMap<String, bool>,
    /// Persistent enable state for runtime modules discovered under `~/Arcadia/Modules/`.
    /// Parallels [`extension_state`](Self::extension_state) for the Rust-extension system.
    /// Keyed by module name. Defaults to `false` — like Python extensions, a runtime
    /// module's body does not run until the user explicitly opts in.
    #[serde(default)]
    pub module_state: BTreeMap<String, bool>,
}

fn required_modules(module_name: &str) -> &'static [&'static str] {
    MODULE_REGISTRY
        .iter()
        .find(|manifest| manifest.name == module_name)
        .map(|manifest| manifest.required_modules)
        .unwrap_or(&[])
}

fn is_known_module(module_name: &str) -> bool {
    MODULE_REGISTRY
        .iter()
        .any(|manifest| manifest.name == module_name)
}

impl Default for ModulesConfig {
    fn default() -> Self {
        let modules = MODULE_REGISTRY
            .iter()
            .map(|manifest| {
                let enabled = manifest.name == SURFACE_MODULE_NAME
                    || manifest.name == PERMISSIONS_MODULE_NAME
                    || manifest.name == ANIMATION_MODULE_NAME;
                (manifest.name.to_string(), enabled)
            })
            .collect();
        Self {
            modules,
            extension_state: BTreeMap::new(),
            module_state: BTreeMap::new(),
        }
    }
}

impl ModulesConfig {
    pub fn manifest_for(module_name: &str) -> Option<&'static ModuleManifest> {
        MODULE_REGISTRY
            .iter()
            .find(|manifest| manifest.name == module_name)
    }

    pub fn required_modules_for(module_name: &str) -> Result<&'static [&'static str], String> {
        if is_known_module(module_name) {
            Ok(required_modules(module_name))
        } else {
            Err("Unknown module key".to_string())
        }
    }

    pub fn missing_requirements_for(&self, module_name: &str) -> Result<Vec<String>, String> {
        if !self.modules.contains_key(module_name) {
            return Err("Unknown module key".to_string());
        }
        let mut missing = Vec::new();
        self.collect_missing_requirements(module_name, &mut missing)?;
        missing.sort();
        missing.dedup();
        Ok(missing)
    }

    fn collect_missing_requirements(
        &self,
        module_name: &str,
        missing: &mut Vec<String>,
    ) -> Result<(), String> {
        for required in Self::required_modules_for(module_name)? {
            let Some(required_enabled) = self.modules.get(*required) else {
                return Err(format!(
                    "Cannot enable {module_name}: required module {required} is missing"
                ));
            };
            if !required_enabled {
                missing.push((*required).to_string());
            }
            self.collect_missing_requirements(required, missing)?;
        }
        Ok(())
    }

    pub fn enable_with_requirements(&mut self, module_name: &str) -> Result<(), String> {
        if !self.modules.contains_key(module_name) {
            return Err("Unknown module key".to_string());
        }

        for required in required_modules(module_name) {
            if !self.modules.contains_key(*required) {
                return Err(format!(
                    "Cannot enable {module_name}: required module {required} is missing"
                ));
            }
            self.enable_with_requirements(required)?;
        }

        self.set_module_state(module_name, true)
    }

    /// Persistent enable state for a Python extension. Unknown ids resolve to `false` — new
    /// extensions discovered on disk start disabled until the user explicitly opts in.
    pub fn python_extension_enabled(&self, extension_id: &str) -> bool {
        self.extension_state
            .get(extension_id)
            .copied()
            .unwrap_or(false)
    }

    pub fn set_python_extension_enabled(&mut self, extension_id: &str, enabled: bool) {
        if extension_id.is_empty() {
            return;
        }
        self.extension_state
            .insert(extension_id.to_string(), enabled);
    }

    /// Persistent enable state for a runtime module loaded from `~/Arcadia/Modules/`.
    /// Unknown names resolve to `false` — newly-discovered modules start disabled
    /// until the user opts in (mirrors [`python_extension_enabled`](Self::python_extension_enabled)).
    pub fn runtime_module_enabled(&self, module_name: &str) -> bool {
        self.module_state
            .get(module_name)
            .copied()
            .unwrap_or(false)
    }

    pub fn set_runtime_module_enabled(&mut self, module_name: &str, enabled: bool) {
        if module_name.is_empty() {
            return;
        }
        self.module_state.insert(module_name.to_string(), enabled);
    }

    pub fn set_module_state(&mut self, module_name: &str, enabled: bool) -> Result<(), String> {
        if !self.modules.contains_key(module_name) {
            return Err("Unknown module key".to_string());
        }

        if enabled {
            for required in required_modules(module_name) {
                let Some(required_enabled) = self.modules.get(*required) else {
                    return Err(format!(
                        "Cannot enable {module_name}: required module {required} is missing"
                    ));
                };
                if !required_enabled {
                    return Err(format!(
                        "Cannot enable {module_name}: requires {required} to be enabled"
                    ));
                }
            }
            if let Some(manifest) = Self::manifest_for(module_name) {
                if !supports_runtime_platform(manifest.supported_platforms) {
                    return Err(format!(
                        "Cannot enable {module_name}: unsupported on this platform ({})",
                        runtime_platform_id()
                    ));
                }
            }
        } else {
            let blocking_dependents = self
                .modules
                .iter()
                .filter(|(_, is_enabled)| **is_enabled)
                .filter_map(|(name, _)| {
                    if required_modules(name).contains(&module_name) {
                        Some(name.as_str())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            if !blocking_dependents.is_empty() {
                return Err(format!(
                    "Cannot disable {module_name}: required by enabled module(s): {}",
                    blocking_dependents.join(", ")
                ));
            }
        }

        if let Some(entry) = self.modules.get_mut(module_name) {
            *entry = enabled;
        }
        Ok(())
    }
}

impl ConfigFile for ModulesConfig {
    fn file_name() -> &'static str {
        FILE_NAME
    }

    fn save(&self) -> io::Result<()> {
        write_config_toml(Self::file_name(), self)?;
        crate::surface_revision::bump_surface_revision();
        Ok(())
    }

    fn merge_defaults(&mut self) -> bool {
        let mut changed = false;

        if let Some(legacy_value) = self.modules.remove(LEGACY_LAN_MODULE_NAME) {
            self.modules
                .entry(LAN_MODULE_NAME.to_string())
                .or_insert(legacy_value);
            changed = true;
        }
        if self.modules.remove("lan-mobile").is_some() {
            changed = true;
        }

        if self.modules.remove("ai-provider-exec-cli").is_some() {
            changed = true;
        }

        if let Some(v) = self.modules.remove(LEGACY_TERMINAL_MODULE_NAME) {
            self.modules
                .entry(TERMINAL_MODULE_NAME.to_string())
                .or_insert(v);
            changed = true;
        }
        if let Some(v) = self.modules.remove(LEGACY_TERMINAL_MOTD_MODULE_NAME) {
            self.modules
                .entry(TERMINAL_MOTD_MODULE_NAME.to_string())
                .or_insert(v);
            changed = true;
        }

        let defaults = Self::default();
        for (key, value) in defaults.modules {
            if !self.modules.contains_key(&key) {
                self.modules.insert(key, value);
                changed = true;
            }
        }

        // Collapse legacy ids for the terminal-styling Python extension onto its new id.
        // `or_insert` preserves any explicit user choice already on the new key.
        for legacy in LEGACY_TERMINAL_THEME_EXTENSION_IDS {
            if let Some(val) = self.extension_state.remove(*legacy) {
                self.extension_state
                    .entry(TERMINAL_THEME_EXTENSION_ID.to_string())
                    .or_insert(val);
                changed = true;
            }
        }

        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> ModulesConfig {
        ModulesConfig::default()
    }

    #[test]
    fn default_surface_and_permissions_enabled_others_disabled() {
        let cfg = base();
        assert_eq!(cfg.modules.get(SURFACE_MODULE_NAME), Some(&true));
        assert_eq!(cfg.modules.get(PERMISSIONS_MODULE_NAME), Some(&true));
        assert_eq!(cfg.modules.get(TERMINAL_MODULE_NAME), Some(&false));
        assert_eq!(cfg.modules.get(NET_MODULE_NAME), Some(&false));
        assert_eq!(cfg.modules.get(LAN_MODULE_NAME), Some(&false));
    }

    #[test]
    fn set_module_state_enables_known_module() {
        let mut cfg = base();
        cfg.set_module_state(TERMINAL_MODULE_NAME, true).unwrap();
        assert_eq!(cfg.modules.get(TERMINAL_MODULE_NAME), Some(&true));
    }

    #[test]
    fn set_module_state_blocks_enable_when_dep_missing() {
        let mut cfg = base();
        // lan requires net; net is disabled
        let err = cfg.set_module_state(LAN_MODULE_NAME, true).unwrap_err();
        assert!(
            err.contains("net"),
            "error should mention missing dep: {err}"
        );
    }

    #[test]
    fn set_module_state_blocks_disable_when_dependent_enabled() {
        let mut cfg = base();
        cfg.set_module_state(NET_MODULE_NAME, true).unwrap();
        cfg.set_module_state(LAN_MODULE_NAME, true).unwrap();
        let err = cfg.set_module_state(NET_MODULE_NAME, false).unwrap_err();
        assert!(
            err.contains("lan"),
            "error should mention blocking dependent: {err}"
        );
    }

    #[test]
    fn enable_with_requirements_transitively_enables_deps() {
        let mut cfg = base();
        cfg.enable_with_requirements(LAN_MODULE_NAME).unwrap();
        assert_eq!(cfg.modules.get(NET_MODULE_NAME), Some(&true));
        assert_eq!(cfg.modules.get(LAN_MODULE_NAME), Some(&true));
    }

    #[test]
    fn enable_with_requirements_remote_session_enables_net_and_lan() {
        let mut cfg = base();
        cfg.enable_with_requirements(REMOTE_SESSION_MODULE_NAME)
            .unwrap();
        assert_eq!(cfg.modules.get(NET_MODULE_NAME), Some(&true));
        assert_eq!(cfg.modules.get(LAN_MODULE_NAME), Some(&true));
        assert_eq!(cfg.modules.get(REMOTE_SESSION_MODULE_NAME), Some(&true));
    }

    #[test]
    fn missing_requirements_for_lan_without_net() {
        let cfg = base();
        let missing = cfg.missing_requirements_for(LAN_MODULE_NAME).unwrap();
        assert!(missing.contains(&NET_MODULE_NAME.to_string()));
    }

    #[test]
    fn missing_requirements_empty_when_dep_met() {
        let mut cfg = base();
        cfg.set_module_state(NET_MODULE_NAME, true).unwrap();
        let missing = cfg.missing_requirements_for(LAN_MODULE_NAME).unwrap();
        assert!(missing.is_empty());
    }

    #[test]
    fn missing_requirements_unknown_module_errors() {
        let cfg = base();
        assert!(cfg.missing_requirements_for("does-not-exist").is_err());
    }

    #[test]
    fn merge_defaults_migrates_legacy_lan_key() {
        let mut cfg = ModulesConfig {
            modules: {
                let mut m = std::collections::BTreeMap::new();
                m.insert(LEGACY_LAN_MODULE_NAME.to_string(), true);
                m
            },
            extension_state: std::collections::BTreeMap::new(),
            module_state: std::collections::BTreeMap::new(),
        };
        let changed = cfg.merge_defaults();
        assert!(changed);
        assert!(!cfg.modules.contains_key(LEGACY_LAN_MODULE_NAME));
        assert_eq!(cfg.modules.get(LAN_MODULE_NAME), Some(&true));
    }

    #[test]
    fn merge_defaults_adds_missing_modules() {
        let mut cfg = ModulesConfig {
            modules: std::collections::BTreeMap::new(),
            extension_state: std::collections::BTreeMap::new(),
            module_state: std::collections::BTreeMap::new(),
        };
        let changed = cfg.merge_defaults();
        assert!(changed);
        for manifest in MODULE_REGISTRY.iter() {
            assert!(
                cfg.modules.contains_key(manifest.name),
                "merge_defaults must add missing module '{}'",
                manifest.name
            );
        }
    }

    #[test]
    fn manifest_for_known_module() {
        let m = ModulesConfig::manifest_for(TERMINAL_MODULE_NAME).unwrap();
        assert_eq!(m.name, TERMINAL_MODULE_NAME);
    }

    #[test]
    fn all_manifest_required_permissions_exist_in_catalog() {
        use crate::config::permissions::is_known_permission_id;
        for m in MODULE_REGISTRY.iter() {
            for pid in m.required_permissions {
                assert!(
                    is_known_permission_id(pid),
                    "module '{}' lists unknown permission '{}'",
                    m.name,
                    pid
                );
            }
        }
    }

    #[test]
    fn manifest_for_unknown_returns_none() {
        assert!(ModulesConfig::manifest_for("totally-fake").is_none());
    }

    #[test]
    fn supports_platform_empty_means_all() {
        assert!(supports_platform(&[], "ios"));
        assert!(supports_runtime_platform(&[]));
    }

    #[test]
    fn supports_platform_whitelist() {
        assert!(supports_platform(&["macos", "linux"], "macos"));
        assert!(!supports_platform(&["macos", "linux"], "ios"));
    }

    #[test]
    fn supports_runtime_platform_owned_nonempty_matches_host() {
        assert!(supports_runtime_platform_owned(&[]));
        let cur = runtime_platform_id().to_string();
        assert!(supports_runtime_platform_owned(&[cur]));
        assert!(!supports_runtime_platform_owned(&[
            "__no_such_os__".to_string()
        ]));
    }

    #[test]
    fn set_module_state_unknown_module_errors() {
        let mut cfg = base();
        assert!(cfg.set_module_state("ghost-module", true).is_err());
    }

    #[test]
    fn python_extension_enabled_defaults_to_false_for_unknown_id() {
        let cfg = base();
        assert!(!cfg.python_extension_enabled("googly-eyes"));
        assert!(!cfg.python_extension_enabled(""));
    }

    #[test]
    fn set_python_extension_enabled_round_trips() {
        let mut cfg = base();
        cfg.set_python_extension_enabled("googly-eyes", true);
        assert!(cfg.python_extension_enabled("googly-eyes"));
        cfg.set_python_extension_enabled("googly-eyes", false);
        assert!(!cfg.python_extension_enabled("googly-eyes"));
    }

    #[test]
    fn set_python_extension_enabled_ignores_empty_id() {
        let mut cfg = base();
        cfg.set_python_extension_enabled("", true);
        assert!(cfg.extension_state.is_empty());
    }

    #[test]
    fn terminal_motd_requires_terminal() {
        let mut cfg = base();
        let err = cfg
            .set_module_state(TERMINAL_MOTD_MODULE_NAME, true)
            .unwrap_err();
        assert!(
            err.contains("terminal"),
            "error should mention terminal: {err}"
        );
        cfg.set_module_state(TERMINAL_MODULE_NAME, true).unwrap();
        cfg.set_module_state(TERMINAL_MOTD_MODULE_NAME, true)
            .unwrap();
        assert_eq!(cfg.modules.get(TERMINAL_MOTD_MODULE_NAME), Some(&true));
    }
}
