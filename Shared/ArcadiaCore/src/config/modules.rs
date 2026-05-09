use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::config::ConfigFile;

const LEGACY_LAN_MODULE_NAME: &str = "lan-module";
const LEGACY_TERMINAL_MODULE_NAME: &str = "shell";
const LEGACY_TERMINAL_MOTD_MODULE_NAME: &str = "shell-motd";
pub const LAN_MODULE_NAME: &str = "lan";
pub const LATE_MODULE_NAME: &str = "late";
pub const NET_MODULE_NAME: &str = "net";
pub const PYTHON_HOST_MODULE_NAME: &str = "python-host";
pub const SURFACE_MODULE_NAME: &str = "surface";
pub const REMOTE_SESSION_MODULE_NAME: &str = "remote-session";
pub const TERMINAL_MODULE_NAME: &str = "terminal";
pub const TERMINAL_MOTD_MODULE_NAME: &str = "terminal-motd";
const FILE_NAME: &str = "modules.toml";

#[derive(Debug, Clone, Copy)]
pub struct ModuleManifest {
    pub name: &'static str,
    pub version: &'static str,
    pub description: &'static str,
    pub required_modules: &'static [&'static str],
}

// Single source of truth for modules and their metadata.
static MODULE_REGISTRY: &[ModuleManifest] = &[
    ModuleManifest {
        name: LAN_MODULE_NAME,
        version: "1.0.0",
        description: "Local network discovery and peer communication.",
        required_modules: &[NET_MODULE_NAME],
    },
    ModuleManifest {
        name: NET_MODULE_NAME,
        version: "1.0.0",
        description: "Shared networking foundation for routed module commands.",
        required_modules: &[],
    },
    ModuleManifest {
        name: SURFACE_MODULE_NAME,
        version: "0.1.0",
        description: "Generic UI snapshot (surface.snapshot) and patches (surface.patch); extend patches for new surfaces.",
        required_modules: &[],
    },
    ModuleManifest {
        name: REMOTE_SESSION_MODULE_NAME,
        version: "0.1.0",
        description: "Permission to route execute_command over LAN (net_as: lan:…); transcript/mirror are automatic on hosts.",
        required_modules: &[NET_MODULE_NAME, LAN_MODULE_NAME],
    },
    ModuleManifest {
        name: TERMINAL_MODULE_NAME,
        version: "1.0.0",
        description: "Interactive terminal command execution for Arcadia surfaces.",
        required_modules: &[],
    },
    ModuleManifest {
        name: TERMINAL_MOTD_MODULE_NAME,
        version: "1.0.0",
        description: "Fastfetch-style banner when opening the Arcadia terminal (requires terminal).",
        required_modules: &[TERMINAL_MODULE_NAME],
    },
    ModuleManifest {
        name: LATE_MODULE_NAME,
        version: "0.1.0",
        description: "Native late.sh client — chat rooms, music stream, reactions, and bonsai.",
        required_modules: &[],
    },
    ModuleManifest {
        name: PYTHON_HOST_MODULE_NAME,
        version: "0.1.0",
        description: "Python extension loader. Scans ~/Arcadia/Extensions/ for .py files and registers their commands.",
        required_modules: &[],
    },
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulesConfig {
    pub modules: BTreeMap<String, bool>,
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
                let enabled = manifest.name == SURFACE_MODULE_NAME;
                (manifest.name.to_string(), enabled)
            })
            .collect();
        Self { modules }
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
    fn default_surface_enabled_others_disabled() {
        let cfg = base();
        assert_eq!(cfg.modules.get(SURFACE_MODULE_NAME), Some(&true));
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
        assert!(err.contains("net"), "error should mention missing dep: {err}");
    }

    #[test]
    fn set_module_state_blocks_disable_when_dependent_enabled() {
        let mut cfg = base();
        cfg.set_module_state(NET_MODULE_NAME, true).unwrap();
        cfg.set_module_state(LAN_MODULE_NAME, true).unwrap();
        let err = cfg.set_module_state(NET_MODULE_NAME, false).unwrap_err();
        assert!(err.contains("lan"), "error should mention blocking dependent: {err}");
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
        cfg.enable_with_requirements(REMOTE_SESSION_MODULE_NAME).unwrap();
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
        };
        let changed = cfg.merge_defaults();
        assert!(changed);
        assert!(!cfg.modules.contains_key(LEGACY_LAN_MODULE_NAME));
        assert_eq!(cfg.modules.get(LAN_MODULE_NAME), Some(&true));
    }

    #[test]
    fn merge_defaults_adds_missing_modules() {
        let mut cfg = ModulesConfig { modules: std::collections::BTreeMap::new() };
        let changed = cfg.merge_defaults();
        assert!(changed);
        for manifest in MODULE_REGISTRY {
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
    fn manifest_for_unknown_returns_none() {
        assert!(ModulesConfig::manifest_for("totally-fake").is_none());
    }

    #[test]
    fn set_module_state_unknown_module_errors() {
        let mut cfg = base();
        assert!(cfg.set_module_state("ghost-module", true).is_err());
    }

    #[test]
    fn terminal_motd_requires_terminal() {
        let mut cfg = base();
        let err = cfg.set_module_state(TERMINAL_MOTD_MODULE_NAME, true).unwrap_err();
        assert!(err.contains("terminal"), "error should mention terminal: {err}");
        cfg.set_module_state(TERMINAL_MODULE_NAME, true).unwrap();
        cfg.set_module_state(TERMINAL_MOTD_MODULE_NAME, true).unwrap();
        assert_eq!(cfg.modules.get(TERMINAL_MOTD_MODULE_NAME), Some(&true));
    }
}
