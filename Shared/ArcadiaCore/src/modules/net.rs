use crate::extension::{Extension, OwnedModuleCommand, OwnedModuleManifest};
use crate::modules::{ExecutionContext, ModuleCommand};

pub const NAME: &str = "net";

fn help(_args: &[&str], context: &ExecutionContext) -> String {
    let timeout = context
        .net_timeout_ms
        .map(|ms| ms.to_string())
        .unwrap_or_else(|| "unset".to_string());
    match &context.net_as {
        Some(target) => format!(
            "Net context active: --net:as {target}, --net:timeout {timeout}"
        ),
        None => format!(
            "Net module ready. Use global flags: --net:as lan:<host/ip/alias>, --net:timeout <milliseconds> (current timeout: {timeout})"
        ),
    }
}

pub fn commands() -> &'static [ModuleCommand] {
    &[ModuleCommand {
        name: "help",
        description: "show net context and global net flag usage",
        required_permissions: &[],
        run: help,
    }]
}

#[derive(Default)]
pub struct NetExtension;

impl Extension for NetExtension {
    fn manifest(&self) -> OwnedModuleManifest {
        OwnedModuleManifest {
            name: NAME.to_string(),
            glyph: "network".to_string(),
            version: "1.0.0".to_string(),
            description: "Shared networking foundation for routed module commands."
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

crate::register_extension!(NetExtension);
