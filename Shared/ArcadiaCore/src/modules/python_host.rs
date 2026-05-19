use super::python_registry;
use crate::config::modules::{ModulesConfig, OVERLAY_MODULE_NAME};
use crate::config::permissions::{PermissionSubject, PermissionsConfig};
use crate::config::ConfigFile;
use crate::modules::{animation, tray, ExecutionContext, ModuleCommand};

pub const NAME: &str = "python-host";

/// When an extension body declares `overlay.hud`, ensure the native `overlay` module is enabled
/// and `module:overlay` has the same permission — otherwise `overlay.show` never runs (defaults
/// keep `overlay` off in `modules.toml`).
pub fn ensure_native_companions_for_loaded_extension(extension_id: &str) {
    let declares_overlay = python_registry::list_modules()
        .into_iter()
        .find(|(n, _, _, _, _, _, _)| n == extension_id)
        .map(|(_, _, _, _, perms, _, _)| perms.iter().any(|p| p == "overlay.hud"))
        .unwrap_or(false);
    if !declares_overlay {
        return;
    }
    let Ok(mut modules_cfg) = ModulesConfig::load_or_create() else {
        return;
    };
    if modules_cfg
        .modules
        .get(OVERLAY_MODULE_NAME)
        .copied()
        .unwrap_or(false)
    {
        return;
    }
    let Ok(mut perms) = PermissionsConfig::load_or_create() else {
        return;
    };
    let subj = PermissionSubject::module(OVERLAY_MODULE_NAME.to_string());
    if let Err(e) = perms.ensure_effective_grants(&subj, &[String::from("overlay.hud")]) {
        eprintln!("python-host: overlay companion: permissions: {e}");
        return;
    }
    if let Err(e) = perms.save() {
        eprintln!("python-host: overlay companion: save permissions: {e}");
        return;
    }
    if let Err(e) = modules_cfg.enable_with_requirements(OVERLAY_MODULE_NAME) {
        eprintln!("python-host: overlay companion: enable overlay module: {e}");
        return;
    }
    if let Err(e) = modules_cfg.save() {
        eprintln!("python-host: overlay companion: save modules: {e}");
    }
}

fn persist_extension_state(name: &str, enabled: bool) -> Result<(), String> {
    let mut cfg = ModulesConfig::load_or_create().map_err(|e| e.to_string())?;
    cfg.set_python_extension_enabled(name, enabled);
    cfg.save().map_err(|e| e.to_string())
}

fn list(args: &[&str], _context: &ExecutionContext) -> String {
    let _ = args;
    let modules = python_registry::list_modules();
    let commands = python_registry::list_commands();

    if modules.is_empty() && commands.is_empty() {
        return "No Python extensions loaded. Place .py files in ~/Arcadia/Extensions/."
            .to_string();
    }

    let mut lines = Vec::new();

    if !modules.is_empty() {
        lines.push("Python modules:".to_string());
        for (name, version, description, enabled, _perms, _plats, _tags) in &modules {
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
    if !python_registry::extension_supported_at_runtime(name) {
        return format!(
            "Extension '{name}' is not supported on this platform (Platform Not Supported)."
        );
    }
    if let Err(e) = persist_extension_state(name, true) {
        return format!("Failed to persist enable state for '{name}': {e}");
    }
    python_registry::set_extension_enabled(name, true);

    // First-enable / re-enable after a restart: the body has never executed (because the
    // loader skips disabled extensions at startup), so the registry entry is still a stub.
    // Drive the host loader to actually run `main.py` now that the user has opted in.
    let needs_body_load = !python_registry::extension_body_loaded(name);

    if needs_body_load {
        let Some(path) = python_registry::extension_path(name) else {
            return format!(
                "Extension '{name}' is enabled in config but no source path is recorded — \
                 reload the python-host to discover it."
            );
        };
        match python_registry::load_one(name.to_string(), path) {
            Ok(canonical) if canonical != *name => {
                // Body declared a different `register_module(name=…)` than the folder-derived
                // stub id — persist the rename so future toggles work against the canonical
                // name instead of leaving an orphan key under the old id.
                if let Ok(mut cfg) = ModulesConfig::load_or_create() {
                    cfg.extension_state.remove(*name);
                    cfg.set_python_extension_enabled(&canonical, true);
                    let _ = cfg.save();
                }
                return format!(
                    "Extension '{canonical}' enabled (declared name differs from folder \
                     id '{name}' — collapsed onto the canonical id)."
                );
            }
            Ok(_) => {}
            Err(e) => {
                return format!("Extension '{name}' enabled, but failed to load: {e}");
            }
        }
    }

    format!("Extension '{name}' enabled.")
}

fn extension_disable(args: &[&str], _context: &ExecutionContext) -> String {
    let Some(name) = args.first() else {
        return "Usage: python-host.extension-disable <extension-name>".to_string();
    };
    if let Err(e) = persist_extension_state(name, false) {
        return format!("Failed to persist disable state for '{name}': {e}");
    }
    python_registry::set_extension_enabled(name, false);

    // Tear down handlers, tokens, styles, shortcuts, tray icons, and animation tweens owned
    // by this extension so disabling has an immediate visible effect.
    python_registry::unregister_extension_contributions(name);
    let _ = tray::remove_items_for_owner(&format!("python:{name}"));
    animation::cancel_all_for_extension(name);

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
            required_permissions: &[],
            run: list,
        },
        ModuleCommand {
            name: "reload",
            description: "Reload all Python extensions from ~/Arcadia/Extensions/.",
            required_permissions: &["python.host"],
            run: reload,
        },
        ModuleCommand {
            name: "extension-enable",
            description: "Enable a Python extension by name: python-host.extension-enable <name>",
            required_permissions: &["python.extension_toggle"],
            run: extension_enable,
        },
        ModuleCommand {
            name: "extension-disable",
            description: "Disable a Python extension by name: python-host.extension-disable <name>",
            required_permissions: &["python.extension_toggle"],
            run: extension_disable,
        },
    ]
}

#[derive(Default)]
pub struct PythonHostExtension;

impl crate::extension::Extension for PythonHostExtension {
    fn manifest(&self) -> crate::extension::OwnedModuleManifest {
        crate::extension::OwnedModuleManifest {
            name: NAME.to_string(),
            glyph: "python".to_string(),
            version: "0.1.0".to_string(),
            description:
                "Python extension loader. Scans ~/Arcadia/Extensions/ for .py files and registers their commands."
                    .to_string(),
            accent: String::new(),
            required_modules: Vec::new(),
            required_permissions: vec![
                "python.host".to_string(),
                "python.extension_toggle".to_string(),
            ],
            workspace_permissions: Vec::new(),
            supported_platforms: Vec::new(),
            api_exports: Vec::new(),
        }
    }

    fn commands(&self) -> Vec<crate::extension::OwnedModuleCommand> {
        commands()
            .iter()
            .map(crate::extension::OwnedModuleCommand::from_static)
            .collect()
    }

    fn nav_pages(&self) -> &'static [crate::navigation::NavigationPageDefinition] {
        use crate::navigation::{NavigationPageDefinition, PageLayoutKind};
        static PAGES: &[NavigationPageDefinition] = &[NavigationPageDefinition {
            id: "extensions.settings",
            title: "Extensions",
            description: "Enable or disable Python extensions loaded from ~/Arcadia/Extensions/.",
            glyph: "extensions",
            system_image: "flask.fill",
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
                id: "python.host",
                title: "Python host",
                description: "Load or reload extensions from disk (python-host.reload).",
                default_global: false,
                system_grant: None,
            },
            PermissionDefinition {
                id: "python.extension_toggle",
                title: "Extension enable",
                description: "Enable or disable Python extensions.",
                default_global: false,
                system_grant: None,
            },
        ];
        PERMS
    }
}

crate::register_extension!(PythonHostExtension);
