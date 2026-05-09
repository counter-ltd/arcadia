pub mod lan;
pub mod late;
pub mod net;
pub mod python_host;
pub mod python_registry;
pub mod remote_mirror;
pub mod remote_session;
pub mod shell;
pub mod shell_motd;
pub mod surface;

use crate::config::modules::{
    ModulesConfig, LAN_MODULE_NAME, NET_MODULE_NAME, REMOTE_SESSION_MODULE_NAME,
};
use crate::config::ConfigFile;

#[derive(Clone, Default)]
pub struct ExecutionContext {
    pub net_as: Option<String>,
    pub net_timeout_ms: Option<u64>,
}

pub struct ModuleCommand {
    pub name: &'static str,
    pub description: &'static str,
    pub run: fn(&[&str], &ExecutionContext) -> String,
}

fn module_commands(module_name: &str) -> Option<&'static [ModuleCommand]> {
    match module_name {
        lan::NAME => Some(lan::commands()),
        late::NAME => Some(late::commands()),
        net::NAME => Some(net::commands()),
        python_host::NAME => Some(python_host::commands()),
        remote_session::NAME => Some(remote_session::commands()),
        shell::NAME => Some(shell::commands()),
        shell_motd::NAME => Some(shell_motd::commands()),
        surface::NAME => Some(surface::commands()),
        _ => None,
    }
}

fn module_enabled(module_name: &str) -> Result<bool, String> {
    let cfg = ModulesConfig::load_or_create().map_err(|err| err.to_string())?;
    cfg.modules
        .get(module_name)
        .copied()
        .ok_or_else(|| "Unknown module key".to_string())
}

pub fn enabled_command_tokens() -> Vec<String> {
    let Ok(cfg) = ModulesConfig::load_or_create() else {
        return Vec::new();
    };

    let mut tokens = Vec::new();
    for (module_name, enabled) in &cfg.modules {
        if !enabled {
            continue;
        }
        if let Some(commands) = module_commands(module_name) {
            for command in commands {
                tokens.push(format!("{module_name}.{}", command.name));
            }
        }
    }
    for (token, _) in python_registry::list_commands() {
        tokens.push(token);
    }
    tokens
}

pub fn enabled_module_names() -> Vec<String> {
    let Ok(cfg) = ModulesConfig::load_or_create() else {
        return Vec::new();
    };

    cfg.modules
        .iter()
        .filter(|(_, enabled)| **enabled)
        .filter_map(|(module_name, _)| {
            if module_commands(module_name).is_some() {
                Some(module_name.clone())
            } else {
                None
            }
        })
        .collect()
}

pub fn enabled_module_command_names(module_name: &str) -> Vec<String> {
    if !matches!(module_enabled(module_name), Ok(true)) {
        return Vec::new();
    }

    let Some(commands) = module_commands(module_name) else {
        return Vec::new();
    };
    commands
        .iter()
        .map(|command| command.name.to_string())
        .collect()
}

/// Dispatches `module.command` locally or forwards via `ExecutionContext::net_as` (e.g. `lan:<host>`).
/// LAN forwarding uses one code path for every registered token — peer runs normal module checks.
pub fn execute_command(
    token: &str,
    args: &[&str],
    context: &ExecutionContext,
) -> Result<Option<String>, String> {
    let Some((module_name, command_name)) = token.split_once('.') else {
        return Ok(None);
    };

    let Some(commands) = module_commands(module_name) else {
        // Python extension fallback: dispatch to dynamically registered handlers.
        if let Some(result) = python_registry::try_dispatch(token, args) {
            return Ok(Some(result));
        }
        return Ok(None);
    };

    let Some(command) = commands.iter().find(|command| command.name == command_name) else {
        return Err(format!("Unknown command: {token}"));
    };

    if (context.net_as.is_some() || context.net_timeout_ms.is_some())
        && !module_enabled(NET_MODULE_NAME)?
    {
        return Err("Global flags --net:* require module net to be enabled".to_string());
    }

    if let Some(route) = &context.net_as {
        if let Some(target) = route.strip_prefix("lan:") {
            if target.is_empty() {
                return Err("Invalid LAN route: use lan:<host/ip/alias>".to_string());
            }
            if !module_enabled(REMOTE_SESSION_MODULE_NAME)? {
                return Err(
                    "LAN command routing requires remote-session module to be enabled locally"
                        .to_string(),
                );
            }
            if !module_enabled(LAN_MODULE_NAME)? {
                return Err(
                    "LAN command routing requires lan module to be enabled locally".to_string(),
                );
            }
            let response =
                lan::execute_remote_command(target, token, args, context.net_timeout_ms)?;
            return Ok(Some(response));
        }
        return Err(format!("Unsupported net route: {route}"));
    }

    if !module_enabled(module_name)? {
        return Err(format!("Module {module_name} is disabled"));
    }

    Ok(Some((command.run)(args, context)))
}

pub fn enabled_command_help_lines() -> Vec<String> {
    let Ok(cfg) = ModulesConfig::load_or_create() else {
        return Vec::new();
    };

    let mut lines = Vec::new();
    for (module_name, enabled) in &cfg.modules {
        if !enabled {
            continue;
        }
        if let Some(commands) = module_commands(module_name) {
            for command in commands {
                lines.push(format!(
                    "- {module_name}.{}: {}",
                    command.name, command.description
                ));
            }
        }
    }
    lines
}

pub fn all_command_entries() -> Vec<(String, String)> {
    let Ok(cfg) = ModulesConfig::load_or_create() else {
        return Vec::new();
    };

    let mut entries = Vec::new();
    for (module_name, enabled) in &cfg.modules {
        if !enabled {
            continue;
        }
        if let Some(commands) = module_commands(module_name) {
            for command in commands {
                entries.push((
                    format!("{module_name}.{}", command.name),
                    command.description.to_string(),
                ));
            }
        }
    }
    for (token, description) in python_registry::list_commands() {
        entries.push((token, description));
    }
    entries
}

pub fn load_all() {
    let _known_modules = [
        lan::NAME,
        late::NAME,
        net::NAME,
        python_host::NAME,
        remote_session::NAME,
        shell::NAME,
        shell_motd::NAME,
        surface::NAME,
    ];

    if let Err(err) = ModulesConfig::load_or_create() {
        eprintln!("Failed to load modules config: {err}");
    }

    // Service binds port regardless; respects lan_enabled() per-request.
    if let Err(err) = lan::start_service() {
        eprintln!("Failed to start LAN service: {err}");
    }
}

pub fn shutdown_all() {
    lan::stop_service();
}
