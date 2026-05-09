use crate::modules::{ExecutionContext, ModuleCommand};
use super::python_registry;

pub const NAME: &str = "python-host";

fn list(args: &[&str], _context: &ExecutionContext) -> String {
    let _ = args;
    let modules = python_registry::list_modules();
    let commands = python_registry::list_commands();

    if modules.is_empty() && commands.is_empty() {
        return "No Python extensions loaded. Place .py files in ~/Arcadia/Extensions/.".to_string();
    }

    let mut lines = Vec::new();

    if !modules.is_empty() {
        lines.push("Python modules:".to_string());
        for (name, version, description, enabled) in &modules {
            let state = if *enabled { "enabled" } else { "disabled" };
            lines.push(format!("  {name} v{version} [{state}] — {description}"));
        }
    }

    if !commands.is_empty() {
        if !lines.is_empty() {
            lines.push(String::new());
        }
        lines.push("Python commands:".to_string());
        let mut sorted = commands.clone();
        sorted.sort_by(|a, b| a.0.cmp(&b.0));
        for (token, description) in &sorted {
            lines.push(format!("  {token} — {description}"));
        }
    }

    lines.join("\n")
}

fn extension_enable(args: &[&str], _context: &ExecutionContext) -> String {
    let Some(name) = args.first() else {
        return "Usage: python-host.extension-enable <extension-name>".to_string();
    };
    python_registry::set_extension_enabled(name, true);
    format!("Extension '{name}' enabled.")
}

fn extension_disable(args: &[&str], _context: &ExecutionContext) -> String {
    let Some(name) = args.first() else {
        return "Usage: python-host.extension-disable <extension-name>".to_string();
    };
    python_registry::set_extension_enabled(name, false);
    format!("Extension '{name}' disabled.")
}

fn reload(args: &[&str], _context: &ExecutionContext) -> String {
    let _ = args;
    match python_registry::reload() {
        Ok(()) => {
            let modules = python_registry::list_modules();
            let commands = python_registry::list_commands();
            format!(
                "Reloaded. {} Python module(s), {} command(s).",
                modules.len(),
                commands.len()
            )
        }
        Err(e) => format!("Reload failed: {e}"),
    }
}

pub fn commands() -> &'static [ModuleCommand] {
    &[
        ModuleCommand {
            name: "list",
            description: "List loaded Python extensions and their commands.",
            run: list,
        },
        ModuleCommand {
            name: "reload",
            description: "Reload all Python extensions from ~/Arcadia/Extensions/.",
            run: reload,
        },
        ModuleCommand {
            name: "extension-enable",
            description: "Enable a Python extension by name: python-host.extension-enable <name>",
            run: extension_enable,
        },
        ModuleCommand {
            name: "extension-disable",
            description: "Disable a Python extension by name: python-host.extension-disable <name>",
            run: extension_disable,
        },
    ]
}
