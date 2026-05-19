pub const NAME: &str = "notification";

use crate::config::notifications::{NotificationDestination, NotificationsConfig};
use crate::config::permissions::{PermissionSubject, PermissionsConfig};
use crate::config::ConfigFile;
use crate::modules::{ExecutionContext, ModuleCommand};
use crate::platform;

pub fn commands() -> &'static [ModuleCommand] {
    &[
        ModuleCommand {
            name: "post",
            description: "Post a notification. Args: --source <id> --title <text> [--body <text>] [--id <notif_id>]",
            required_permissions: &[],
            run: run_post,
        },
        ModuleCommand {
            name: "list",
            description: "List all notifications as a JSON array.",
            required_permissions: &[],
            run: run_list,
        },
        ModuleCommand {
            name: "clear",
            description: "Delete all stored notifications.",
            required_permissions: &[],
            run: run_clear,
        },
        ModuleCommand {
            name: "mark_read",
            description: "Mark a notification as read. Args: <id>",
            required_permissions: &[],
            run: run_mark_read,
        },
        ModuleCommand {
            name: "mark_all_read",
            description: "Mark all notifications as read.",
            required_permissions: &[],
            run: run_mark_all_read,
        },
        ModuleCommand {
            name: "unread_count",
            description: "Return the number of unread notifications.",
            required_permissions: &[],
            run: run_unread_count,
        },
    ]
}

fn run_post(args: &[&str], ctx: &ExecutionContext) -> String {
    let mut source = String::new();
    let mut title = String::new();
    let mut body = String::new();
    let mut notif_id = String::new();

    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--source" if i + 1 < args.len() => {
                source = args[i + 1].to_string();
                i += 2;
            }
            "--title" if i + 1 < args.len() => {
                title = args[i + 1].to_string();
                i += 2;
            }
            "--body" if i + 1 < args.len() => {
                body = args[i + 1].to_string();
                i += 2;
            }
            "--id" if i + 1 < args.len() => {
                notif_id = args[i + 1].to_string();
                i += 2;
            }
            _ => {
                i += 1;
            }
        }
    }

    if source.is_empty() {
        return "Error: --source is required".to_string();
    }
    if title.is_empty() {
        return "Error: --title is required".to_string();
    }

    // Check notifications.send for the calling subject. Python extensions take precedence
    // over the --source field so they cannot spoof a different module's identity.
    let subject = if let Some(ext) = &ctx.invoking_python_extension {
        PermissionSubject::python(ext.as_str())
    } else {
        PermissionSubject::module(&source)
    };

    let mut notif_cfg = match NotificationsConfig::load_or_create() {
        Ok(c) => c,
        Err(e) => return format!("Error loading notifications config: {e}"),
    };

    let perms = match PermissionsConfig::load_or_create() {
        Ok(p) => p,
        Err(e) => return format!("Error loading permissions: {e}"),
    };

    if notif_cfg.trust_all_sources {
        if perms.subject_explicitly_denied(&subject, "notifications.send") {
            return format!(
                "Error: notifications.send explicitly denied for '{}'",
                subject.storage_key()
            );
        }
    } else if !perms.effective_allowed(&subject, "notifications.send") {
        return format!(
            "Error: notifications.send not granted for '{}'",
            subject.storage_key()
        );
    }

    if notif_id.is_empty() {
        notif_id = format!("{source}-{}", timestamp_now());
    }

    let dispatch_title = title.clone();
    let dispatch_body = body.clone();
    let destinations = notif_cfg.destinations.clone();

    notif_cfg.post(notif_id.clone(), title, body, source);
    match notif_cfg.save() {
        Ok(_) => {
            if destinations.contains(&NotificationDestination::System) {
                platform::send_system_notification(&dispatch_title, &dispatch_body);
            }
            format!("ok:{notif_id}")
        }
        Err(e) => format!("Error saving: {e}"),
    }
}

fn run_list(_args: &[&str], _ctx: &ExecutionContext) -> String {
    match NotificationsConfig::load_or_create() {
        Ok(cfg) => serde_json::to_string(&cfg.entries).unwrap_or_else(|e| format!("Error: {e}")),
        Err(e) => format!("Error: {e}"),
    }
}

fn run_clear(_args: &[&str], _ctx: &ExecutionContext) -> String {
    match NotificationsConfig::load_or_create() {
        Ok(mut cfg) => {
            cfg.entries.clear();
            match cfg.save() {
                Ok(_) => "ok".to_string(),
                Err(e) => format!("Error: {e}"),
            }
        }
        Err(e) => format!("Error: {e}"),
    }
}

fn run_mark_read(args: &[&str], _ctx: &ExecutionContext) -> String {
    let Some(id) = args.first() else {
        return "Error: notification id required".to_string();
    };
    match NotificationsConfig::load_or_create() {
        Ok(mut cfg) => {
            cfg.mark_read(id);
            match cfg.save() {
                Ok(_) => "ok".to_string(),
                Err(e) => format!("Error: {e}"),
            }
        }
        Err(e) => format!("Error: {e}"),
    }
}

fn run_mark_all_read(_args: &[&str], _ctx: &ExecutionContext) -> String {
    match NotificationsConfig::load_or_create() {
        Ok(mut cfg) => {
            cfg.mark_all_read();
            match cfg.save() {
                Ok(_) => "ok".to_string(),
                Err(e) => format!("Error: {e}"),
            }
        }
        Err(e) => format!("Error: {e}"),
    }
}

fn run_unread_count(_args: &[&str], _ctx: &ExecutionContext) -> String {
    match NotificationsConfig::load_or_create() {
        Ok(cfg) => cfg.unread_count().to_string(),
        Err(e) => format!("Error: {e}"),
    }
}

fn timestamp_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[derive(Default)]
pub struct NotificationExtension;

impl crate::extension::Extension for NotificationExtension {
    fn manifest(&self) -> crate::extension::OwnedModuleManifest {
        crate::extension::OwnedModuleManifest {
            name: NAME.to_string(),
            glyph: "notification".to_string(),
            version: "0.1.0".to_string(),
            description:
                "In-app notification centre. Modules and extensions can post alerts; each source requires an explicit notifications.send grant."
                    .to_string(),
            accent: String::new(),
            required_modules: Vec::new(),
            required_permissions: Vec::new(),
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
        static PAGES: &[NavigationPageDefinition] = &[
            NavigationPageDefinition {
                id: "notification.main",
                title: "Notifications",
                description: "In-app notification centre. View and dismiss alerts from modules and extensions.",
                glyph: "notification",
                system_image: "bell",
                accent: "amber",
                required_module: Some(NAME),
                layout_kind: PageLayoutKind::Standard,
                blocks_platform_goto: false,
            },
            NavigationPageDefinition {
                id: "notification.settings",
                title: "Notifications",
                description: "Configure notification storage, per-source send permissions, and display preferences.",
                glyph: "notification",
                system_image: "bell.badge",
                accent: "amber",
                required_module: Some(NAME),
                layout_kind: PageLayoutKind::Standard,
                blocks_platform_goto: false,
            },
        ];
        PAGES
    }

    fn permissions(&self) -> &'static [crate::config::permissions::PermissionDefinition] {
        use crate::config::permissions::PermissionDefinition;
        static PERMS: &[PermissionDefinition] = &[
            PermissionDefinition {
                id: "notifications.receive",
                title: "Receive notifications",
                description: "Allow the notification module to store and display in-app notifications.",
                default_global: true,
                system_grant: None,
            },
            PermissionDefinition {
                id: "notifications.send",
                title: "Send notifications",
                description: "Allow a module or extension to post notifications via notification.post. Grant per-source in the Notifications settings.",
                default_global: false,
                system_grant: None,
            },
        ];
        PERMS
    }
}

crate::register_extension!(NotificationExtension);
