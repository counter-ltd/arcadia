use std::borrow::Cow;

use arcadia_core::modules::python_registry;
use include_dir::{include_dir, Dir};
use openframe::{AssetSource, Result, SharedString};

static ASSETS: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/assets");

/// Family name of the bundled monospace font, as recorded in the .ttf metadata.
/// Use this in `.font_family(...)` for any cell-grid rendering (code editor, TUI)
/// so glyph advances are uniform regardless of OS font fallbacks.
pub const MONO_FONT_FAMILY: &str = "JetBrains Mono";

/// Register the bundled JetBrains Mono faces with the text system. Must run once
/// at startup before any window renders text.
pub fn register_bundled_fonts(text_system: &openframe::TextSystem) {
    let fonts: Vec<Cow<'static, [u8]>> = vec![
        Cow::Borrowed(include_bytes!("../../assets/fonts/JetBrainsMono-Regular.ttf").as_slice()),
        Cow::Borrowed(include_bytes!("../../assets/fonts/JetBrainsMono-Bold.ttf").as_slice()),
        Cow::Borrowed(include_bytes!("../../assets/fonts/JetBrainsMono-Italic.ttf").as_slice()),
        Cow::Borrowed(include_bytes!("../../assets/fonts/JetBrainsMono-BoldItalic.ttf").as_slice()),
    ];
    if let Err(e) = text_system.add_fonts(fonts) {
        eprintln!("arcadia: failed to register bundled monospace font: {e}");
    }
}

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
        if let Some(module_id) = path.strip_prefix("module-icon/") {
            use arcadia_core::modules::wasm_registry;
            return match wasm_registry::resolve_module_asset_path(module_id, "icon.svg") {
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
