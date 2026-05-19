use crate::config::workspace::{new_workspace_id, WorkspaceEntry, WorkspacesConfig};
use crate::config::ConfigFile;
use crate::modules::{ExecutionContext, ModuleCommand};

pub const NAME: &str = "workspace";

fn list_cmd(_args: &[&str], _ctx: &ExecutionContext) -> String {
    let Ok(cfg) = WorkspacesConfig::load_or_create() else {
        return "Error: could not load workspace.toml".to_string();
    };
    if cfg.workspaces.is_empty() {
        return "No workspaces registered.".to_string();
    }
    let mut out = Vec::new();
    for w in &cfg.workspaces {
        out.push(format!(
            "[{}] {} — {}  permissions: {}",
            w.id,
            w.label,
            w.path,
            if w.granted_permissions.is_empty() {
                "(none)".to_string()
            } else {
                w.granted_permissions.join(", ")
            }
        ));
    }
    out.join("\n")
}

fn add_cmd(args: &[&str], _ctx: &ExecutionContext) -> String {
    let (Some(path), Some(label)) = (args.first(), args.get(1)) else {
        return "Usage: workspace.add <path> <label>".to_string();
    };
    let Ok(mut cfg) = WorkspacesConfig::load_or_create() else {
        return "Error: could not load workspace.toml".to_string();
    };
    if cfg.workspaces.iter().any(|w| w.path == *path) {
        return format!("Workspace already exists for path: {path}");
    }
    let entry = WorkspaceEntry {
        id: new_workspace_id(),
        label: label.to_string(),
        path: path.to_string(),
        granted_permissions: vec!["workspace.read".to_string()],
    };
    let id = entry.id.clone();
    cfg.workspaces.push(entry);
    if let Err(e) = cfg.save() {
        return format!("Error saving: {e}");
    }
    format!("Workspace added: {id}")
}

fn remove_cmd(args: &[&str], _ctx: &ExecutionContext) -> String {
    let Some(id) = args.first() else {
        return "Usage: workspace.remove <id>".to_string();
    };
    let Ok(mut cfg) = WorkspacesConfig::load_or_create() else {
        return "Error: could not load workspace.toml".to_string();
    };
    let before = cfg.workspaces.len();
    cfg.workspaces.retain(|w| w.id != *id);
    if cfg.workspaces.len() == before {
        return format!("No workspace with id: {id}");
    }
    if let Err(e) = cfg.save() {
        return format!("Error saving: {e}");
    }
    format!("Workspace removed: {id}")
}

fn grant_cmd(args: &[&str], _ctx: &ExecutionContext) -> String {
    let (Some(id), Some(perm)) = (args.first(), args.get(1)) else {
        return "Usage: workspace.grant <id> <permission_id>".to_string();
    };
    let Ok(mut cfg) = WorkspacesConfig::load_or_create() else {
        return "Error: could not load workspace.toml".to_string();
    };
    let Some(ws) = cfg.workspaces.iter_mut().find(|w| w.id == *id) else {
        return format!("No workspace with id: {id}");
    };
    if ws.granted_permissions.iter().any(|p| p == *perm) {
        return format!("{perm} already granted in {id}");
    }
    ws.granted_permissions.push(perm.to_string());
    if let Err(e) = cfg.save() {
        return format!("Error saving: {e}");
    }
    format!("Granted {perm} in {id}")
}

fn revoke_cmd(args: &[&str], _ctx: &ExecutionContext) -> String {
    let (Some(id), Some(perm)) = (args.first(), args.get(1)) else {
        return "Usage: workspace.revoke <id> <permission_id>".to_string();
    };
    let Ok(mut cfg) = WorkspacesConfig::load_or_create() else {
        return "Error: could not load workspace.toml".to_string();
    };
    let Some(ws) = cfg.workspaces.iter_mut().find(|w| w.id == *id) else {
        return format!("No workspace with id: {id}");
    };
    ws.granted_permissions.retain(|p| p != *perm);
    if let Err(e) = cfg.save() {
        return format!("Error saving: {e}");
    }
    format!("Revoked {perm} in {id}")
}

fn check_cmd(args: &[&str], _ctx: &ExecutionContext) -> String {
    let (Some(path), Some(perm)) = (args.first(), args.get(1)) else {
        return "Usage: workspace.check <path> <permission_id>".to_string();
    };
    let granted = crate::config::workspace::any_workspace_grants(path, perm);
    if granted {
        "true".to_string()
    } else {
        "false".to_string()
    }
}

pub fn commands() -> &'static [ModuleCommand] {
    &[
        ModuleCommand {
            name: "list",
            description: "List all registered workspaces.",
            required_permissions: &[],
            run: list_cmd,
        },
        ModuleCommand {
            name: "add",
            description: "Register a workspace: workspace.add <path> <label>",
            required_permissions: &[],
            run: add_cmd,
        },
        ModuleCommand {
            name: "remove",
            description: "Remove a workspace by id: workspace.remove <id>",
            required_permissions: &[],
            run: remove_cmd,
        },
        ModuleCommand {
            name: "grant",
            description: "Grant a permission in a workspace: workspace.grant <id> <permission_id>",
            required_permissions: &[],
            run: grant_cmd,
        },
        ModuleCommand {
            name: "revoke",
            description: "Revoke a permission in a workspace: workspace.revoke <id> <permission_id>",
            required_permissions: &[],
            run: revoke_cmd,
        },
        ModuleCommand {
            name: "check",
            description: "Check if any workspace covers path with permission: workspace.check <path> <permission_id>",
            required_permissions: &[],
            run: check_cmd,
        },
    ]
}

#[derive(Default)]
pub struct WorkspaceExtension;

impl crate::extension::Extension for WorkspaceExtension {
    fn manifest(&self) -> crate::extension::OwnedModuleManifest {
        use crate::extension::OwnedWorkspacePermissionDef;
        crate::extension::OwnedModuleManifest {
            name: NAME.to_string(),
            glyph: "workspaces".to_string(),
            version: "0.1.0".to_string(),
            description:
                "Workspace directory registry with scoped file and execution permissions."
                    .to_string(),
            accent: String::new(),
            required_modules: Vec::new(),
            required_permissions: Vec::new(),
            workspace_permissions: vec![
                OwnedWorkspacePermissionDef {
                    id: "workspace.read".to_string(),
                    title: "File read".to_string(),
                    description: "Read files within this workspace.".to_string(),
                    default_granted: true,
                },
                OwnedWorkspacePermissionDef {
                    id: "workspace.write".to_string(),
                    title: "File write".to_string(),
                    description: "Create, modify, and delete files.".to_string(),
                    default_granted: false,
                },
                OwnedWorkspacePermissionDef {
                    id: "workspace.execute".to_string(),
                    title: "Command execution".to_string(),
                    description: "Run commands scoped to this workspace.".to_string(),
                    default_granted: false,
                },
            ],
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
            id: "global.workspaces",
            title: "Workspaces",
            description: "Register project directories and grant scoped file and execution permissions.",
            glyph: "workspaces",
            system_image: "folder",
            accent: "emerald",
            required_module: Some(NAME),
            layout_kind: PageLayoutKind::Standard,
            blocks_platform_goto: false,
        }];
        PAGES
    }
}

crate::register_extension!(WorkspaceExtension);
