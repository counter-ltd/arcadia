//! LAN routing gate (`execute_command` with `net_as: lan:…`). No dedicated mirror verbs — use [`crate::modules::surface`].

use crate::config::modules::{LAN_MODULE_NAME, NET_MODULE_NAME};
use crate::extension::{Extension, OwnedModuleManifest};
use crate::modules::ModuleCommand;

pub const NAME: &str = "remote-session";

pub fn commands() -> &'static [ModuleCommand] {
    &[]
}

#[derive(Default)]
pub struct RemoteSessionExtension;

impl Extension for RemoteSessionExtension {
    fn manifest(&self) -> OwnedModuleManifest {
        OwnedModuleManifest {
            name: NAME.to_string(),
            glyph: "network".to_string(),
            version: "0.1.0".to_string(),
            description:
                "Permission to route execute_command over LAN (net_as: lan:…); transcript/mirror are automatic on hosts."
                    .to_string(),
            accent: String::new(),
            required_modules: vec![NET_MODULE_NAME.to_string(), LAN_MODULE_NAME.to_string()],
            required_permissions: vec!["session.remote_route".to_string()],
            workspace_permissions: Vec::new(),
            supported_platforms: Vec::new(),
            api_exports: Vec::new(),
        }
    }
}

crate::register_extension!(RemoteSessionExtension);
