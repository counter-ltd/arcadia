use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

pub struct PythonModuleInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub enabled: bool,
}

/// Glyph rendering parameters provided by a Python extension when registering a style.
/// Colors are hex strings ("#rrggbb"). `border_chars` is a 7-char string:
/// top-left, top, top-right, side, bottom-left, bottom, bottom-right.
///
/// Optional **`border_horizontal_pattern`** / **`border_vertical_pattern`** override the single
/// horizontal (`border_chars` indices 1 & 5) and vertical (index 3) glyphs with **repeating**
/// sequences — include ordinary spaces for gaps (e.g. `"- "` or `"├ "`). Corners (indices 0,2,4,6)
/// stay single glyphs from `border_chars`.
#[derive(Clone, Debug, Default)]
pub struct GlyphParams {
    pub bg: Option<String>,
    pub surface: Option<String>,
    pub surface2: Option<String>,
    pub text: Option<String>,
    pub dim: Option<String>,
    /// Optional global UI font family for the app surface (e.g. "Inter", "IBM Plex Sans").
    pub ui_font_family: Option<String>,
    pub border: Option<String>,
    pub accent: Option<String>,
    pub border_chars: Option<String>,
    /// Repeating sequence for top and bottom rules (indices 1 & 5 when unset). Spaces allowed.
    pub border_horizontal_pattern: Option<String>,
    /// Repeating sequence for left/right rails (index 3 when unset). One character per line; cycles.
    pub border_vertical_pattern: Option<String>,
    /// Font family for glyph-drawn panel borders, e.g. `"monospace"` or `"JetBrains Mono"`.
    pub border_font_family: Option<String>,
    /// Font size for border glyphs in **rem** (matches GPUI `text_xs` scale when unset).
    pub border_font_size_rems: Option<f32>,
    /// Width in px of the left/right border rails (fits wide box-drawing glyphs when larger).
    pub border_side_rail_px: Option<f32>,
    pub border_radius: f32,
}

/// Declared UI/config token for a styling extension (persisted under `extension_tokens/`).
#[derive(Clone, Debug)]
pub struct StyleTokenSpec {
    pub key: String,
    pub label: String,
    pub kind: StyleTokenKind,
    /// Serialized default (same representation saved in TOML).
    pub default_value: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StyleTokenKind {
    Color,
    Float,
    String,
    Bool,
    Int,
}

/// Metadata for a render style registered by a Python extension.
#[derive(Clone, Debug)]
pub struct StyleInfo {
    pub name: String,
    pub label: String,
    pub description: String,
    /// Glyph params used for dark mode (or all modes when `glyph_light` is `None`).
    pub glyph: Option<GlyphParams>,
    /// Optional light-mode glyph params for mode-specific alternates.
    pub glyph_light: Option<GlyphParams>,
    /// Owning Python extension module id (e.g. `tui-style`) for token files + overrides.
    pub module_name: Option<String>,
}

type DynCommandFn = Arc<dyn Fn(Vec<String>) -> String + Send + Sync + 'static>;
type ReloadFn = Arc<dyn Fn() -> Result<(), String> + Send + Sync + 'static>;

struct PythonRegistry {
    modules: Vec<PythonModuleInfo>,
    commands: HashMap<String, (String, DynCommandFn)>, // token → (description, fn)
    reload_fn: Option<ReloadFn>,
    styles: Vec<StyleInfo>,
    /// Extension module id → token declarations from `register_tokens`.
    style_tokens: HashMap<String, Vec<StyleTokenSpec>>,
}

impl PythonRegistry {
    fn new() -> Self {
        Self {
            modules: Vec::new(),
            commands: HashMap::new(),
            reload_fn: None,
            styles: Vec::new(),
            style_tokens: HashMap::new(),
        }
    }
}

fn registry() -> &'static Mutex<PythonRegistry> {
    static REGISTRY: OnceLock<Mutex<PythonRegistry>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(PythonRegistry::new()))
}

pub fn register_module(name: String, version: String, description: String) {
    if let Ok(mut reg) = registry().lock() {
        // Preserve existing enabled state if re-registering (e.g. after reload).
        let was_enabled = reg.modules.iter().find(|m| m.name == name).map(|m| m.enabled);
        reg.modules.retain(|m| m.name != name);
        reg.modules.push(PythonModuleInfo {
            name,
            version,
            description,
            enabled: was_enabled.unwrap_or(true),
        });
    }
}

pub fn set_extension_enabled(name: &str, enabled: bool) {
    if let Ok(mut reg) = registry().lock() {
        if let Some(m) = reg.modules.iter_mut().find(|m| m.name == name) {
            m.enabled = enabled;
        }
    }
}

pub fn register_command(token: String, description: String, handler: DynCommandFn) {
    if let Ok(mut reg) = registry().lock() {
        reg.commands.insert(token, (description, handler));
    }
}

pub fn set_reload_handler(f: ReloadFn) {
    if let Ok(mut reg) = registry().lock() {
        reg.reload_fn = Some(f);
    }
}

pub fn register_style(
    name: String,
    label: String,
    description: String,
    glyph: Option<GlyphParams>,
    glyph_light: Option<GlyphParams>,
    module_name: Option<String>,
) {
    if let Ok(mut reg) = registry().lock() {
        reg.styles.retain(|s| s.name != name);
        reg.styles.push(StyleInfo {
            name,
            label,
            description,
            glyph,
            glyph_light,
            module_name,
        });
    }
}

pub fn register_tokens(module_name: String, tokens: Vec<StyleTokenSpec>) {
    if let Ok(mut reg) = registry().lock() {
        reg.style_tokens.insert(module_name, tokens);
    }
}

pub fn list_style_tokens() -> Vec<(String, Vec<StyleTokenSpec>)> {
    let Ok(reg) = registry().lock() else {
        return Vec::new();
    };
    let mut out: Vec<(String, Vec<StyleTokenSpec>)> = reg
        .style_tokens
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

pub fn list_styles() -> Vec<StyleInfo> {
    let Ok(reg) = registry().lock() else { return Vec::new() };
    reg.styles.clone()
}

pub fn clear() {
    if let Ok(mut reg) = registry().lock() {
        reg.modules.clear();
        reg.commands.clear();
        reg.styles.clear();
        reg.style_tokens.clear();
    }
}

pub fn try_dispatch(token: &str, args: &[&str]) -> Option<String> {
    let handler = {
        let reg = registry().lock().ok()?;
        // Check that the owning extension module is enabled.
        if let Some(module_name) = token.split_once('.').map(|(m, _)| m) {
            if let Some(m) = reg.modules.iter().find(|m| m.name == module_name) {
                if !m.enabled {
                    return None;
                }
            }
        }
        reg.commands.get(token).map(|(_, f)| Arc::clone(f))?
    };
    let args_vec: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    Some(handler(args_vec))
}

pub fn reload() -> Result<(), String> {
    let reload_fn = {
        let reg = registry().lock().map_err(|_| "Registry poisoned".to_string())?;
        reg.reload_fn.as_ref().map(Arc::clone)
    };
    match reload_fn {
        Some(f) => f(),
        None => Err(
            "Python host not initialized. Restart the app with python-host enabled.".to_string(),
        ),
    }
}

pub fn list_modules() -> Vec<(String, String, String, bool)> {
    let Ok(reg) = registry().lock() else { return Vec::new() };
    reg.modules
        .iter()
        .map(|m| (m.name.clone(), m.version.clone(), m.description.clone(), m.enabled))
        .collect()
}

pub fn list_commands() -> Vec<(String, String)> {
    let Ok(reg) = registry().lock() else { return Vec::new() };
    reg.commands
        .iter()
        .map(|(token, (desc, _))| (token.clone(), desc.clone()))
        .collect()
}
