//! Runtime registry for WASM modules loaded from `~/Arcadia/Modules/`.
//!
//! Runtime-agnostic — like [`python_registry`](crate::modules::python_registry), this holds
//! owned module info and boundary-crossing command closures, and knows nothing about `wasmi`.
//! The `wasmi` host lives in the separate `arcadia-wasm` crate, which installs its loader
//! callbacks here via [`set_reload_handler`] / [`set_load_one_handler`].
//!
//! Lifecycle mirrors the Python host: discovery registers a disabled **stub**
//! ([`register_discovered`], `loaded = false`); the user opts in through the settings UI;
//! [`load_one`] runs the host loader which instantiates the `.wasm` and calls
//! [`register_module`] + [`register_command`].

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

use crate::config::modules::supports_runtime_platform_owned;

/// Command handler — a boundary-crossing closure created by `arcadia-wasm` that owns a handle
/// into the module's `wasmi` instance.
type DynCommandFn = Arc<dyn Fn(Vec<String>) -> String + Send + Sync + 'static>;
/// Host-supplied callback: rescan `~/Arcadia/Modules/` and reload all enabled modules.
type ReloadFn = Arc<dyn Fn() -> Result<(), String> + Send + Sync + 'static>;
/// Host-supplied loader for one module: `(stub_id, path) -> canonical name`.
type LoadOneFn = Arc<dyn Fn(String, PathBuf) -> Result<String, String> + Send + Sync + 'static>;

pub struct WasmModuleInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub enabled: bool,
    /// Declared in the module's `arcadia.manifest` section, for the first-enable / settings UI.
    pub required_permissions: Vec<String>,
    /// Empty = all platforms. Otherwise a whitelist of `macos` / `windows` / `linux` / `ios`.
    pub supported_platforms: Vec<String>,
    /// Author-declared tags.
    pub tags: Vec<String>,
    /// On-disk `.wasm` path — set by [`register_discovered`] so [`load_one`] can find it.
    pub path: Option<PathBuf>,
    /// `true` once the module has been instantiated and its body has run. Stubs start `false`.
    pub loaded: bool,
    /// ABI version declared in the manifest.
    pub abi_version: u32,
}

struct CommandRecord {
    description: String,
    required_permissions: Vec<String>,
    handler: DynCommandFn,
}

struct WasmRegistry {
    modules: Vec<WasmModuleInfo>,
    commands: HashMap<String, CommandRecord>,
    reload_fn: Option<ReloadFn>,
    load_one_fn: Option<LoadOneFn>,
}

impl WasmRegistry {
    fn new() -> Self {
        Self {
            modules: Vec::new(),
            commands: HashMap::new(),
            reload_fn: None,
            load_one_fn: None,
        }
    }
}

fn registry() -> &'static Mutex<WasmRegistry> {
    static REGISTRY: OnceLock<Mutex<WasmRegistry>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(WasmRegistry::new()))
}

/// Pre-register a module discovered on disk before it has been instantiated. The host calls
/// this for every `.wasm` it finds (manifest read from the `arcadia.manifest` custom section)
/// so the settings page can list and toggle modules without instantiating them.
///
/// `persisted_enabled` comes from `ModulesConfig.module_state[name]`.
pub fn register_discovered(
    name: String,
    path: PathBuf,
    persisted_enabled: bool,
    version: String,
    description: String,
    required_permissions: Vec<String>,
    supported_platforms: Vec<String>,
    tags: Vec<String>,
    abi_version: u32,
) {
    if name.is_empty() {
        return;
    }
    if let Ok(mut reg) = registry().lock() {
        if let Some(existing) = reg.modules.iter_mut().find(|m| m.name == name) {
            existing.path = Some(path);
            existing.enabled = persisted_enabled;
            if !existing.loaded {
                existing.version = version;
                existing.description = description;
                existing.required_permissions = required_permissions;
                existing.supported_platforms = supported_platforms;
                existing.tags = tags;
                existing.abi_version = abi_version;
            }
            return;
        }
        reg.modules.push(WasmModuleInfo {
            name,
            version,
            description,
            enabled: persisted_enabled,
            required_permissions,
            supported_platforms,
            tags,
            path: Some(path),
            loaded: false,
            abi_version,
        });
    }
}

/// Mark a module's body as loaded. Called by the host after instantiation succeeds.
/// Preserves the prior `enabled` / `path` from the stub.
pub fn register_module(name: String) {
    if let Ok(mut reg) = registry().lock() {
        if let Some(m) = reg.modules.iter_mut().find(|m| m.name == name) {
            m.loaded = true;
        }
    }
}

pub fn register_command(
    token: String,
    description: String,
    handler: DynCommandFn,
    required_permissions: Vec<String>,
) {
    if let Ok(mut reg) = registry().lock() {
        reg.commands.insert(
            token,
            CommandRecord {
                description,
                required_permissions,
                handler,
            },
        );
    }
}

pub fn set_module_enabled(name: &str, enabled: bool) {
    if let Ok(mut reg) = registry().lock() {
        if let Some(m) = reg.modules.iter_mut().find(|m| m.name == name) {
            m.enabled = enabled;
        }
    }
}

pub fn module_enabled(name: &str) -> bool {
    let Ok(reg) = registry().lock() else {
        return false;
    };
    reg.modules
        .iter()
        .find(|m| m.name == name)
        .map(|m| m.enabled)
        .unwrap_or(false)
}

/// `true` once a module has been instantiated. Used to avoid loading the same `.wasm` twice.
pub fn module_body_loaded(name: &str) -> bool {
    let Ok(reg) = registry().lock() else {
        return false;
    };
    reg.modules
        .iter()
        .find(|m| m.name == name)
        .is_some_and(|m| m.loaded)
}

/// On-disk `.wasm` path for a module id, if discovered.
pub fn module_path(name: &str) -> Option<PathBuf> {
    let reg = registry().lock().ok()?;
    reg.modules
        .iter()
        .find(|m| m.name == name)
        .and_then(|m| m.path.clone())
}

/// Forget the command closures contributed by a module — used on disable so stale handlers
/// (which hold a `wasmi` instance handle) stop firing and the instance can drop.
pub fn unregister_module_contributions(name: &str) {
    if name.is_empty() {
        return;
    }
    if let Ok(mut reg) = registry().lock() {
        let prefix = format!("{name}.");
        reg.commands.retain(|token, _| !token.starts_with(&prefix));
        if let Some(m) = reg.modules.iter_mut().find(|m| m.name == name) {
            m.loaded = false;
        }
    }
}

pub fn set_reload_handler(f: ReloadFn) {
    if let Ok(mut reg) = registry().lock() {
        reg.reload_fn = Some(f);
    }
}

pub fn set_load_one_handler(f: LoadOneFn) {
    if let Ok(mut reg) = registry().lock() {
        reg.load_one_fn = Some(f);
    }
}

/// Instantiate a single module on demand via the host-registered loader. Used by
/// `wasm-host.module-enable` to bring a disabled stub online.
pub fn load_one(stub_id: String, path: PathBuf) -> Result<String, String> {
    let load_fn = {
        let reg = registry()
            .lock()
            .map_err(|_| "WASM registry poisoned".to_string())?;
        reg.load_one_fn.as_ref().map(Arc::clone)
    };
    match load_fn {
        Some(f) => f(stub_id, path),
        None => Err(
            "WASM host not initialized. Restart the app with wasm-host enabled.".to_string(),
        ),
    }
}

pub fn reload() -> Result<(), String> {
    let reload_fn = {
        let reg = registry()
            .lock()
            .map_err(|_| "WASM registry poisoned".to_string())?;
        reg.reload_fn.as_ref().map(Arc::clone)
    };
    match reload_fn {
        Some(f) => f(),
        None => Err(
            "WASM host not initialized. Restart the app with wasm-host enabled.".to_string(),
        ),
    }
}

/// `(name, version, description, enabled, permissions, platforms, tags, loaded)`.
pub fn list_modules() -> Vec<(String, String, String, bool, Vec<String>, Vec<String>, Vec<String>, bool)>
{
    let Ok(reg) = registry().lock() else {
        return Vec::new();
    };
    reg.modules
        .iter()
        .map(|m| {
            (
                m.name.clone(),
                m.version.clone(),
                m.description.clone(),
                m.enabled,
                m.required_permissions.clone(),
                m.supported_platforms.clone(),
                m.tags.clone(),
                m.loaded,
            )
        })
        .collect()
}

pub fn list_commands() -> Vec<(String, String)> {
    let Ok(reg) = registry().lock() else {
        return Vec::new();
    };
    reg.commands
        .iter()
        .map(|(token, rec)| (token.clone(), rec.description.clone()))
        .collect()
}

/// `true` when the module declares no platform list or the host OS is listed.
pub fn module_supported_at_runtime(name: &str) -> bool {
    let Ok(reg) = registry().lock() else {
        return true;
    };
    reg.modules
        .iter()
        .find(|m| m.name == name)
        .map(|m| supports_runtime_platform_owned(&m.supported_platforms))
        .unwrap_or(true)
}

/// Dispatch a `module.verb` token to a loaded WASM module. `Ok(None)` = not a WASM token
/// (the caller falls through). Mirrors `python_registry::try_dispatch`.
pub fn try_dispatch(token: &str, args: &[&str]) -> Result<Option<String>, String> {
    let module_id = token
        .split_once('.')
        .map(|(a, _)| a)
        .ok_or_else(|| "Invalid command token".to_string())?;

    let (handler, perms) = {
        let reg = registry()
            .lock()
            .map_err(|_| "WASM registry poisoned".to_string())?;
        if let Some(m) = reg.modules.iter().find(|m| m.name == module_id) {
            if !m.enabled {
                return Ok(None);
            }
            if !supports_runtime_platform_owned(&m.supported_platforms) {
                return Err(format!(
                    "WASM module '{module_id}' is not supported on this platform"
                ));
            }
        }
        let Some(rec) = reg.commands.get(token) else {
            return Ok(None);
        };
        (Arc::clone(&rec.handler), rec.required_permissions.clone())
    };

    use crate::config::permissions::{is_known_permission_id, PermissionSubject, PermissionsConfig};
    use crate::config::ConfigFile;

    let cfg = PermissionsConfig::load_or_create().map_err(|e| e.to_string())?;
    let subj = PermissionSubject::wasm(module_id.to_string());
    for p in &perms {
        if !is_known_permission_id(p) {
            return Err(format!(
                "WASM module '{module_id}' references unknown permission id: {p}"
            ));
        }
        if !cfg.effective_allowed(&subj, p) {
            return Err(format!(
                "Permission denied: {p} (subject {})",
                subj.storage_key()
            ));
        }
    }

    let args_vec: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    Ok(Some(handler(args_vec)))
}

/// Drop all registry state — used by `reload` before a rescan.
pub fn clear() {
    if let Ok(mut reg) = registry().lock() {
        reg.modules.clear();
        reg.commands.clear();
    }
}
