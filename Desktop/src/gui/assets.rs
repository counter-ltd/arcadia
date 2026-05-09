use std::borrow::Cow;

use gpui::{AssetSource, Result, SharedString};

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
            "icons/app-icon.png" => Ok(Some(Cow::Borrowed(include_bytes!(
                "../../../Resources/Icons/Production/Final-1-appicon.png"
            )))),
            _ => Ok(None),
        }
    }

    fn list(&self, _path: &str) -> Result<Vec<SharedString>> {
        Ok(vec![])
    }
}
