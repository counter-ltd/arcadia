pub mod ai;
pub mod ai_chat_store;
pub mod ai_context;
pub mod ai_exec_cli;
pub mod ai_sandbox;
pub mod ai_tools;
pub mod ai_types;
pub mod animation;
pub mod code_editor;
pub mod cursor;
pub mod lan;
pub mod late;
pub mod llama_cpp;
pub mod net;
pub mod notification;
pub mod ollama;
pub mod openai;
pub mod overlay;
pub mod overlay_hud_sprite;
pub mod permissions;
pub mod python_host;
pub mod python_registry;
pub mod remote_mirror;
pub mod remote_session;
pub mod shell;
pub mod shell_motd;
pub mod style_tokens;
pub mod surface;
pub mod tray;
pub mod workspace;

use crate::config::modules::{
    supports_runtime_platform, ModulesConfig, AI_LLAMA_CPP_MODULE_NAME, AI_MODULE_NAME,
    AI_OLLAMA_MODULE_NAME, AI_OPENAI_MODULE_NAME, CODE_EDITOR_MODULE_NAME, CURSOR_MODULE_NAME,
    LAN_MODULE_NAME, NET_MODULE_NAME, NOTIFICATION_MODULE_NAME, OVERLAY_MODULE_NAME,
    REMOTE_SESSION_MODULE_NAME, TERMINAL_MODULE_NAME, TERMINAL_MOTD_MODULE_NAME, TRAY_MODULE_NAME,
    WORKSPACE_MODULE_NAME,
};
use crate::config::permissions::{self as perm_cfg, PermissionSubject, PermissionsConfig};
use crate::config::ConfigFile;

#[derive(Clone, Default)]
pub struct ExecutionContext {
    pub net_as: Option<String>,
    pub net_timeout_ms: Option<u64>,
    /// When set (e.g. Python extension on the stack), native command permission checks also
    /// accept matching grants on `python:<this id>` for the same permission ids.
    pub invoking_python_extension: Option<String>,
}

pub struct ModuleCommand {
    pub name: &'static str,
    pub description: &'static str,
    /// Extra gates beyond module enable; empty = none.
    pub required_permissions: &'static [&'static str],
    pub run: fn(&[&str], &ExecutionContext) -> String,
}

fn module_commands(module_key: &str) -> Option<&'static [ModuleCommand]> {
    match module_key {
        animation::NAME => Some(animation::commands()),
        TERMINAL_MODULE_NAME | shell::NAME => Some(shell::commands()),
        TERMINAL_MOTD_MODULE_NAME | shell_motd::NAME => Some(shell_motd::commands()),
        CURSOR_MODULE_NAME => Some(cursor::commands()),
        lan::NAME => Some(lan::commands()),
        late::NAME => Some(late::commands()),
        net::NAME => Some(net::commands()),
        permissions::NAME => Some(permissions::commands()),
        python_host::NAME => Some(python_host::commands()),
        remote_session::NAME => Some(remote_session::commands()),
        surface::NAME => Some(surface::commands()),
        TRAY_MODULE_NAME => Some(tray::commands()),
        OVERLAY_MODULE_NAME => Some(overlay::commands()),
        WORKSPACE_MODULE_NAME => Some(workspace::commands()),
        CODE_EDITOR_MODULE_NAME => Some(code_editor::commands()),
        AI_MODULE_NAME => Some(ai::commands()),
        AI_LLAMA_CPP_MODULE_NAME => Some(llama_cpp::commands()),
        AI_OLLAMA_MODULE_NAME => Some(ollama::commands()),
        AI_OPENAI_MODULE_NAME => Some(openai::commands()),
        NOTIFICATION_MODULE_NAME => Some(notification::commands()),
        _ => None,
    }
}

fn registry_key_for_module_namespace(ns: &str) -> &str {
    match ns {
        shell::NAME => TERMINAL_MODULE_NAME,
        shell_motd::NAME => TERMINAL_MOTD_MODULE_NAME,
        other => other,
    }
}

fn command_token_prefix_for_registry(reg: &str) -> &str {
    match reg {
        TERMINAL_MODULE_NAME => shell::NAME,
        TERMINAL_MOTD_MODULE_NAME => shell_motd::NAME,
        other => other,
    }
}

fn command_permission_subject(command_module_prefix: &str) -> Option<PermissionSubject> {
    perm_cfg::registry_module_for_command_prefix(command_module_prefix)
        .map(|registry_name| PermissionSubject::module(registry_name.to_string()))
}

fn ensure_permissions(subject: &PermissionSubject, required: &[&str]) -> Result<(), String> {
    if required.is_empty() {
        return Ok(());
    }
    let cfg = PermissionsConfig::load_or_create().map_err(|e| e.to_string())?;
    for pid in required {
        if !cfg.effective_allowed(subject, pid) {
            return Err(format!(
                "Permission denied: {pid} (subject {}, global or per-module grant missing)",
                subject.storage_key()
            ));
        }
    }
    Ok(())
}

fn ensure_command_permissions(
    command_module_prefix: &str,
    required: &[&str],
    context: &ExecutionContext,
) -> Result<(), String> {
    if required.is_empty() {
        return Ok(());
    }
    let Some(subject) = command_permission_subject(command_module_prefix) else {
        return Err(format!(
            "Permission check: unknown command namespace '{command_module_prefix}'"
        ));
    };
    let cfg = PermissionsConfig::load_or_create().map_err(|e| e.to_string())?;
    for pid in required {
        if cfg.effective_allowed(&subject, pid) {
            continue;
        }
        if let Some(ext) = context.invoking_python_extension.as_deref() {
            let py = PermissionSubject::python(ext.to_string());
            if cfg.effective_allowed(&py, pid) {
                continue;
            }
        }
        return Err(format!(
            "Permission denied: {pid} (subject {}, global or per-module grant missing)",
            subject.storage_key()
        ));
    }
    Ok(())
}

fn module_enabled(namespace: &str) -> Result<bool, String> {
    let key = registry_key_for_module_namespace(namespace);
    let cfg = ModulesConfig::load_or_create().map_err(|err| err.to_string())?;
    cfg.modules
        .get(key)
        .copied()
        .ok_or_else(|| format!("Unknown module key (resolved: {key})"))
}

fn native_module_supported_at_runtime(registry_key: &str) -> bool {
    ModulesConfig::manifest_for(registry_key)
        .map(|m| supports_runtime_platform(m.supported_platforms))
        .unwrap_or(true)
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
        if !native_module_supported_at_runtime(module_name) {
            continue;
        }
        if let Some(commands) = module_commands(module_name) {
            let ns = command_token_prefix_for_registry(module_name);
            for command in commands {
                tokens.push(format!("{ns}.{}", command.name));
            }
        }
    }
    for (token, _) in python_registry::list_commands() {
        if python_registry::extension_command_supported_at_runtime(&token) {
            tokens.push(token);
        }
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
        .filter(|(module_name, _)| native_module_supported_at_runtime(module_name))
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
    if !native_module_supported_at_runtime(module_name) {
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

    if let Some(commands) = module_commands(module_name) {
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
                let remote_subj = PermissionSubject::module(REMOTE_SESSION_MODULE_NAME.to_string());
                ensure_permissions(&remote_subj, &["session.remote_route"])?;
                ensure_command_permissions(module_name, command.required_permissions, context)?;
                let response =
                    lan::execute_remote_command(target, token, args, context.net_timeout_ms)?;
                return Ok(Some(response));
            }
            return Err(format!("Unsupported net route: {route}"));
        }

        if !module_enabled(module_name)? {
            return Err(format!("Module {module_name} is disabled"));
        }

        let reg_key = registry_key_for_module_namespace(module_name);
        if !native_module_supported_at_runtime(reg_key) {
            return Err(format!(
                "Module {module_name} is not supported on this platform"
            ));
        }

        ensure_command_permissions(module_name, command.required_permissions, context)?;

        return Ok(Some((command.run)(args, context)));
    }

    match python_registry::try_dispatch(token, args)? {
        Some(s) => Ok(Some(s)),
        None => Ok(None),
    }
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
        if !native_module_supported_at_runtime(module_name) {
            continue;
        }
        if let Some(commands) = module_commands(module_name) {
            let ns = command_token_prefix_for_registry(module_name);
            for command in commands {
                lines.push(format!("- {ns}.{}: {}", command.name, command.description));
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
        if !native_module_supported_at_runtime(module_name) {
            continue;
        }
        if let Some(commands) = module_commands(module_name) {
            let ns = command_token_prefix_for_registry(module_name);
            for command in commands {
                entries.push((
                    format!("{ns}.{}", command.name),
                    command.description.to_string(),
                ));
            }
        }
    }
    for (token, description) in python_registry::list_commands() {
        if python_registry::extension_command_supported_at_runtime(token.as_str()) {
            entries.push((token, description));
        }
    }
    entries
}

pub fn load_all() {
    let _known_modules = [
        animation::NAME,
        cursor::NAME,
        lan::NAME,
        late::NAME,
        net::NAME,
        permissions::NAME,
        python_host::NAME,
        remote_session::NAME,
        shell::NAME,
        shell_motd::NAME,
        surface::NAME,
        tray::NAME,
        overlay::NAME,
    ];

    if let Err(err) = ModulesConfig::load_or_create() {
        eprintln!("Failed to load modules config: {err}");
    }
    if let Err(err) = PermissionsConfig::load_or_create() {
        eprintln!("Failed to load permissions config: {err}");
    }

    // Service binds port regardless; respects lan_enabled() per-request.
    if let Err(err) = lan::start_service() {
        eprintln!("Failed to start LAN service: {err}");
    }
}

pub fn shutdown_all() {
    lan::stop_service();
}
