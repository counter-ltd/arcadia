use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

pub struct PythonModuleInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub enabled: bool,
}

type DynCommandFn = Arc<dyn Fn(Vec<String>) -> String + Send + Sync + 'static>;
type ReloadFn = Arc<dyn Fn() -> Result<(), String> + Send + Sync + 'static>;

struct PythonRegistry {
    modules: Vec<PythonModuleInfo>,
    commands: HashMap<String, (String, DynCommandFn)>, // token → (description, fn)
    reload_fn: Option<ReloadFn>,
}

impl PythonRegistry {
    fn new() -> Self {
        Self { modules: Vec::new(), commands: HashMap::new(), reload_fn: None }
    }
}

fn registry() -> &'static Mutex<PythonRegistry> {
    static REGISTRY: OnceLock<Mutex<PythonRegistry>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(PythonRegistry::new()))
}

pub fn register_module(name: String, version: String, description: String) {
    if let Ok(mut reg) = registry().lock() {
        // Preserve existing enabled state if re-registering (e.g. after reload).
        let was_enabled = reg.modules.iter().find(|m| m.name == name).map(|m| m.enabled);
        reg.modules.retain(|m| m.name != name);
        reg.modules.push(PythonModuleInfo {
            name,
            version,
            description,
            enabled: was_enabled.unwrap_or(true),
        });
    }
}

pub fn set_extension_enabled(name: &str, enabled: bool) {
    if let Ok(mut reg) = registry().lock() {
        if let Some(m) = reg.modules.iter_mut().find(|m| m.name == name) {
            m.enabled = enabled;
        }
    }
}

pub fn register_command(token: String, description: String, handler: DynCommandFn) {
    if let Ok(mut reg) = registry().lock() {
        reg.commands.insert(token, (description, handler));
    }
}

pub fn set_reload_handler(f: ReloadFn) {
    if let Ok(mut reg) = registry().lock() {
        reg.reload_fn = Some(f);
    }
}

pub fn clear() {
    if let Ok(mut reg) = registry().lock() {
        reg.modules.clear();
        reg.commands.clear();
    }
}

pub fn try_dispatch(token: &str, args: &[&str]) -> Option<String> {
    let handler = {
        let reg = registry().lock().ok()?;
        // Check that the owning extension module is enabled.
        if let Some(module_name) = token.split_once('.').map(|(m, _)| m) {
            if let Some(m) = reg.modules.iter().find(|m| m.name == module_name) {
                if !m.enabled {
                    return None;
                }
            }
        }
        reg.commands.get(token).map(|(_, f)| Arc::clone(f))?
    };
    let args_vec: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    Some(handler(args_vec))
}

pub fn reload() -> Result<(), String> {
    let reload_fn = {
        let reg = registry().lock().map_err(|_| "Registry poisoned".to_string())?;
        reg.reload_fn.as_ref().map(Arc::clone)
    };
    match reload_fn {
        Some(f) => f(),
        None => Err(
            "Python host not initialized. Restart the app with python-host enabled.".to_string(),
        ),
    }
}

pub fn list_modules() -> Vec<(String, String, String, bool)> {
    let Ok(reg) = registry().lock() else { return Vec::new() };
    reg.modules
        .iter()
        .map(|m| (m.name.clone(), m.version.clone(), m.description.clone(), m.enabled))
        .collect()
}

pub fn list_commands() -> Vec<(String, String)> {
    let Ok(reg) = registry().lock() else { return Vec::new() };
    reg.commands
        .iter()
        .map(|(token, (desc, _))| (token.clone(), desc.clone()))
        .collect()
}
