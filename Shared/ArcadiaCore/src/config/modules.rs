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
/// `terminal-theme` so `python_extensions[<id>] = true` survives the rename instead of
/// silently resetting the extension to disabled.
const LEGACY_TERMINAL_THEME_EXTENSION_IDS: &[&str] = &["tui-style", "shell-theme", "flux-theme"];
const TERMINAL_THEME_EXTENSION_ID: &str = "terminal-theme";
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
pub const OVERLAY_MODULE_NAME: &str = "overlay";
pub const WORKSPACE_MODULE_NAME: &str = "workspace";
pub const CODE_EDITOR_MODULE_NAME: &str = "code-editor";
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
const FILE_NAME: &str = "modules.toml";

#[derive(Debug, Clone, Copy)]
pub struct ModuleManifest {
    pub name: &'static str,
    pub version: &'static str,
    pub description: &'static str,
    pub required_modules: &'static [&'static str],
    /// Permissions that must be globally and per-subject granted for full use (first-enable flow).
    pub required_permissions: &'static [&'static str],
    /// Workspace-scoped permissions this module contributes. The workspace panel iterates all
    /// enabled modules and shows these as per-workspace grant toggles.
    pub workspace_permissions: &'static [WorkspacePermissionDef],
    /// Empty = all platforms. Otherwise whitelist of [`runtime_platform_id`] values.
    pub supported_platforms: &'static [&'static str],
}

// Single source of truth for modules and their metadata.
pub static MODULE_REGISTRY: &[ModuleManifest] = &[
    ModuleManifest {
        name: ANIMATION_MODULE_NAME,
        version: "0.1.0",
        description: "Shared tween engine for modules and extensions. One 16 ms driver loop services all running animations.",
        required_modules: &[],
        required_permissions: &[],
        workspace_permissions: &[],
        supported_platforms: &[],
    },
    ModuleManifest {
        name: LAN_MODULE_NAME,
        version: "1.0.0",
        description: "Local network discovery and peer communication.",
        required_modules: &[NET_MODULE_NAME],
        required_permissions: &["network.lan"],
        workspace_permissions: &[],
        supported_platforms: &[],
    },
    ModuleManifest {
        name: NET_MODULE_NAME,
        version: "1.0.0",
        description: "Shared networking foundation for routed module commands.",
        required_modules: &[],
        required_permissions: &[],
        workspace_permissions: &[],
        supported_platforms: &[],
    },
    ModuleManifest {
        name: SURFACE_MODULE_NAME,
        version: "0.1.0",
        description: "Generic UI snapshot (surface.snapshot) and patches (surface.patch); extend patches for new surfaces.",
        required_modules: &[],
        required_permissions: &["surface.read", "surface.control"],
        workspace_permissions: &[],
        supported_platforms: &[],
    },
    ModuleManifest {
        name: REMOTE_SESSION_MODULE_NAME,
        version: "0.1.0",
        description: "Permission to route execute_command over LAN (net_as: lan:…); transcript/mirror are automatic on hosts.",
        required_modules: &[NET_MODULE_NAME, LAN_MODULE_NAME],
        required_permissions: &["session.remote_route"],
        workspace_permissions: &[],
        supported_platforms: &[],
    },
    ModuleManifest {
        name: TERMINAL_MODULE_NAME,
        version: "1.0.0",
        description: "Interactive terminal command execution for Arcadia surfaces.",
        required_modules: &[],
        required_permissions: &["shell.run", "shell.bridge"],
        workspace_permissions: &[],
        supported_platforms: &[],
    },
    ModuleManifest {
        name: TERMINAL_MOTD_MODULE_NAME,
        version: "1.0.0",
        description: "Fastfetch-style banner when opening the Arcadia terminal (requires terminal).",
        required_modules: &[TERMINAL_MODULE_NAME],
        required_permissions: &[],
        workspace_permissions: &[],
        supported_platforms: &[],
    },
    ModuleManifest {
        name: LATE_MODULE_NAME,
        version: "0.1.0",
        description: "Native late.sh client — chat rooms, music stream, reactions, and bonsai.",
        required_modules: &[],
        required_permissions: &["late.outbound"],
        workspace_permissions: &[],
        supported_platforms: &[],
    },
    ModuleManifest {
        name: PYTHON_HOST_MODULE_NAME,
        version: "0.1.0",
        description: "Python extension loader. Scans ~/Arcadia/Extensions/ for .py files and registers their commands.",
        required_modules: &[],
        required_permissions: &["python.host", "python.extension_toggle"],
        workspace_permissions: &[],
        supported_platforms: &[],
    },
    ModuleManifest {
        name: PERMISSIONS_MODULE_NAME,
        version: "0.1.0",
        description: "Permission catalog, grants, and headless permit/list commands.",
        required_modules: &[],
        required_permissions: &[],
        workspace_permissions: &[],
        supported_platforms: &[],
    },
    ModuleManifest {
        name: TRAY_MODULE_NAME,
        version: "0.1.0",
        description: "Menu-bar (macOS) and system-tray (Windows/Linux) icons with dynamic images and menus.",
        required_modules: &[],
        required_permissions: &["tray.create"],
        workspace_permissions: &[],
        supported_platforms: &["macos", "windows", "linux"],
    },
    ModuleManifest {
        name: CURSOR_MODULE_NAME,
        version: "0.1.0",
        description: "OS-global cursor position and primary display size for extensions that track input.",
        required_modules: &[],
        required_permissions: &["cursor.global_position"],
        workspace_permissions: &[],
        supported_platforms: &["macos", "windows", "linux"],
    },
    ModuleManifest {
        name: OVERLAY_MODULE_NAME,
        version: "0.1.0",
        description: "Single always-on-top transparent HUD window for overlays (pointer pass-through v1).",
        required_modules: &[],
        required_permissions: &["overlay.hud"],
        workspace_permissions: &[],
        supported_platforms: &["macos", "windows", "linux"],
    },
    ModuleManifest {
        name: WORKSPACE_MODULE_NAME,
        version: "0.1.0",
        description: "Workspace directory registry with scoped file and execution permissions.",
        required_modules: &[],
        required_permissions: &[],
        workspace_permissions: &[
            WorkspacePermissionDef {
                id: "workspace.read",
                title: "File read",
                description: "Read files within this workspace.",
                default_granted: true,
            },
            WorkspacePermissionDef {
                id: "workspace.write",
                title: "File write",
                description: "Create, modify, and delete files.",
                default_granted: false,
            },
            WorkspacePermissionDef {
                id: "workspace.execute",
                title: "Command execution",
                description: "Run commands scoped to this workspace.",
                default_granted: false,
            },
        ],
        supported_platforms: &[],
    },
    ModuleManifest {
        name: CODE_EDITOR_MODULE_NAME,
        version: "0.1.0",
        description: "Code editor with per-file tabs in the sidebar.",
        required_modules: &[],
        required_permissions: &[],
        workspace_permissions: &[],
        supported_platforms: &[],
    },
    ModuleManifest {
        name: AI_MODULE_NAME,
        version: "0.1.0",
        description: "AI chat interface. Requires an AI provider module to be enabled.",
        required_modules: &[],
        required_permissions: &[],
        workspace_permissions: &[
            WorkspacePermissionDef {
                id: "workspace.read",
                title: "File read (AI)",
                description: "Allow the AI to read files via @mention and read_file tool.",
                default_granted: false,
            },
            WorkspacePermissionDef {
                id: "workspace.write",
                title: "File write (AI)",
                description: "Allow the AI to create and modify files via write_file tool.",
                default_granted: false,
            },
            WorkspacePermissionDef {
                id: "workspace.execute",
                title: "Command execution (AI)",
                description: "Allow the AI to run allowlisted commands via run_command tool.",
                default_granted: false,
            },
        ],
        supported_platforms: &[],
    },
    ModuleManifest {
        name: AI_LLAMA_CPP_MODULE_NAME,
        version: "0.1.0",
        description: "llama.cpp local inference provider for the AI chat module.",
        required_modules: &[AI_MODULE_NAME],
        required_permissions: &[],
        workspace_permissions: &[],
        supported_platforms: &[],
    },
    ModuleManifest {
        name: AI_OLLAMA_MODULE_NAME,
        version: "0.1.0",
        description: "Ollama local inference provider for the AI chat module.",
        required_modules: &[AI_MODULE_NAME],
        required_permissions: &[],
        workspace_permissions: &[],
        supported_platforms: &[],
    },
    ModuleManifest {
        name: AI_OPENAI_MODULE_NAME,
        version: "0.1.0",
        description: "OpenAI API provider for the AI chat module.",
        required_modules: &[AI_MODULE_NAME],
        required_permissions: &[],
        workspace_permissions: &[],
        supported_platforms: &[],
    },
    ModuleManifest {
        name: AI_RULES_MODULE_NAME,
        version: "0.1.0",
        description: "AI Rules — per-chat constraints: forbidden tools, response format, persona. Adds a configuration page to the AI sidebar.",
        required_modules: &[AI_MODULE_NAME],
        required_permissions: &[],
        workspace_permissions: &[],
        supported_platforms: &[],
    },
    ModuleManifest {
        name: AI_SKILLS_MODULE_NAME,
        version: "0.1.0",
        description: "AI Skills — named behaviours: system prompt fragments, tool allowlists, parameter overrides. Adds a configuration page to the AI sidebar.",
        required_modules: &[AI_MODULE_NAME],
        required_permissions: &[],
        workspace_permissions: &[],
        supported_platforms: &[],
    },
    ModuleManifest {
        name: AI_EXEC_CLAUDE_MODULE_NAME,
        version: "0.1.0",
        description: "Claude CLI provider — uses the installed `claude` binary with a Claude Pro subscription. No API key required.",
        required_modules: &[AI_MODULE_NAME],
        required_permissions: &[],
        workspace_permissions: &[],
        supported_platforms: &[],
    },
    ModuleManifest {
        name: AI_EXEC_CODEX_MODULE_NAME,
        version: "0.1.0",
        description: "Codex CLI provider — uses the installed `codex` binary with a ChatGPT Plus subscription. No API key required.",
        required_modules: &[AI_MODULE_NAME],
        required_permissions: &[],
        workspace_permissions: &[],
        supported_platforms: &[],
    },
    ModuleManifest {
        name: AI_EXEC_GEMINI_MODULE_NAME,
        version: "0.1.0",
        description: "Gemini CLI provider — uses the installed `gemini` binary with a Gemini Advanced subscription. No API key required.",
        required_modules: &[AI_MODULE_NAME],
        required_permissions: &[],
        workspace_permissions: &[],
        supported_platforms: &[],
    },
    ModuleManifest {
        name: AI_EXEC_AIDER_MODULE_NAME,
        version: "0.1.0",
        description: "Aider CLI provider — uses the installed `aider` binary with its own configured backend.",
        required_modules: &[AI_MODULE_NAME],
        required_permissions: &[],
        workspace_permissions: &[],
        supported_platforms: &[],
    },
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulesConfig {
    pub modules: BTreeMap<String, bool>,
    /// Persistent enable state for Python extensions discovered under `~/Arcadia/Extensions/`.
    /// Keyed by extension id (directory name or file stem, kebab-cased). Defaults to `false`
    /// for newly-discovered extensions — the user must explicitly enable each one through the
    /// Extensions settings page so that side-effectful `main.py` bodies do not run (and OS
    /// permission prompts do not fire) until the user opts in.
    #[serde(default)]
    pub python_extensions: BTreeMap<String, bool>,
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
            python_extensions: BTreeMap::new(),
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
        self.python_extensions
            .get(extension_id)
            .copied()
            .unwrap_or(false)
    }

    pub fn set_python_extension_enabled(&mut self, extension_id: &str, enabled: bool) {
        if extension_id.is_empty() {
            return;
        }
        self.python_extensions
            .insert(extension_id.to_string(), enabled);
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
            if let Some(val) = self.python_extensions.remove(*legacy) {
                self.python_extensions
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
            python_extensions: std::collections::BTreeMap::new(),
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
            python_extensions: std::collections::BTreeMap::new(),
        };
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
    fn all_manifest_required_permissions_exist_in_catalog() {
        use crate::config::permissions::is_known_permission_id;
        for m in MODULE_REGISTRY {
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
        assert!(!supports_runtime_platform_owned(&["__no_such_os__".to_string()]));
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
        assert!(cfg.python_extensions.is_empty());
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
