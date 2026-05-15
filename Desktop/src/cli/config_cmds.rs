use std::path::PathBuf;
use std::process::Command;

use arcadia_core::config::commandline::CommandlineConfig;
use arcadia_core::config::modules::ModulesConfig;
use arcadia_core::config::ConfigFile;

#[derive(Clone, Copy)]
pub struct ConfigProviderSpec {
    pub name: &'static str,
    pub list_keys: fn() -> Result<Vec<String>, String>,
    pub get_value: fn(&str) -> Result<String, String>,
    pub set_value: fn(&str, &str) -> Result<(), String>,
    pub reset: fn(Option<&str>) -> Result<(), String>,
    pub ensure_exists: fn() -> Result<(), String>,
    pub file_path: fn() -> Result<PathBuf, String>,
}

pub static CONFIG_PROVIDERS: &[ConfigProviderSpec] = &[
    ConfigProviderSpec {
        name: "commandline",
        list_keys: commandline_keys,
        get_value: commandline_get,
        set_value: commandline_set,
        reset: commandline_reset,
        ensure_exists: commandline_ensure_exists,
        file_path: commandline_path,
    },
    ConfigProviderSpec {
        name: "modules",
        list_keys: modules_keys,
        get_value: modules_get,
        set_value: modules_set,
        reset: modules_reset,
        ensure_exists: modules_ensure_exists,
        file_path: modules_path,
    },
];

pub fn provider_names() -> Vec<String> {
    CONFIG_PROVIDERS
        .iter()
        .map(|p| p.name.to_string())
        .collect()
}

pub fn resolve_provider(name: &str) -> Option<ConfigProviderSpec> {
    CONFIG_PROVIDERS.iter().find(|p| p.name == name).copied()
}

pub fn scoped_key_candidates() -> Vec<String> {
    let mut candidates = Vec::new();
    for provider in CONFIG_PROVIDERS {
        if let Ok(keys) = (provider.list_keys)() {
            for key in keys {
                candidates.push(format!("{}.{}", provider.name, key));
            }
        }
    }
    candidates
}

pub fn commandline_keys() -> Result<Vec<String>, String> {
    Ok(vec![
        "input_symbol".to_string(),
        "output_symbol".to_string(),
        "input_color".to_string(),
        "output_color".to_string(),
        "clear_on_start".to_string(),
    ])
}

fn commandline_get_value(cfg: &CommandlineConfig, key: &str) -> Option<String> {
    match key {
        "input_symbol" => Some(cfg.input_symbol.clone()),
        "output_symbol" => Some(cfg.output_symbol.clone()),
        "input_color" => Some(cfg.input_color.clone()),
        "output_color" => Some(cfg.output_color.clone()),
        "clear_on_start" => Some(cfg.clear_on_start.to_string()),
        _ => None,
    }
}

pub fn commandline_get(key: &str) -> Result<String, String> {
    let cfg = super::settings();
    commandline_get_value(&cfg, key).ok_or_else(|| "Unknown config key".to_string())
}

pub fn commandline_set(key: &str, value: &str) -> Result<(), String> {
    let mut guard = super::settings_lock()
        .write()
        .map_err(|_| "Failed to update config: settings lock poisoned".to_string())?;
    let applied = match key {
        "input_symbol" => {
            guard.input_symbol = value.to_string();
            true
        }
        "output_symbol" => {
            guard.output_symbol = value.to_string();
            true
        }
        "input_color" => {
            guard.input_color = value.to_string();
            true
        }
        "output_color" => {
            guard.output_color = value.to_string();
            true
        }
        "clear_on_start" => match value.to_ascii_lowercase().as_str() {
            "true" => {
                guard.clear_on_start = true;
                true
            }
            "false" => {
                guard.clear_on_start = false;
                true
            }
            _ => return Err("clear_on_start must be true or false".to_string()),
        },
        _ => return Err("Unknown config key".to_string()),
    };
    if applied {
        guard
            .save()
            .map_err(|err| format!("Config updated in-memory, save failed: {err}"))?;
    }
    Ok(())
}

pub fn commandline_reset(target: Option<&str>) -> Result<(), String> {
    let mut guard = super::settings_lock()
        .write()
        .map_err(|_| "Failed to reset config: settings lock poisoned".to_string())?;
    let defaults = CommandlineConfig::default();
    match target {
        None => *guard = defaults,
        Some("input_symbol") => guard.input_symbol = defaults.input_symbol,
        Some("output_symbol") => guard.output_symbol = defaults.output_symbol,
        Some("input_color") => guard.input_color = defaults.input_color,
        Some("output_color") => guard.output_color = defaults.output_color,
        Some("clear_on_start") => guard.clear_on_start = defaults.clear_on_start,
        Some(_) => return Err("Unknown config key".to_string()),
    }
    guard
        .save()
        .map_err(|err| format!("Config reset in-memory, save failed: {err}"))
}

fn commandline_ensure_exists() -> Result<(), String> {
    CommandlineConfig::load_or_create()
        .map(|_| ())
        .map_err(|err| err.to_string())
}

fn commandline_path() -> Result<PathBuf, String> {
    CommandlineConfig::file_path().map_err(|err| err.to_string())
}

pub fn modules_keys() -> Result<Vec<String>, String> {
    let cfg = ModulesConfig::load_or_create().map_err(|err| err.to_string())?;
    Ok(cfg.modules.keys().cloned().collect())
}

fn modules_get(key: &str) -> Result<String, String> {
    let cfg = ModulesConfig::load_or_create().map_err(|err| err.to_string())?;
    cfg.modules
        .get(key)
        .map(|enabled| enabled.to_string())
        .ok_or_else(|| "Unknown module key".to_string())
}

fn modules_set(key: &str, value: &str) -> Result<(), String> {
    let mut cfg = ModulesConfig::load_or_create().map_err(|err| err.to_string())?;
    let parsed = match value.to_ascii_lowercase().as_str() {
        "true" => true,
        "false" => false,
        _ => return Err("Module value must be true or false".to_string()),
    };
    cfg.set_module_state(key, parsed)?;
    cfg.save().map_err(|err| err.to_string())?;
    Ok(())
}

fn modules_reset(target: Option<&str>) -> Result<(), String> {
    match target {
        None => {
            ModulesConfig::default()
                .save()
                .map_err(|err| err.to_string())?;
            Ok(())
        }
        Some(key) => {
            let defaults = ModulesConfig::default();
            let default_value = defaults
                .modules
                .get(key)
                .copied()
                .ok_or_else(|| "Unknown module key".to_string())?;
            let mut cfg = ModulesConfig::load_or_create().map_err(|err| err.to_string())?;
            cfg.set_module_state(key, default_value)?;
            cfg.save().map_err(|err| err.to_string())?;
            Ok(())
        }
    }
}

fn modules_ensure_exists() -> Result<(), String> {
    ModulesConfig::load_or_create()
        .map(|_| ())
        .map_err(|err| err.to_string())
}

fn modules_path() -> Result<PathBuf, String> {
    ModulesConfig::file_path().map_err(|err| err.to_string())
}

fn print_available_configs() {
    super::print_response("Available configs:");
    for name in provider_names() {
        super::print_response(&format!("- {name}"));
    }
}

fn show_config_keys(config_name: &str) {
    let Some(provider) = resolve_provider(config_name) else {
        super::print_response("Unknown config");
        return;
    };
    match (provider.list_keys)() {
        Ok(keys) => {
            super::print_response(&format!("{} keys:", provider.name));
            for key in keys {
                super::print_response(&format!("- {key}"));
            }
        }
        Err(err) => {
            super::print_response(&format!("Failed to load {} config: {err}", provider.name));
        }
    }
}

fn config_get(key: &str) {
    match commandline_get(key) {
        Ok(value) => super::print_response(&format!("{key} = {value}")),
        Err(err) => super::print_response(&err),
    }
}

fn config_get_scoped(reference: &str) {
    let Some((config_name, key)) = reference.split_once('.') else {
        super::print_response(
            "Use scoped format: <config>.<key> (example: commandline.clear_on_start)",
        );
        return;
    };
    let Some(provider) = resolve_provider(config_name) else {
        super::print_response("Unknown config");
        return;
    };
    match (provider.get_value)(key) {
        Ok(value) => super::print_response(&format!("{config_name}.{key} = {value}")),
        Err(err) => super::print_response(&err),
    }
}

fn config_set(key: &str, value: &str) {
    match commandline_set(key, value) {
        Ok(_) => super::print_response("Config updated"),
        Err(err) => super::print_response(&err),
    }
}

fn config_set_scoped(reference: &str, value: &str) {
    let Some((config_name, key)) = reference.split_once('.') else {
        super::print_response(
            "Use scoped format: <config>.<key> (example: commandline.clear_on_start)",
        );
        return;
    };
    let Some(provider) = resolve_provider(config_name) else {
        super::print_response("Unknown config");
        return;
    };
    match (provider.set_value)(key, value) {
        Ok(_) => super::print_response("Config updated"),
        Err(err) => super::print_response(&err),
    }
}

fn config_reset() {
    match commandline_reset(None) {
        Ok(_) => super::print_response("Config reset to defaults"),
        Err(err) => super::print_response(&err),
    }
}

fn config_reset_scoped(reference: &str) {
    let (config_name, target_key) = match reference.split_once('.') {
        Some((name, key)) => (name, Some(key)),
        None => (reference, None),
    };
    let Some(provider) = resolve_provider(config_name) else {
        super::print_response("Unknown config");
        return;
    };
    match (provider.reset)(target_key) {
        Ok(_) => {
            if target_key.is_some() {
                super::print_response("Config key reset to default");
            } else {
                super::print_response("Config reset to defaults");
            }
        }
        Err(err) => super::print_response(&err),
    }
}

pub fn open_config(config_name: &str) {
    let Some(provider) = resolve_provider(config_name) else {
        super::print_response("Unknown config");
        return;
    };
    if let Err(err) = (provider.ensure_exists)() {
        super::print_response(&format!("Failed to create {} config: {err}", provider.name));
        return;
    }
    let path = match (provider.file_path)() {
        Ok(path) => path,
        Err(err) => {
            super::print_response(&format!(
                "Failed to resolve {} config path: {err}",
                provider.name
            ));
            return;
        }
    };

    let status = {
        #[cfg(target_os = "macos")]
        {
            Command::new("open").arg(&path).status()
        }
        #[cfg(target_os = "linux")]
        {
            Command::new("xdg-open").arg(&path).status()
        }
        #[cfg(target_os = "windows")]
        {
            Command::new("cmd")
                .args(["/C", "start", "", &path.to_string_lossy()])
                .status()
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            Err::<std::process::ExitStatus, _>(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "open not supported on this platform",
            ))
        }
    };

    match status {
        Ok(exit) if exit.success() => super::print_response(&format!("Opened {}", path.display())),
        Ok(exit) => super::print_response(&format!(
            "Failed to open {} (exit code: {:?})",
            path.display(),
            exit.code()
        )),
        Err(err) => super::print_response(&format!(
            "Failed to launch editor for {}: {err}",
            path.display()
        )),
    }
}

pub fn handle_configuration(parts: &[&str]) {
    match parts {
        ["configuration"] => print_available_configs(),
        ["configuration", "show"] => print_available_configs(),
        ["configuration", "show", config_name] => show_config_keys(config_name),
        ["configuration", "get", key] if key.contains('.') => config_get_scoped(key),
        ["configuration", "get", key] => config_get(key),
        ["configuration", "set", key, value] if key.contains('.') => config_set_scoped(key, value),
        ["configuration", "set", key, value] => config_set(key, value),
        ["configuration", "reset", target] => config_reset_scoped(target),
        ["configuration", "reset"] => config_reset(),
        ["configuration", name] => open_config(name),
        _ => {
            super::print_response("Usage: configuration <name> | configuration [show|get <key>|get <config>.<key>|set <key> <value>|set <config>.<key> <value>|reset|reset <config>|reset <config>.<key>]");
            super::print_response(
                "Keys: input_symbol, output_symbol, input_color, output_color, clear_on_start",
            );
        }
    }
}
