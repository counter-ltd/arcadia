use std::borrow::Cow;

use arcadia_core::modules::python_registry;
use openframe::{AssetSource, Result, SharedString};

pub struct EmbeddedAssets;

impl AssetSource for EmbeddedAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        match path {
            "icons/terminal.svg" => Ok(Some(Cow::Borrowed(include_bytes!(
                "../../assets/icons/terminal.svg"
            )))),
            "icons/home.svg" => Ok(Some(Cow::Borrowed(include_bytes!(
                "../../assets/icons/home.svg"
            )))),
            "icons/logs.svg" => Ok(Some(Cow::Borrowed(include_bytes!(
                "../../assets/icons/logs.svg"
            )))),
            "icons/log-out.svg" => Ok(Some(Cow::Borrowed(include_bytes!(
                "../../assets/icons/log-out.svg"
            )))),
            "icons/x.svg" => Ok(Some(Cow::Borrowed(include_bytes!(
                "../../assets/icons/x.svg"
            )))),
            "icons/settings.svg" => Ok(Some(Cow::Borrowed(include_bytes!(
                "../../assets/icons/settings.svg"
            )))),
            "icons/modules.svg" => Ok(Some(Cow::Borrowed(include_bytes!(
                "../../assets/icons/modules.svg"
            )))),
            "icons/nodes.svg" => Ok(Some(Cow::Borrowed(include_bytes!(
                "../../assets/icons/nodes.svg"
            )))),
            "icons/tools.svg" => Ok(Some(Cow::Borrowed(include_bytes!(
                "../../assets/icons/tools.svg"
            )))),
            "icons/services.svg" => Ok(Some(Cow::Borrowed(include_bytes!(
                "../../assets/icons/services.svg"
            )))),
            "icons/network.svg" => Ok(Some(Cow::Borrowed(include_bytes!(
                "../../assets/icons/network.svg"
            )))),
            "icons/chat.svg" => Ok(Some(Cow::Borrowed(include_bytes!(
                "../../assets/icons/chat.svg"
            )))),
            "icons/music.svg" => Ok(Some(Cow::Borrowed(include_bytes!(
                "../../assets/icons/music.svg"
            )))),
            "icons/flask.svg" => Ok(Some(Cow::Borrowed(include_bytes!(
                "../../assets/icons/flask.svg"
            )))),
            "icons/coffee.svg" => Ok(Some(Cow::Borrowed(include_bytes!(
                "../../assets/icons/coffee.svg"
            )))),
            "icons/appearance.svg" => Ok(Some(Cow::Borrowed(include_bytes!(
                "../../assets/icons/appearance.svg"
            )))),
            "icons/extensions.svg" => Ok(Some(Cow::Borrowed(include_bytes!(
                "../../assets/icons/extensions.svg"
            )))),
            "icons/python.svg" => Ok(Some(Cow::Borrowed(include_bytes!(
                "../../assets/icons/python.svg"
            )))),
            "icons/permissions.svg" => Ok(Some(Cow::Borrowed(include_bytes!(
                "../../assets/icons/permissions.svg"
            )))),
            "icons/folder.svg" => Ok(Some(Cow::Borrowed(include_bytes!(
                "../../assets/icons/folder.svg"
            )))),
            "icons/folder-open.svg" => Ok(Some(Cow::Borrowed(include_bytes!(
                "../../assets/icons/folder-open.svg"
            )))),
            "icons/file.svg" => Ok(Some(Cow::Borrowed(include_bytes!(
                "../../assets/icons/file.svg"
            )))),
            "icons/file-code.svg" => Ok(Some(Cow::Borrowed(include_bytes!(
                "../../assets/icons/file-code.svg"
            )))),
            "icons/file-text.svg" => Ok(Some(Cow::Borrowed(include_bytes!(
                "../../assets/icons/file-text.svg"
            )))),
            "icons/chevron-right.svg" => Ok(Some(Cow::Borrowed(include_bytes!(
                "../../assets/icons/chevron-right.svg"
            )))),
            "icons/chevron-down.svg" => Ok(Some(Cow::Borrowed(include_bytes!(
                "../../assets/icons/chevron-down.svg"
            )))),
            "icons/message.svg" => Ok(Some(Cow::Borrowed(include_bytes!(
                "../../assets/icons/message.svg"
            )))),
            "icons/app-icon.png" => Ok(Some(Cow::Borrowed(include_bytes!(
                "../../../Resources/Icons/Production/Final-1-appicon.png"
            )))),
            "icons/app-icon-tui.svg" => Ok(Some(Cow::Borrowed(include_bytes!(
                "../../assets/icons/app-icon-tui.svg"
            )))),
            path if path.starts_with("extension-icon/") => {
                let ext_id = &path["extension-icon/".len()..];
                match python_registry::resolve_extension_asset_path(ext_id, "icon.svg") {
                    Ok(p) => match std::fs::read(&p) {
                        Ok(bytes) => Ok(Some(Cow::Owned(bytes))),
                        Err(_) => Ok(None),
                    },
                    Err(_) => Ok(None),
                }
            }
            _ => Ok(None),
        }
    }

    fn list(&self, _path: &str) -> Result<Vec<SharedString>> {
        Ok(vec![])
    }
}
