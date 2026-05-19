//! WASM module host — the built-in `Extension` providing the `wasm-host` module.
//!
//! Mirrors [`python_host`](crate::modules::python_host): its own settings page and
//! `wasm-host.*` management commands are static; the runtime modules it loads register
//! through [`wasm_registry`](crate::modules::wasm_registry). The `wasmi` engine itself lives
//! in the separate `arcadia-wasm` crate.

use super::wasm_registry;
use crate::config::modules::ModulesConfig;
use crate::config::ConfigFile;
use crate::modules::{ExecutionContext, ModuleCommand};

pub const NAME: &str = "wasm-host";

fn persist_module_state(name: &str, enabled: bool) -> Result<(), String> {
    let mut cfg = ModulesConfig::load_or_create().map_err(|e| e.to_string())?;
    cfg.set_runtime_module_enabled(name, enabled);
    cfg.save().map_err(|e| e.to_string())
}

fn list(_args: &[&str], _context: &ExecutionContext) -> String {
    let modules = wasm_registry::list_modules();
    let commands = wasm_registry::list_commands();

    if modules.is_empty() && commands.is_empty() {
        return "No WASM modules found. Place .wasm files in ~/Arcadia/Modules/.".to_string();
    }

    let mut lines = Vec::new();
    if !modules.is_empty() {
        lines.push("WASM modules:".to_string());
        for (name, version, description, enabled, _perms, _plats, _tags, loaded) in &modules {
            let state = match (*enabled, *loaded) {
                (true, true) => "enabled",
                (true, false) => "enabled — not loaded",
                (false, _) => "disabled",
            };
            lines.push(format!("  {name} v{version} [{state}] — {description}"));
        }
    }
    if !commands.is_empty() {
        if !lines.is_empty() {
            lines.push(String::new());
        }
        lines.push("WASM commands:".to_string());
        let mut sorted = commands.clone();
        sorted.sort_by(|a, b| a.0.cmp(&b.0));
        for (token, description) in &sorted {
            lines.push(format!("  {token} — {description}"));
        }
    }
    lines.join("\n")
}

fn module_enable(args: &[&str], _context: &ExecutionContext) -> String {
    let Some(name) = args.first() else {
        return "Usage: wasm-host.module-enable <module-name>".to_string();
    };
    if !wasm_registry::module_supported_at_runtime(name) {
        return format!("WASM module '{name}' is not supported on this platform.");
    }
    if let Err(e) = persist_module_state(name, true) {
        return format!("Failed to persist enable state for '{name}': {e}");
    }
    wasm_registry::set_module_enabled(name, true);

    // First enable (or re-enable after a restart): the module has not been instantiated yet,
    // so drive the host loader now that the user has opted in.
    if !wasm_registry::module_body_loaded(name) {
        let Some(path) = wasm_registry::module_path(name) else {
            return format!(
                "WASM module '{name}' is enabled in config but no .wasm path is recorded — \
                 run wasm-host.reload to rediscover it."
            );
        };
        match wasm_registry::load_one(name.to_string(), path) {
            Ok(_) => {}
            Err(e) => return format!("WASM module '{name}' enabled, but failed to load: {e}"),
        }
    }
    format!("WASM module '{name}' enabled.")
}

fn module_disable(args: &[&str], _context: &ExecutionContext) -> String {
    let Some(name) = args.first() else {
        return "Usage: wasm-host.module-disable <module-name>".to_string();
    };
    if let Err(e) = persist_module_state(name, false) {
        return format!("Failed to persist disable state for '{name}': {e}");
    }
    wasm_registry::set_module_enabled(name, false);
    wasm_registry::unregister_module_contributions(name);
    format!("WASM module '{name}' disabled.")
}

fn reload(_args: &[&str], _context: &ExecutionContext) -> String {
    match wasm_registry::reload() {
        Ok(()) => {
            let modules = wasm_registry::list_modules();
            let commands = wasm_registry::list_commands();
            format!(
                "Reloaded. {} WASM module(s), {} command(s).",
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
            description: "List discovered WASM modules and their commands.",
            required_permissions: &[],
            run: list,
        },
        ModuleCommand {
            name: "reload",
            description: "Rediscover WASM modules from ~/Arcadia/Modules/.",
            required_permissions: &["wasm.host"],
            run: reload,
        },
        ModuleCommand {
            name: "module-enable",
            description: "Enable a WASM module by name: wasm-host.module-enable <name>",
            required_permissions: &["wasm.module_toggle"],
            run: module_enable,
        },
        ModuleCommand {
            name: "module-disable",
            description: "Disable a WASM module by name: wasm-host.module-disable <name>",
            required_permissions: &["wasm.module_toggle"],
            run: module_disable,
        },
    ]
}

#[derive(Default)]
pub struct WasmHostExtension;

impl crate::extension::Extension for WasmHostExtension {
    fn manifest(&self) -> crate::extension::OwnedModuleManifest {
        crate::extension::OwnedModuleManifest {
            name: NAME.to_string(),
            glyph: "modules".to_string(),
            version: "0.1.0".to_string(),
            description:
                "WASM module loader. Scans ~/Arcadia/Modules/ for .wasm files and registers their commands."
                    .to_string(),
            accent: String::new(),
            required_modules: Vec::new(),
            required_permissions: vec![
                "wasm.host".to_string(),
                "wasm.module_toggle".to_string(),
            ],
            workspace_permissions: Vec::new(),
            supported_platforms: Vec::new(),
            api_exports: Vec::new(),
        }
    }

    fn commands(&self) -> &'static [crate::modules::ModuleCommand] {
        commands()
    }

    fn nav_pages(&self) -> &'static [crate::navigation::NavigationPageDefinition] {
        use crate::navigation::{NavigationPageDefinition, PageLayoutKind};
        static PAGES: &[NavigationPageDefinition] = &[NavigationPageDefinition {
            id: "wasm-modules.settings",
            title: "Modules",
            description: "Enable or disable WASM modules loaded from ~/Arcadia/Modules/.",
            glyph: "modules",
            system_image: "puzzlepiece.extension.fill",
            accent: "indigo",
            required_module: Some(NAME),
            layout_kind: PageLayoutKind::Standard,
            blocks_platform_goto: false,
        }];
        PAGES
    }

    fn permissions(&self) -> &'static [crate::config::permissions::PermissionDefinition] {
        use crate::config::permissions::PermissionDefinition;
        static PERMS: &[PermissionDefinition] = &[
            PermissionDefinition {
                id: "wasm.host",
                title: "WASM host",
                description: "Discover and reload WASM modules from disk (wasm-host.reload).",
                default_global: false,
                system_grant: None,
            },
            PermissionDefinition {
                id: "wasm.module_toggle",
                title: "WASM module enable",
                description: "Enable or disable WASM modules.",
                default_global: false,
                system_grant: None,
            },
        ];
        PERMS
    }
}

crate::register_extension!(WasmHostExtension);
