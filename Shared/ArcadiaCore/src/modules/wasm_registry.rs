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

thread_local! {
    /// Module ids currently mid-dispatch on this thread. A module re-entering its own
    /// dispatch (via `host_execute_command`) would deadlock its per-module `Mutex`, so
    /// same-module recursion is rejected. Cross-module chains (A→B→A) are allowed —
    /// distinct ids, distinct `Mutex`es.
    static DISPATCH_STACK: std::cell::RefCell<Vec<String>> =
        const { std::cell::RefCell::new(Vec::new()) };
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

/// Bundle root for a module: the directory containing its `.wasm`. For a folder bundle
/// (`Modules/<name>/module.wasm`) this is `Modules/<name>`; for a loose `.wasm` it is
/// `Modules/` itself. Mirrors `python_registry::extension_bundle_root`.
pub fn module_bundle_root(name: &str) -> Option<PathBuf> {
    module_path(name)?.parent().map(|p| p.to_path_buf())
}

/// Asset directory for a module: `<bundle_root>/Assets`.
pub fn module_assets_dir(name: &str) -> Option<PathBuf> {
    Some(module_bundle_root(name)?.join("Assets"))
}

fn validate_asset_relative(relative: &str) -> Result<(), String> {
    use std::path::{Component, Path};
    let s = relative.trim();
    if s.is_empty() {
        return Err("empty relative asset path".into());
    }
    let p = Path::new(s);
    if p.is_absolute() {
        return Err("absolute asset paths are not allowed".into());
    }
    for c in p.components() {
        match c {
            Component::Normal(os) => {
                if os.to_str().is_none() {
                    return Err("asset path must be UTF-8".into());
                }
            }
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err("invalid asset path".into());
            }
        }
    }
    Ok(())
}

/// Resolve a safe relative path (no `..`, not absolute) under a module's `Assets/` dir.
pub fn resolve_module_asset_path(name: &str, relative_under_assets: &str) -> Result<PathBuf, String> {
    validate_asset_relative(relative_under_assets)?;
    let base = module_assets_dir(name)
        .ok_or_else(|| format!("no on-disk path for WASM module '{name}'"))?;
    Ok(base.join(relative_under_assets.trim_start_matches(['/', '\\'])))
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

    // Re-entrancy guard — see DISPATCH_STACK.
    let already_dispatching = DISPATCH_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        if stack.iter().any(|m| m == module_id) {
            true
        } else {
            stack.push(module_id.to_string());
            false
        }
    });
    if already_dispatching {
        return Err(format!(
            "WASM module '{module_id}' tried to call one of its own commands \
             (same-module recursion is not allowed)"
        ));
    }

    let args_vec: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let result = handler(args_vec);

    DISPATCH_STACK.with(|stack| {
        stack.borrow_mut().pop();
    });
    Ok(Some(result))
}

/// Drop all registry state — used by `reload` before a rescan.
pub fn clear() {
    if let Ok(mut reg) = registry().lock() {
        reg.modules.clear();
        reg.commands.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex as StdMutex;

    static TEST_LOCK: StdMutex<()> = StdMutex::new(());

    /// Serialize tests (the registry is a process-global) and start each from empty.
    fn guard() -> std::sync::MutexGuard<'static, ()> {
        let g = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear();
        g
    }

    fn discover(name: &str, enabled: bool) {
        register_discovered(
            name.to_string(),
            PathBuf::from(format!("/fake/Modules/{name}.wasm")),
            enabled,
            "1.0.0".to_string(),
            "test module".to_string(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            1,
        );
    }

    #[test]
    fn discover_then_list() {
        let _g = guard();
        discover("alpha", false);
        let mods = list_modules();
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].0, "alpha");
        assert!(!mods[0].3, "freshly discovered module is disabled");
        assert!(!mods[0].7, "freshly discovered module is not loaded");
    }

    #[test]
    fn enable_disable_toggle() {
        let _g = guard();
        discover("beta", false);
        set_module_enabled("beta", true);
        assert!(module_enabled("beta"));
        set_module_enabled("beta", false);
        assert!(!module_enabled("beta"));
    }

    #[test]
    fn clear_empties_registry() {
        let _g = guard();
        discover("gamma", true);
        register_command("gamma.x".to_string(), String::new(), Arc::new(|_| String::new()), Vec::new());
        clear();
        assert!(list_modules().is_empty());
        assert!(list_commands().is_empty());
    }

    #[test]
    fn reload_without_handler_errors() {
        let _g = guard();
        assert!(reload().is_err(), "reload before the host installs a handler must error");
    }

    #[test]
    fn unregister_drops_module_commands() {
        let _g = guard();
        discover("delta", true);
        register_command("delta.a".to_string(), String::new(), Arc::new(|_| String::new()), Vec::new());
        register_command("other.a".to_string(), String::new(), Arc::new(|_| String::new()), Vec::new());
        unregister_module_contributions("delta");
        let tokens: Vec<String> = list_commands().into_iter().map(|(t, _)| t).collect();
        assert!(!tokens.contains(&"delta.a".to_string()));
        assert!(tokens.contains(&"other.a".to_string()));
    }

    #[test]
    fn same_module_recursion_is_rejected() {
        let _g = guard();
        discover("rec", true);
        register_command(
            "rec.inner".to_string(),
            String::new(),
            Arc::new(|_| "inner-ran".to_string()),
            Vec::new(),
        );
        // The outer handler re-enters dispatch for one of its own module's commands.
        register_command(
            "rec.outer".to_string(),
            String::new(),
            Arc::new(|_| match try_dispatch("rec.inner", &[]) {
                Ok(Some(s)) => s,
                Ok(None) => "none".to_string(),
                Err(e) => format!("ERR:{e}"),
            }),
            Vec::new(),
        );
        let out = try_dispatch("rec.outer", &[])
            .expect("dispatch must not fail")
            .expect("outer command exists");
        assert!(out.starts_with("ERR:"), "same-module recursion must be rejected, got: {out}");
        assert!(out.contains("recursion"), "got: {out}");
    }
}
