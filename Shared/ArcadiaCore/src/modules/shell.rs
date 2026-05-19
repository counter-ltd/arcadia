use crate::capabilities::SHELL_INTERNAL_NO_RUNTIME;
use crate::modules::{ExecutionContext, ModuleCommand};
use std::sync::OnceLock;

pub const NAME: &str = "shell";
type InternalExecutor = fn(&str) -> String;
static INTERNAL_EXECUTOR: OnceLock<InternalExecutor> = OnceLock::new();

pub fn set_internal_executor(executor: InternalExecutor) {
    let _ = INTERNAL_EXECUTOR.set(executor);
}

fn strip_ansi_sequences(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    while i < bytes.len() {
        // Drop ANSI CSI sequences: ESC [ ... final-byte.
        if bytes[i] == 0x1B && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            i += 2;
            while i < bytes.len() {
                let b = bytes[i];
                if (0x40..=0x7E).contains(&b) {
                    i += 1;
                    break;
                }
                i += 1;
            }
            continue;
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

fn execute(args: &[&str], context: &ExecutionContext) -> String {
    if args.is_empty() {
        return "Usage: shell.execute <command...>".to_string();
    }
    let _ = context;

    #[cfg(target_os = "ios")]
    {
        return crate::capabilities::SHELL_EXECUTE_UNAVAILABLE_IOS_LOCAL.to_string();
    }

    #[cfg(not(target_os = "ios"))]
    {
        use std::process::Command;

        let command_line = args.join(" ");
        let output = {
            #[cfg(target_os = "windows")]
            {
                Command::new("cmd").args(["/C", &command_line]).output()
            }
            #[cfg(not(target_os = "windows"))]
            {
                Command::new("sh").args(["-c", &command_line]).output()
            }
        };

        match output {
            Ok(output) => {
                let stdout = strip_ansi_sequences(&String::from_utf8_lossy(&output.stdout));
                let stderr = strip_ansi_sequences(&String::from_utf8_lossy(&output.stderr));
                let mut lines = Vec::new();
                if !stdout.is_empty() {
                    lines.push(stdout);
                }
                if !stderr.is_empty() {
                    lines.push(stderr);
                }
                if !output.status.success() {
                    if let Some(code) = output.status.code() {
                        lines.push(format!("(exit code: {code})"));
                    } else {
                        lines.push("(process terminated by signal)".to_string());
                    }
                }
                lines.join("\n")
            }
            Err(err) => format!("Failed to execute shell command: {err}"),
        }
    }
}

fn internal(args: &[&str], context: &ExecutionContext) -> String {
    if args.is_empty() {
        return "Usage: shell.internal <command...>".to_string();
    }
    let _ = context;
    let command_line = args.join(" ");
    match INTERNAL_EXECUTOR.get() {
        Some(executor) => executor(&command_line),
        None => SHELL_INTERNAL_NO_RUNTIME.to_string(),
    }
}

pub fn commands() -> &'static [ModuleCommand] {
    &[
        ModuleCommand {
            name: "execute",
            description: "execute shell command(s): shell.execute <command...>",
            required_permissions: &["shell.run"],
            run: execute,
        },
        ModuleCommand {
            name: "internal",
            description: "execute internal CLI command(s): shell.internal <command...>",
            required_permissions: &["shell.bridge"],
            run: internal,
        },
    ]
}

/// Interactive terminal module. Registry name is `terminal`; its commands
/// dispatch under the `shell.` prefix.
#[derive(Default)]
pub struct TerminalExtension;

impl crate::extension::Extension for TerminalExtension {
    fn manifest(&self) -> crate::extension::OwnedModuleManifest {
        crate::extension::OwnedModuleManifest {
            name: crate::config::modules::TERMINAL_MODULE_NAME.to_string(),
            glyph: "terminal".to_string(),
            version: "1.0.0".to_string(),
            description: "Interactive terminal command execution for Arcadia surfaces."
                .to_string(),
            accent: String::new(),
            required_modules: Vec::new(),
            required_permissions: vec!["shell.run".to_string(), "shell.bridge".to_string()],
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
            id: "utility.shell",
            title: "Terminal",
            description: "Run and manage terminal commands.",
            glyph: "terminal",
            system_image: "terminal",
            accent: "emerald",
            required_module: Some(crate::config::modules::TERMINAL_MODULE_NAME),
            layout_kind: PageLayoutKind::FullHeight,
            blocks_platform_goto: false,
        }];
        PAGES
    }

    fn permissions(&self) -> &'static [crate::config::permissions::PermissionDefinition] {
        use crate::config::permissions::PermissionDefinition;
        static PERMS: &[PermissionDefinition] = &[
            PermissionDefinition {
                id: "shell.run",
                title: "Shell commands",
                description: "Spawn local subprocesses (shell.execute).",
                default_global: false,
                system_grant: None,
            },
            PermissionDefinition {
                id: "shell.bridge",
                title: "Shell bridge",
                description: "Host/runtime bridge (shell.internal).",
                default_global: false,
                system_grant: None,
            },
        ];
        PERMS
    }

    fn shortcuts(&self) -> &'static [crate::shortcuts::ShortcutDefinition] {
        use crate::shortcuts::{
            ShortcutActionStatic, ShortcutDefinition, ShortcutScopeStatic, ShortcutTriggerStatic,
            ShortcutVisibility,
        };
        static SHORTCUTS: &[ShortcutDefinition] = &[ShortcutDefinition {
            id: "terminal:toggle-shell-mode",
            label: "Toggle shell execution mode (generic vs internal)",
            owner: "terminal",
            required_registry_module: Some(crate::config::modules::TERMINAL_MODULE_NAME),
            scope: ShortcutScopeStatic::Pages(&["utility.shell"]),
            visibility: ShortcutVisibility::Both,
            priority: 90,
            consumes: true,
            bypass_text_focus: false,
            system_wide: false,
            triggers: &[ShortcutTriggerStatic::Chord {
                key: "tab",
                control: false,
                alt: false,
                shift: true,
                platform: false,
                function: false,
            }],
            actions: &[ShortcutActionStatic::UiControl {
                control_id: "terminal.toggle_shell_mode",
            }],
        }];
        SHORTCUTS
    }
}

crate::register_extension!(TerminalExtension);
