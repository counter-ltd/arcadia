use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use crate::config::modules::supports_runtime_platform_owned;
use crate::modules::style_tokens;

pub use style_tokens::{
    clamp_numeric_display_for_spec, effective_token_display, format_slider_value,
    merge_token_numeric_bounds, numeric_clamp_lo_hi, parse_granularity_str, resolve_slider_numeric,
    snap_slider_value, style_token_row_visible, StyleTokenCompareOp, StyleTokenKind,
    StyleTokenNumericBounds, StyleTokenNumericGranularity, StyleTokenNumericPartial, StyleTokenSpec,
    StyleTokenVisibility,
};

pub struct PythonModuleInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub enabled: bool,
    /// Declared at `register_module` for first-enable / settings UI.
    pub required_permissions: Vec<String>,
    /// Empty = all platforms. Otherwise whitelist of `macos` / `windows` / `linux` / `ios` / `unknown`.
    pub supported_platforms: Vec<String>,
    /// On-disk source for this extension — populated by `register_discovered` so the host can
    /// call `load_extension(path)` when the user toggles a stub on.
    pub path: Option<PathBuf>,
    /// `true` once the extension's Python body has actually run (i.e. `register_module` was
    /// called from inside the script). Stubs created by `register_discovered` start as `false`
    /// so the UI can label them "Disabled — not loaded" until the user opts in.
    pub loaded: bool,
}

type DynCommandFn = Arc<dyn Fn(Vec<String>) -> String + Send + Sync + 'static>;
type TrayIconClickFn = Arc<dyn Fn(Vec<String>) + Send + Sync + 'static>;
type ReloadFn = Arc<dyn Fn() -> Result<(), String> + Send + Sync + 'static>;
type DynHighlightFn = Arc<dyn Fn(&str) -> Vec<HighlightSpan> + Send + Sync + 'static>;
type DynDecorationFn = Arc<dyn Fn(&str, usize) -> Vec<DecorationRect> + Send + Sync + 'static>;
/// Loader for a single extension: takes the stub id the host knows the file by plus the
/// on-disk path, and returns the canonical module name the body actually registered (often
/// the same as the stub id, but may differ when the folder name and `register_module(name=…)`
/// disagree — see `load_and_merge` in `arcadia-python`).
type LoadOneFn = Arc<dyn Fn(String, PathBuf) -> Result<String, String> + Send + Sync + 'static>;

/// A syntax-highlighted byte range within the document. `token` is the semantic name
/// declared by the extension (e.g. `"keyword"`, `"string"`, `"comment"`).
#[derive(Clone)]
pub struct HighlightSpan {
    pub start: usize,
    pub end: usize,
    pub token: String,
}

/// A coloured decoration box covering `col_width` columns at `col_start` within a line.
/// Rendered behind text as a translucent rectangle.
#[derive(Clone)]
pub struct DecorationRect {
    pub col_start: usize,
    pub col_width: usize,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

struct CommandRecord {
    description: String,
    required_permissions: Vec<String>,
    handler: DynCommandFn,
}

struct PythonRegistry {
    modules: Vec<PythonModuleInfo>,
    commands: HashMap<String, CommandRecord>,
    reload_fn: Option<ReloadFn>,
    /// Host-supplied loader for a single extension file — invoked by `extension-enable` to
    /// execute the Python body on demand without touching already-loaded extensions.
    load_one_fn: Option<LoadOneFn>,
    styles: Vec<StyleInfo>,
    /// Extension module id → token declarations from `register_tokens`.
    style_tokens: HashMap<String, Vec<style_tokens::StyleTokenSpec>>,
    /// Shortcuts registered via `register_extension_shortcut` (cleared on reload).
    shortcuts: Vec<crate::shortcuts::EffectiveMergedShortcut>,
    /// Per-extension tray-icon click handler: `(tray_item_id, button)` as `["tray-1","left"]`.
    tray_icon_click_handlers: HashMap<String, TrayIconClickFn>,
    /// Language id → (ext_id, fn). One provider per language; later registration wins.
    highlight_providers: HashMap<String, (String, DynHighlightFn)>,
    /// Ordered list of decoration providers: (ext_id, fn).
    decoration_providers: Vec<(String, DynDecorationFn)>,
    /// Extension ids whose tokens should appear in the editor settings panel instead of as
    /// standalone settings pages.
    editor_token_modules: std::collections::HashSet<String>,
    /// Nav pages declared by extensions via `register_nav_page`.
    nav_pages: Vec<NavPageDeclaration>,
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

/// Button style hint for a nav page action.
#[derive(Clone, Debug, PartialEq)]
pub enum NavActionStyle {
    Primary,
    Secondary,
    Destructive,
}

/// A single callable action declared by an extension's nav page.
#[derive(Clone, Debug)]
pub struct NavPageAction {
    pub label: String,
    pub command: String,
    pub args: Vec<String>,
    pub style: NavActionStyle,
}

/// A navigation page declared by a Python extension via `arcadia.register_nav_page()`.
/// The page ID is always `"python.nav_page|{extension_id}"`.
#[derive(Clone, Debug)]
pub struct NavPageDeclaration {
    pub extension_id: String,
    pub group_id: String,
    pub title: String,
    pub description: String,
    pub glyph: String,
    pub system_image: String,
    pub accent: String,
    pub status_command: Option<String>,
    pub actions: Vec<NavPageAction>,
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
    /// Owning Python extension module id (e.g. `shell-theme`) for token files + overrides.
    pub module_name: Option<String>,
}

impl PythonRegistry {
    fn new() -> Self {
        Self {
            modules: Vec::new(),
            commands: HashMap::new(),
            reload_fn: None,
            load_one_fn: None,
            styles: Vec::new(),
            style_tokens: HashMap::new(),
            shortcuts: Vec::new(),
            tray_icon_click_handlers: HashMap::new(),
            highlight_providers: HashMap::new(),
            decoration_providers: Vec::new(),
            editor_token_modules: std::collections::HashSet::new(),
            nav_pages: Vec::new(),
        }
    }
}

fn registry() -> &'static Mutex<PythonRegistry> {
    static REGISTRY: OnceLock<Mutex<PythonRegistry>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(PythonRegistry::new()))
}

pub fn register_module(
    name: String,
    version: String,
    description: String,
    required_permissions: Vec<String>,
    supported_platforms: Vec<String>,
) {
    if let Ok(mut reg) = registry().lock() {
        let prev = reg.modules.iter().find(|m| m.name == name);
        let was_enabled = prev.map(|m| m.enabled);
        let was_perms = prev.map(|m| m.required_permissions.clone());
        let was_plat = prev.map(|m| m.supported_platforms.clone());
        let was_path = prev.and_then(|m| m.path.clone());
        reg.modules.retain(|m| m.name != name);
        let perms = if required_permissions.is_empty() {
            was_perms.unwrap_or_default()
        } else {
            required_permissions
        };
        let platforms = if supported_platforms.is_empty() {
            was_plat.unwrap_or_default()
        } else {
            supported_platforms
        };
        // Default to `false` for first-ever registration. The previous behaviour of
        // `unwrap_or(true)` auto-enabled every extension dropped into `~/Arcadia/Extensions/`
        // on first launch, which is exactly what we don't want — the user must explicitly
        // opt in via the Extensions settings page so OS permission prompts only fire when
        // the user has accepted them in Arcadia. See `python_extensions` in `ModulesConfig`.
        reg.modules.push(PythonModuleInfo {
            name,
            version,
            description,
            enabled: was_enabled.unwrap_or(false),
            required_permissions: perms,
            supported_platforms: platforms,
            path: was_path,
            loaded: true,
        });
    }
}

/// Pre-register an extension discovered on disk before its body has run. The loader calls this
/// for every `.py` / `<dir>/main.py` it finds so the Extensions settings page can list and
/// toggle extensions without first executing their (potentially side-effectful) bodies.
///
/// `persisted_enabled` comes from `ModulesConfig.python_extensions[id]` — when `false` the
/// loader skips `load_extension(path)` and the stub stays in the registry as a disabled,
/// unloaded entry the user can flip on later.
///
/// `declared_permissions` is pre-parsed from the source by the host so the first-enable
/// permission modal can prompt the user *before* the body actually runs and tries to call
/// permission-protected APIs.
///
/// `declared_platforms` is pre-parsed from `platforms=[...]` in the same call (see
/// `arcadia-python`); empty means all platforms.
pub fn register_discovered(
    id: String,
    path: PathBuf,
    persisted_enabled: bool,
    declared_permissions: Vec<String>,
    declared_platforms: Vec<String>,
) {
    if id.is_empty() {
        return;
    }
    if let Ok(mut reg) = registry().lock() {
        if let Some(existing) = reg.modules.iter_mut().find(|m| m.name == id) {
            existing.path = Some(path);
            existing.enabled = persisted_enabled;
            // Don't clobber permissions declared from a fully-loaded body — the body is the
            // ground truth once it has run.
            if !existing.loaded && !declared_permissions.is_empty() {
                existing.required_permissions = declared_permissions;
            }
            if !existing.loaded && !declared_platforms.is_empty() {
                existing.supported_platforms = declared_platforms;
            }
            return;
        }
        reg.modules.push(PythonModuleInfo {
            name: id,
            version: "0.0.0".to_string(),
            description: "(not loaded)".to_string(),
            enabled: persisted_enabled,
            required_permissions: declared_permissions,
            supported_platforms: declared_platforms,
            path: Some(path),
            loaded: false,
        });
    }
}

/// Source path recorded by `register_discovered` for an extension id, if any. Used by
/// `python-host.extension-enable` to actually load the body when the user opts in.
pub fn extension_path(name: &str) -> Option<PathBuf> {
    let reg = registry().lock().ok()?;
    reg.modules
        .iter()
        .find(|m| m.name == name)
        .and_then(|m| m.path.clone())
}

/// On-disk directory for one extension bundle: parent of `main.py` or of a loose `.py` entry.
///
/// Layout for import/download: `…/Extensions/<folder>/main.py` plus `…/Extensions/<folder>/Assets/*`.
pub fn extension_bundle_root(extension_module_id: &str) -> Option<PathBuf> {
    let path = extension_path(extension_module_id)?;
    path.parent().map(|p| p.to_path_buf())
}

/// Canonical asset directory: `<extension_bundle_root>/Assets`.
pub fn extension_assets_dir(extension_module_id: &str) -> Option<PathBuf> {
    Some(extension_bundle_root(extension_module_id)?.join("Assets"))
}

fn validate_extension_asset_relative(relative: &str) -> Result<(), String> {
    let s = relative.trim();
    if s.is_empty() {
        return Err("empty relative asset path".into());
    }
    let p = Path::new(s);
    if p.is_absolute() {
        return Err("absolute asset paths are not allowed".into());
    }
    for c in p.components() {
        match c {
            Component::Normal(os) => {
                if os.to_str().is_none() {
                    return Err("asset path must be UTF-8".into());
                }
            }
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err("invalid asset path".into());
            }
        }
    }
    Ok(())
}

/// Resolved path under [`extension_assets_dir`] for a safe relative path (no `..`, not absolute).
pub fn resolve_extension_asset_path(
    extension_module_id: &str,
    relative_under_assets: &str,
) -> Result<PathBuf, String> {
    validate_extension_asset_relative(relative_under_assets)?;
    let base = extension_assets_dir(extension_module_id)
        .ok_or_else(|| format!("no on-disk path for extension '{extension_module_id}'"))?;
    Ok(base.join(relative_under_assets.trim_start_matches(['/', '\\'])))
}

/// Snapshot of all currently-registered module ids, in registration order. Used by the loader
/// to detect entries created during a single `load_extension` call so we can collapse the
/// stub onto the body's declared canonical name (folder rename / declared-name mismatch).
pub fn module_names() -> Vec<String> {
    let Ok(reg) = registry().lock() else {
        return Vec::new();
    };
    reg.modules.iter().map(|m| m.name.clone()).collect()
}

/// Drop an entry by name. Used when a body has registered a module under a different
/// canonical name than the stub created from the on-disk path — the stub is then redundant.
pub fn remove_module(name: &str) {
    if name.is_empty() {
        return;
    }
    if let Ok(mut reg) = registry().lock() {
        reg.modules.retain(|m| m.name != name);
        // Drop any per-extension token specs / styles / shortcuts owned by the stub id too.
        reg.style_tokens.remove(name);
        reg.styles
            .retain(|s| s.module_name.as_deref() != Some(name));
        reg.shortcuts
            .retain(|s| s.source_extension_id.as_deref() != Some(name));
        reg.tray_icon_click_handlers.remove(name);
        let prefix = format!("{name}.");
        reg.commands.retain(|token, _| !token.starts_with(&prefix));
    }
}

/// Attach (or replace) the on-disk source path for an existing extension entry. Called when
/// a stub is merged into the body's declared name so subsequent toggles can still find the
/// `.py` file to reload.
pub fn attach_path(name: &str, path: PathBuf) {
    if name.is_empty() {
        return;
    }
    if let Ok(mut reg) = registry().lock() {
        if let Some(m) = reg.modules.iter_mut().find(|m| m.name == name) {
            m.path = Some(path);
        }
    }
}

/// Forget any in-memory command, style, token, and shortcut state contributed by an extension
/// — used when the user disables an extension so stale handlers don't keep firing.
pub fn unregister_extension_contributions(name: &str) {
    if name.is_empty() {
        return;
    }
    if let Ok(mut reg) = registry().lock() {
        let prefix = format!("{name}.");
        reg.commands.retain(|token, _| !token.starts_with(&prefix));
        reg.styles
            .retain(|s| s.module_name.as_deref() != Some(name));
        reg.style_tokens.remove(name);
        reg.shortcuts
            .retain(|s| s.source_extension_id.as_deref() != Some(name));
        reg.tray_icon_click_handlers.remove(name);
        reg.highlight_providers.retain(|_, (id, _)| id != name);
        reg.decoration_providers.retain(|(id, _)| id != name);
        reg.editor_token_modules.remove(name);
        if let Some(m) = reg.modules.iter_mut().find(|m| m.name == name) {
            m.loaded = false;
        }
    }
}

pub fn set_extension_enabled(name: &str, enabled: bool) {
    if let Ok(mut reg) = registry().lock() {
        if let Some(m) = reg.modules.iter_mut().find(|m| m.name == name) {
            m.enabled = enabled;
        }
    }
}

pub fn extension_enabled(name: &str) -> bool {
    let Ok(reg) = registry().lock() else {
        return false;
    };
    reg.modules
        .iter()
        .find(|m| m.name == name)
        .map(|m| m.enabled)
        .unwrap_or(false)
}

/// `true` once an extension body has executed (`register_module` ran). Used to avoid loading the
/// same `main.py` twice when the host is started from more than one surface entrypoint.
pub fn extension_body_loaded(name: &str) -> bool {
    let Ok(reg) = registry().lock() else {
        return false;
    };
    reg.modules
        .iter()
        .find(|m| m.name == name)
        .is_some_and(|m| m.loaded)
}

pub fn register_extension_shortcut(
    shortcut: crate::shortcuts::EffectiveMergedShortcut,
) -> Result<(), String> {
    let Some(ext) = shortcut.source_extension_id.as_deref() else {
        return Err("Python shortcuts must set source extension id".into());
    };
    if ext.is_empty() {
        return Err("extension id cannot be empty".into());
    }
    let Ok(mut reg) = registry().lock() else {
        return Err("registry poisoned".into());
    };
    reg.shortcuts
        .retain(|s| !(s.source_extension_id.as_deref() == Some(ext) && s.id == shortcut.id));
    reg.shortcuts.push(shortcut);
    Ok(())
}

pub fn list_extension_shortcuts() -> Vec<crate::shortcuts::EffectiveMergedShortcut> {
    let Ok(reg) = registry().lock() else {
        return Vec::new();
    };
    reg.shortcuts.clone()
}

pub fn register_command(
    token: String,
    description: String,
    handler: DynCommandFn,
    required_permissions: Vec<String>,
) {
    if let Ok(mut reg) = registry().lock() {
        reg.commands.insert(
            token,
            CommandRecord {
                description,
                required_permissions,
                handler,
            },
        );
    }
}

pub fn register_tray_icon_click_handler(extension_id: String, handler: TrayIconClickFn) {
    if extension_id.is_empty() {
        return;
    }
    if let Ok(mut reg) = registry().lock() {
        reg.tray_icon_click_handlers.insert(extension_id, handler);
    }
}

/// Register a syntax highlight provider for `language` (e.g. `"rust"`). One provider per
/// language; a later call replaces the previous one. Lock is released before calling the fn.
pub fn register_highlight_provider(language: String, ext_id: String, f: DynHighlightFn) {
    if language.is_empty() || ext_id.is_empty() {
        return;
    }
    if let Ok(mut reg) = registry().lock() {
        reg.highlight_providers.insert(language, (ext_id, f));
    }
}

/// Register a decoration provider (coloured boxes per line). Multiple providers coexist.
pub fn register_decoration_provider(ext_id: String, f: DynDecorationFn) {
    if ext_id.is_empty() {
        return;
    }
    if let Ok(mut reg) = registry().lock() {
        reg.decoration_providers.retain(|(id, _)| id != &ext_id);
        reg.decoration_providers.push((ext_id, f));
    }
}

/// Mark an extension's tokens as editor-scoped so they render inside the Editor settings
/// panel instead of appearing as a separate standalone settings page.
pub fn register_editor_token_module(ext_id: String) {
    if ext_id.is_empty() {
        return;
    }
    if let Ok(mut reg) = registry().lock() {
        reg.editor_token_modules.insert(ext_id);
    }
}

/// Token modules registered via [`register_editor_token_module`], filtered to those that
/// have token specs and are enabled + platform-compatible. Sorted by module id.
pub fn editor_scoped_token_modules() -> Vec<(String, Vec<style_tokens::StyleTokenSpec>)> {
    let Ok(reg) = registry().lock() else {
        return Vec::new();
    };
    let mut out: Vec<(String, Vec<StyleTokenSpec>)> = reg
        .style_tokens
        .iter()
        .filter(|(k, specs)| {
            if specs.is_empty() || !reg.editor_token_modules.contains(k.as_str()) {
                return false;
            }
            reg.modules
                .iter()
                .find(|m| m.name == **k)
                .map(|m| supports_runtime_platform_owned(&m.supported_platforms))
                .unwrap_or(true)
        })
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// Call the highlight provider registered for `language`, if any. Lock is acquired just to
/// clone the Arc, then released before calling the fn (which may acquire the Python GIL).
pub fn call_highlight_provider(language: &str, text: &str) -> Vec<HighlightSpan> {
    let handler = {
        let Ok(reg) = registry().lock() else {
            return Vec::new();
        };
        reg.highlight_providers
            .get(language)
            .map(|(_, f)| Arc::clone(f))
    };
    handler.map(|f| f(text)).unwrap_or_default()
}

/// Call all registered decoration providers for `line` at `line_idx`, collecting results.
pub fn call_decoration_providers(line: &str, line_idx: usize) -> Vec<DecorationRect> {
    let handlers: Vec<DynDecorationFn> = {
        let Ok(reg) = registry().lock() else {
            return Vec::new();
        };
        reg.decoration_providers
            .iter()
            .map(|(_, f)| Arc::clone(f))
            .collect()
    };
    handlers
        .into_iter()
        .flat_map(|f| f(line, line_idx))
        .collect()
}

/// Tray-icon primary click (from desktop `tray-icon`); forwards to the owning Python extension
/// when it registered [`register_tray_icon_click_handler`].
pub fn dispatch_tray_icon_click(tray_item_id: String, button: String) {
    use crate::modules::tray;

    let Some(item) = tray::get_item(&tray_item_id) else {
        return;
    };
    let rest = item.owner.strip_prefix("python:");
    let Some(rest) = rest else {
        return;
    };
    let ext_id = rest
        .split_once("::")
        .map(|(base, _)| base.to_string())
        .unwrap_or_else(|| rest.to_string());

    let handler = {
        let Ok(reg) = registry().lock() else {
            return;
        };
        if let Some(m) = reg.modules.iter().find(|m| m.name == ext_id) {
            if !m.enabled {
                return;
            }
            if !supports_runtime_platform_owned(&m.supported_platforms) {
                return;
            }
        }
        reg.tray_icon_click_handlers.get(&ext_id).cloned()
    };
    let Some(handler) = handler else {
        return;
    };

    let tid = tray_item_id.clone();
    let btn = button.clone();
    crate::scheduling::spawn(move || {
        handler(vec![tid, btn]);
    });
}

pub fn set_reload_handler(f: ReloadFn) {
    if let Ok(mut reg) = registry().lock() {
        reg.reload_fn = Some(f);
    }
}

pub fn set_load_one_handler(f: LoadOneFn) {
    if let Ok(mut reg) = registry().lock() {
        reg.load_one_fn = Some(f);
    }
}

/// Execute a single extension's Python body via the host-registered loader. Used by
/// `python-host.extension-enable` to bring a previously-disabled extension online without
/// re-scanning the entire `~/Arcadia/Extensions/` directory.
///
/// `stub_id` is the id the registry currently knows the extension by (typically the
/// folder-derived id from `register_discovered`). The returned `String` is the canonical
/// module name after the body has run, which may differ from `stub_id` when the body
/// declares a different `register_module(name=…)`.
pub fn load_one(stub_id: String, path: PathBuf) -> Result<String, String> {
    let load_fn = {
        let reg = registry()
            .lock()
            .map_err(|_| "Registry poisoned".to_string())?;
        reg.load_one_fn.as_ref().map(Arc::clone)
    };
    match load_fn {
        Some(f) => f(stub_id, path),
        None => Err(
            "Python host not initialized. Restart the app with python-host enabled.".to_string(),
        ),
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

pub fn register_tokens(module_name: String, tokens: Vec<style_tokens::StyleTokenSpec>) {
    if let Ok(mut reg) = registry().lock() {
        reg.style_tokens.insert(module_name, tokens);
    }
}

pub fn list_style_tokens() -> Vec<(String, Vec<style_tokens::StyleTokenSpec>)> {
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

/// `true` when this extension owns a registered render style — those tokens are edited under
/// Appearance when that style is active, not on a standalone settings page.
pub fn extension_tokens_editable_under_appearance(module_id: &str) -> bool {
    let Ok(reg) = registry().lock() else {
        return false;
    };
    reg.styles
        .iter()
        .any(|s| s.module_name.as_deref() == Some(module_id))
}

/// Extensions with `register_tokens` entries that are **not** tied to an Appearance style.
pub fn standalone_extension_token_modules() -> Vec<(String, Vec<style_tokens::StyleTokenSpec>)> {
    let Ok(reg) = registry().lock() else {
        return Vec::new();
    };
    let style_modules: std::collections::HashSet<&str> = reg
        .styles
        .iter()
        .filter_map(|s| s.module_name.as_deref())
        .collect();
    let mut out: Vec<(String, Vec<StyleTokenSpec>)> = reg
        .style_tokens
        .iter()
        .filter(|(k, specs)| {
            if specs.is_empty()
                || style_modules.contains(k.as_str())
                || reg.editor_token_modules.contains(k.as_str())
            {
                return false;
            }
            let plat_ok = reg
                .modules
                .iter()
                .find(|m| m.name == **k)
                .map(|m| supports_runtime_platform_owned(&m.supported_platforms))
                .unwrap_or(true);
            plat_ok
        })
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

pub fn list_styles() -> Vec<StyleInfo> {
    let Ok(reg) = registry().lock() else {
        return Vec::new();
    };
    reg.styles.clone()
}

pub fn clear() {
    crate::scheduling::cancel_all_recurring();
    let owners: std::collections::HashSet<String> = crate::modules::tray::list_items()
        .into_iter()
        .filter(|it| it.owner.starts_with("python:"))
        .map(|it| it.owner)
        .collect();
    for o in owners {
        let _ = crate::modules::tray::remove_items_for_owner(&o);
    }
    if let Ok(mut reg) = registry().lock() {
        reg.modules.clear();
        reg.commands.clear();
        reg.styles.clear();
        reg.style_tokens.clear();
        reg.shortcuts.clear();
        reg.tray_icon_click_handlers.clear();
        reg.highlight_providers.clear();
        reg.decoration_providers.clear();
        reg.editor_token_modules.clear();
        reg.nav_pages.clear();
    }
}

pub fn register_nav_page(decl: NavPageDeclaration) {
    if let Ok(mut reg) = registry().lock() {
        reg.nav_pages.retain(|p| p.extension_id != decl.extension_id);
        reg.nav_pages.push(decl);
    }
}

pub fn list_nav_pages() -> Vec<NavPageDeclaration> {
    let Ok(reg) = registry().lock() else {
        return Vec::new();
    };
    reg.nav_pages.clone()
}

pub fn nav_page_for(extension_id: &str) -> Option<NavPageDeclaration> {
    let Ok(reg) = registry().lock() else {
        return None;
    };
    reg.nav_pages.iter().find(|p| p.extension_id == extension_id).cloned()
}

pub fn try_dispatch(token: &str, args: &[&str]) -> Result<Option<String>, String> {
    let ext_id = token
        .split_once('.')
        .map(|(a, _)| a)
        .ok_or_else(|| "Invalid command token".to_string())?;

    let (handler, perms) = {
        let reg = registry()
            .lock()
            .map_err(|_| "Python registry poisoned".to_string())?;
        if let Some(m) = reg.modules.iter().find(|m| m.name == ext_id) {
            if !m.enabled {
                return Ok(None);
            }
            if !supports_runtime_platform_owned(&m.supported_platforms) {
                return Err(format!(
                    "Extension '{ext_id}' is not supported on this platform"
                ));
            }
        }
        let Some(rec) = reg.commands.get(token) else {
            return Ok(None);
        };
        (Arc::clone(&rec.handler), rec.required_permissions.clone())
    };

    use crate::config::permissions::{
        is_known_permission_id, PermissionSubject, PermissionsConfig,
    };
    use crate::config::ConfigFile;

    let cfg = PermissionsConfig::load_or_create().map_err(|e| e.to_string())?;
    let subj = PermissionSubject::python(ext_id.to_string());
    for p in &perms {
        if !is_known_permission_id(p) {
            return Err(format!(
                "Extension '{ext_id}' references unknown permission id: {p}"
            ));
        }
        if !cfg.effective_allowed(&subj, p) {
            return Err(format!(
                "Permission denied: {p} (subject {})",
                subj.storage_key()
            ));
        }
    }

    let args_vec: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    Ok(Some(handler(args_vec)))
}

pub fn reload() -> Result<(), String> {
    let reload_fn = {
        let reg = registry()
            .lock()
            .map_err(|_| "Registry poisoned".to_string())?;
        reg.reload_fn.as_ref().map(Arc::clone)
    };
    match reload_fn {
        Some(f) => f(),
        None => Err(
            "Python host not initialized. Restart the app with python-host enabled.".to_string(),
        ),
    }
}

pub fn list_modules() -> Vec<(String, String, String, bool, Vec<String>, Vec<String>)> {
    let Ok(reg) = registry().lock() else {
        return Vec::new();
    };
    reg.modules
        .iter()
        .map(|m| {
            (
                m.name.clone(),
                m.version.clone(),
                m.description.clone(),
                m.enabled,
                m.required_permissions.clone(),
                m.supported_platforms.clone(),
            )
        })
        .collect()
}

/// `true` when the extension declares no platform list or the current host OS is listed.
pub fn extension_supported_at_runtime(extension_id: &str) -> bool {
    let Ok(reg) = registry().lock() else {
        return true;
    };
    reg.modules
        .iter()
        .find(|m| m.name == extension_id)
        .map(|m| supports_runtime_platform_owned(&m.supported_platforms))
        .unwrap_or(true)
}

/// Whether a `extensionId.command` token may run on this host (extension exists and passes platform).
pub fn extension_command_supported_at_runtime(token: &str) -> bool {
    let Some((ext_id, _)) = token.split_once('.') else {
        return true;
    };
    extension_supported_at_runtime(ext_id)
}

pub fn extension_declared_permissions(name: &str) -> Vec<String> {
    let Ok(reg) = registry().lock() else {
        return Vec::new();
    };
    reg.modules
        .iter()
        .find(|m| m.name == name)
        .map(|m| m.required_permissions.clone())
        .unwrap_or_default()
}

pub fn list_commands() -> Vec<(String, String)> {
    let Ok(reg) = registry().lock() else {
        return Vec::new();
    };
    reg.commands
        .iter()
        .map(|(token, rec)| (token.clone(), rec.description.clone()))
        .collect()
}

#[cfg(test)]
mod extension_asset_path_tests {
    use super::*;

    #[test]
    fn bundle_and_assets_under_main_py() {
        register_discovered(
            "asset-test-a".into(),
            PathBuf::from("/fake/Arcadia/Extensions/my_ext/main.py"),
            false,
            vec![],
            vec![],
        );
        assert_eq!(
            extension_bundle_root("asset-test-a").unwrap(),
            PathBuf::from("/fake/Arcadia/Extensions/my_ext")
        );
        assert_eq!(
            extension_assets_dir("asset-test-a").unwrap(),
            PathBuf::from("/fake/Arcadia/Extensions/my_ext/Assets")
        );
        assert_eq!(
            resolve_extension_asset_path("asset-test-a", "icons/x.png").unwrap(),
            PathBuf::from("/fake/Arcadia/Extensions/my_ext/Assets/icons/x.png")
        );
        remove_module("asset-test-a");
    }

    #[test]
    fn bundle_root_for_loose_py_is_parent_dir() {
        register_discovered(
            "asset-test-b".into(),
            PathBuf::from("/fake/Arcadia/Extensions/oneoff.py"),
            false,
            vec![],
            vec![],
        );
        assert_eq!(
            extension_assets_dir("asset-test-b").unwrap(),
            PathBuf::from("/fake/Arcadia/Extensions/Assets")
        );
        remove_module("asset-test-b");
    }

    #[test]
    fn rejects_parent_dir_in_relative() {
        register_discovered(
            "asset-test-c".into(),
            PathBuf::from("/fake/Arcadia/Extensions/z/main.py"),
            false,
            vec![],
            vec![],
        );
        assert!(resolve_extension_asset_path("asset-test-c", "../secret").is_err());
        remove_module("asset-test-c");
    }
}
