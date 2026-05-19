use crate::extension::{Extension, OwnedModuleCommand, OwnedModuleManifest};
use crate::navigation;

use super::{ExecutionContext, ModuleCommand};

pub const NAME: &str = "goto";

pub fn commands() -> &'static [ModuleCommand] {
    &[ModuleCommand {
        name: "goto.page",
        description: "Resolve a navigation page by id: goto.page <page_id>",
        required_permissions: &[],
        run: run_page,
    }]
}

fn run_page(args: &[&str], _ctx: &ExecutionContext) -> String {
    let id = args.first().copied().unwrap_or("");
    match navigation::page_by_id(id) {
        Some(p) => format!("{} ({})", p.title, p.id),
        None => format!("unknown page: {id}"),
    }
}

/// Keyboard-driven navigation module, registered as a self-contained extension.
#[derive(Default)]
pub struct GotoExtension;

impl Extension for GotoExtension {
    fn manifest(&self) -> OwnedModuleManifest {
        OwnedModuleManifest {
            name: NAME.to_string(),
            glyph: "goto".to_string(),
            version: "0.1.0".to_string(),
            description:
                "Keyboard-driven navigation. Cmd+G opens the goto bar; goto.page <id> resolves pages from the CLI."
                    .to_string(),
            accent: String::new(),
            required_modules: Vec::new(),
            required_permissions: Vec::new(),
            workspace_permissions: Vec::new(),
            supported_platforms: Vec::new(),
            api_exports: Vec::new(),
        }
    }

    fn commands(&self) -> Vec<OwnedModuleCommand> {
        commands().iter().map(OwnedModuleCommand::from_static).collect()
    }

    fn shortcuts(&self) -> &'static [crate::shortcuts::ShortcutDefinition] {
        use crate::shortcuts::{
            ShortcutActionStatic, ShortcutDefinition, ShortcutScopeStatic, ShortcutTriggerStatic,
            ShortcutVisibility,
        };
        static SHORTCUTS: &[ShortcutDefinition] = &[ShortcutDefinition {
            id: "goto:open",
            label: "Open goto bar",
            owner: "goto",
            required_registry_module: Some(NAME),
            scope: ShortcutScopeStatic::ArcadiaWide,
            visibility: ShortcutVisibility::Both,
            priority: 90,
            consumes: true,
            bypass_text_focus: false,
            system_wide: false,
            triggers: &[ShortcutTriggerStatic::Chord {
                key: "g",
                control: false,
                alt: false,
                shift: false,
                platform: true,
                function: false,
            }],
            actions: &[ShortcutActionStatic::UiControl {
                control_id: "goto.open_page",
            }],
        }];
        SHORTCUTS
    }
}

crate::register_extension!(GotoExtension);
