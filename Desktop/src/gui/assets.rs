use std::borrow::Cow;

use arcadia_core::modules::python_registry;
use include_dir::{include_dir, Dir};
use openframe::{AssetSource, Result, SharedString};

static ASSETS: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/assets");

pub struct EmbeddedAssets;

impl AssetSource for EmbeddedAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path == "icons/app-icon.png" {
            return Ok(Some(Cow::Borrowed(include_bytes!(
                "../../../Resources/Icons/Production/Final-1-appicon.png"
            ))));
        }
        if let Some(ext_id) = path.strip_prefix("extension-icon/") {
            return match python_registry::resolve_extension_asset_path(ext_id, "icon.svg") {
                Ok(p) => match std::fs::read(&p) {
                    Ok(bytes) => Ok(Some(Cow::Owned(bytes))),
                    Err(_) => Ok(None),
                },
                Err(_) => Ok(None),
            };
        }
        Ok(ASSETS.get_file(path).map(|f| Cow::Borrowed(f.contents())))
    }

    fn list(&self, _path: &str) -> Result<Vec<SharedString>> {
        Ok(vec![])
    }
}
