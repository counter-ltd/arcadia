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
}

crate::register_extension!(GotoExtension);
