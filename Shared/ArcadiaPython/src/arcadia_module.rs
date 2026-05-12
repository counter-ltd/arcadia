use arcadia_core::config::extension_tokens;
use arcadia_core::config::permissions::{PermissionSubject, PermissionsConfig};
use arcadia_core::config::ConfigFile;
use arcadia_core::modules::python_registry::StyleTokenKind;
use arcadia_core::modules::style_tokens::{
    merge_token_numeric_bounds, parse_granularity_str, StyleTokenCompareOp, StyleTokenNumericPartial,
    StyleTokenSpec, StyleTokenVisibility,
};
use arcadia_core::modules::{
    animation, cursor as core_cursor, overlay_hud_sprite, python_registry, tray as core_tray,
    ExecutionContext,
};
use arcadia_core::scheduling;
use arcadia_core::shortcuts::ShortcutRegistrationOwned;
use crate::python_scope;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList};
use std::sync::Arc;
use std::time::Duration;

fn glyph_from_fields(
    bg: Option<String>,
    surface: Option<String>,
    surface2: Option<String>,
    text: Option<String>,
    dim: Option<String>,
    ui_font_family: Option<String>,
    border: Option<String>,
    accent: Option<String>,
    border_chars: Option<String>,
    border_horizontal_pattern: Option<String>,
    border_vertical_pattern: Option<String>,
    border_font_family: Option<String>,
    border_font_size_rems: Option<f32>,
    border_side_rail_px: Option<f32>,
    border_radius: f32,
) -> Option<python_registry::GlyphParams> {
    let has_glyph = bg.is_some()
        || surface.is_some()
        || surface2.is_some()
        || text.is_some()
        || dim.is_some()
        || ui_font_family.is_some()
        || border.is_some()
        || accent.is_some()
        || border_chars.is_some()
        || border_horizontal_pattern.is_some()
        || border_vertical_pattern.is_some()
        || border_font_family.is_some()
        || border_font_size_rems.is_some()
        || border_side_rail_px.is_some();

    if has_glyph {
        Some(python_registry::GlyphParams {
            bg,
            surface,
            surface2,
            text,
            dim,
            ui_font_family,
            border,
            accent,
            border_chars,
            border_horizontal_pattern,
            border_vertical_pattern,
            border_font_family,
            border_font_size_rems,
            border_side_rail_px,
            border_radius,
        })
    } else {
        None
    }
}

#[pyfunction]
#[pyo3(signature = (name, version, description, permissions=None, platforms=None))]
fn register_module(
    name: String,
    version: String,
    description: String,
    permissions: Option<Vec<String>>,
    platforms: Option<Vec<String>>,
) {
    python_registry::register_module(
        name,
        version,
        description,
        permissions.unwrap_or_default(),
        platforms.unwrap_or_default(),
    );
}

#[pyfunction]
#[pyo3(signature = (token, description, handler, permissions=None))]
fn register_command(
    token: String,
    description: String,
    handler: PyObject,
    permissions: Option<Vec<String>>,
) -> PyResult<()> {
    let token_err = token.clone();
    let scope_ext = token
        .split_once('.')
        .map(|(prefix, _)| prefix.to_string());
    let handler_fn: Arc<dyn Fn(Vec<String>) -> String + Send + Sync> =
        Arc::new(move |args: Vec<String>| {
            let _scope = scope_ext
                .as_ref()
                .map(|id| python_scope::PythonExtensionScope::enter(id.clone()));
            Python::with_gil(|py| {
                let py_list = PyList::new_bound(py, &args);
                match handler.call1(py, (py_list,)) {
                    Ok(result) => result
                        .extract::<String>(py)
                        .unwrap_or_else(|_| "(non-string return value)".to_string()),
                    Err(e) => format!("Python error in {token_err}: {e}"),
                }
            })
        });
    python_registry::register_command(
        token,
        description,
        handler_fn,
        permissions.unwrap_or_default(),
    );
    Ok(())
}

#[pyfunction]
#[pyo3(signature = (
    name,
    label,
    description,
    *,
    module = None,
    bg = None,
    surface = None,
    surface2 = None,
    text = None,
    dim = None,
    ui_font_family = None,
    border = None,
    accent = None,
    border_chars = None,
    border_horizontal_pattern = None,
    border_vertical_pattern = None,
    border_font_family = None,
    border_font_size_rems = None,
    border_side_rail_px = None,
    light_bg = None,
    light_surface = None,
    light_surface2 = None,
    light_text = None,
    light_dim = None,
    light_ui_font_family = None,
    light_border = None,
    light_accent = None,
    light_border_chars = None,
    light_border_horizontal_pattern = None,
    light_border_vertical_pattern = None,
    light_border_font_family = None,
    light_border_font_size_rems = None,
    light_border_side_rail_px = None,
    light_border_radius = None,
    border_radius = 0.0
))]
fn register_style(
    name: String,
    label: String,
    description: String,
    module: Option<String>,
    bg: Option<String>,
    surface: Option<String>,
    surface2: Option<String>,
    text: Option<String>,
    dim: Option<String>,
    ui_font_family: Option<String>,
    border: Option<String>,
    accent: Option<String>,
    border_chars: Option<String>,
    border_horizontal_pattern: Option<String>,
    border_vertical_pattern: Option<String>,
    border_font_family: Option<String>,
    border_font_size_rems: Option<f32>,
    border_side_rail_px: Option<f32>,
    light_bg: Option<String>,
    light_surface: Option<String>,
    light_surface2: Option<String>,
    light_text: Option<String>,
    light_dim: Option<String>,
    light_ui_font_family: Option<String>,
    light_border: Option<String>,
    light_accent: Option<String>,
    light_border_chars: Option<String>,
    light_border_horizontal_pattern: Option<String>,
    light_border_vertical_pattern: Option<String>,
    light_border_font_family: Option<String>,
    light_border_font_size_rems: Option<f32>,
    light_border_side_rail_px: Option<f32>,
    light_border_radius: Option<f32>,
    border_radius: f32,
) {
    let glyph = glyph_from_fields(
        bg,
        surface,
        surface2,
        text,
        dim,
        ui_font_family,
        border,
        accent,
        border_chars,
        border_horizontal_pattern,
        border_vertical_pattern,
        border_font_family,
        border_font_size_rems,
        border_side_rail_px,
        border_radius,
    );
    let glyph_light = glyph_from_fields(
        light_bg,
        light_surface,
        light_surface2,
        light_text,
        light_dim,
        light_ui_font_family,
        light_border,
        light_accent,
        light_border_chars,
        light_border_horizontal_pattern,
        light_border_vertical_pattern,
        light_border_font_family,
        light_border_font_size_rems,
        light_border_side_rail_px,
        light_border_radius.unwrap_or(border_radius),
    );

    python_registry::register_style(name, label, description, glyph, glyph_light, module);
}

fn py_default_to_string(obj: &Bound<'_, PyAny>) -> PyResult<String> {
    if let Ok(s) = obj.extract::<String>() {
        return Ok(s);
    }
    if let Ok(b) = obj.extract::<bool>() {
        return Ok(b.to_string());
    }
    if let Ok(i) = obj.extract::<i64>() {
        return Ok(i.to_string());
    }
    if let Ok(f) = obj.extract::<f64>() {
        return Ok(f.to_string());
    }
    Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
        "token default must be str, bool, int, or float",
    ))
}

fn parse_visibility_py(obj: &Bound<'_, PyAny>) -> PyResult<StyleTokenVisibility> {
    let d = obj.downcast::<PyDict>()?;
    if let Some(all_v) = d.get_item("all")? {
        let list = all_v.downcast::<PyList>()?;
        let mut children = Vec::new();
        for item in list.iter() {
            children.push(parse_visibility_py(&item)?);
        }
        return Ok(StyleTokenVisibility::All(children));
    }
    if let Some(any_v) = d.get_item("any")? {
        let list = any_v.downcast::<PyList>()?;
        let mut children = Vec::new();
        for item in list.iter() {
            children.push(parse_visibility_py(&item)?);
        }
        return Ok(StyleTokenVisibility::Any(children));
    }
    let ref_key: String = d
        .get_item("token")?
        .ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "visible_when: compare form needs 'token' key, or use 'all' / 'any' with a list",
            )
        })?
        .extract()?;
    const OPS: &[(&str, StyleTokenCompareOp)] = &[
        ("eq", StyleTokenCompareOp::Eq),
        ("ne", StyleTokenCompareOp::Ne),
        ("gt", StyleTokenCompareOp::Gt),
        ("gte", StyleTokenCompareOp::Ge),
        ("ge", StyleTokenCompareOp::Ge),
        ("lt", StyleTokenCompareOp::Lt),
        ("lte", StyleTokenCompareOp::Le),
        ("le", StyleTokenCompareOp::Le),
    ];
    for (name, op) in OPS {
        if let Some(v) = d.get_item(name)? {
            let literal = py_default_to_string(&v)?;
            return Ok(StyleTokenVisibility::Compare {
                token: ref_key,
                op: *op,
                literal,
            });
        }
    }
    Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
        "visible_when: compare needs one of eq, ne, gt, gte, ge, lt, lte, le (with 'token')",
    ))
}

#[pyfunction]
fn register_tokens(module: String, tokens: Bound<'_, PyAny>) -> PyResult<()> {
    let list = tokens.downcast::<PyList>()?;
    let mut out: Vec<StyleTokenSpec> = Vec::new();
    for item in list.iter() {
        let d = item.downcast::<PyDict>()?;
        let key: String = d
            .get_item("key")?
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("token missing key"))?
            .extract()?;
        let label: String = d
            .get_item("label")?
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("token missing label"))?
            .extract()?;
        let kind_str: String = d
            .get_item("kind")?
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("token missing kind"))?
            .extract()?;
        let kind = match kind_str.to_ascii_lowercase().as_str() {
            "color" => StyleTokenKind::Color,
            "float" => StyleTokenKind::Float,
            "string" => StyleTokenKind::String,
            "bool" | "boolean" => StyleTokenKind::Bool,
            "int" | "integer" => StyleTokenKind::Int,
            other => {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "unknown token kind: {other}"
                )));
            }
        };
        let default_obj = d.get_item("default")?.ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("token {key} missing default"))
        })?;
        let default_value = py_default_to_string(&default_obj)?;
        let visibility = match d.get_item("visible_when")? {
            Some(v) => Some(parse_visibility_py(&v)?),
            None => None,
        };
        let numeric = if matches!(kind, StyleTokenKind::Int | StyleTokenKind::Float) {
            let mut partial = StyleTokenNumericPartial::default();
            if let Some(m) = d.get_item("min")? {
                partial.min = Some(py_default_to_string(&m)?);
            }
            if let Some(m) = d.get_item("max")? {
                partial.max = Some(py_default_to_string(&m)?);
            }
            if let Some(gv) = d.get_item("granularity")? {
                let gs: String = gv.extract()?;
                partial.granularity = Some(
                    parse_granularity_str(&gs).map_err(|e| {
                        PyErr::new::<pyo3::exceptions::PyValueError, _>(e)
                    })?,
                );
            }
            merge_token_numeric_bounds(kind, &default_value, partial).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(e)
            })?
        } else {
            for nk in ["min", "max", "granularity"] {
                if d.get_item(nk)?.is_some() {
                    return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "token {key}: '{nk}' is only valid for int or float tokens"
                    )));
                }
            }
            None
        };
        out.push(StyleTokenSpec {
            key,
            label,
            kind,
            default_value,
            visibility,
            numeric,
        });
    }
    python_registry::register_tokens(module, out);
    Ok(())
}

#[pyfunction]
fn register_shortcut_json(extension_id: String, json_spec: String) -> PyResult<()> {
    let reg: ShortcutRegistrationOwned = serde_json::from_str(&json_spec)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
    let eff = reg
        .into_effective(&extension_id)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;
    python_registry::register_extension_shortcut(eff)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;
    Ok(())
}

#[pyfunction]
fn execute(py: Python<'_>, token: String, args: Vec<String>) -> String {
    let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let context = ExecutionContext {
        invoking_python_extension: python_scope::invoking_extension_id(),
        ..Default::default()
    };
    // Release GIL while calling into Rust core (may do I/O, subprocess, network).
    py.allow_threads(|| {
        match arcadia_core::modules::execute_command(&token, &args_refs, &context) {
            Ok(Some(result)) => result,
            Ok(None) => format!("Command not found: {token}"),
            Err(e) => format!("Error: {e}"),
        }
    })
}

/// Verify that `python:<extension_id>` has been granted `permission_id` and that the global
/// gate is on. Returns a `PermissionError` mapped to `PyPermissionError` when missing — the
/// extension is expected to declare the permission in `register_module(permissions=[...])`
/// so the first-enable flow asks the user to accept.
fn ensure_python_permission(extension_id: &str, permission_id: &str) -> PyResult<()> {
    let cfg = PermissionsConfig::load_or_create()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
    let subj = PermissionSubject::python(extension_id.to_string());
    if cfg.effective_allowed(&subj, permission_id) {
        Ok(())
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyPermissionError, _>(
            format!("Permission denied: {permission_id} for python:{extension_id}"),
        ))
    }
}

#[pyfunction]
#[pyo3(signature = (extension_id, label, tooltip=None, instance=None))]
fn tray_register(
    extension_id: String,
    label: String,
    tooltip: Option<String>,
    instance: Option<String>,
) -> PyResult<String> {
    ensure_python_permission(&extension_id, "tray.create")?;
    let owner = match instance.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(key) => format!("python:{extension_id}::{key}"),
        None => format!("python:{extension_id}"),
    };
    let id = core_tray::register_item(owner, label);
    if let Some(text) = tooltip {
        core_tray::set_tooltip(&id, text)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;
    }
    Ok(id)
}

#[pyfunction]
fn tray_set_image(
    extension_id: String,
    tray_id: String,
    rgba: &Bound<'_, PyBytes>,
    width: u32,
    height: u32,
) -> PyResult<()> {
    ensure_python_permission(&extension_id, "tray.create")?;
    let bytes = rgba.as_bytes().to_vec();
    core_tray::set_image(&tray_id, bytes, width, height)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))
}

#[pyfunction]
fn tray_set_tooltip(extension_id: String, tray_id: String, text: String) -> PyResult<()> {
    ensure_python_permission(&extension_id, "tray.create")?;
    core_tray::set_tooltip(&tray_id, text)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))
}

#[pyfunction]
fn tray_set_menu(extension_id: String, tray_id: String, items: Bound<'_, PyList>) -> PyResult<()> {
    ensure_python_permission(&extension_id, "tray.create")?;
    let mut menu: Vec<core_tray::TrayMenuItem> = Vec::new();
    for entry in items.iter() {
        let d = entry.downcast::<PyDict>()?;

        let is_sep = d
            .get_item("separator")?
            .and_then(|v| v.extract::<bool>().ok())
            .unwrap_or(false);
        if is_sep {
            menu.push(core_tray::TrayMenuItem {
                label: String::new(),
                command_token: String::new(),
                args: Vec::new(),
            });
            continue;
        }

        let label: String = d
            .get_item("label")?
            .ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>("menu item missing 'label'")
            })?
            .extract()?;
        let command_token: String = d
            .get_item("command")?
            .ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>("menu item missing 'command'")
            })?
            .extract()?;
        let args: Vec<String> = match d.get_item("args")? {
            Some(v) => v.extract().unwrap_or_default(),
            None => Vec::new(),
        };
        menu.push(core_tray::TrayMenuItem {
            label,
            command_token,
            args,
        });
    }
    core_tray::set_menu(&tray_id, menu)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))
}

#[pyfunction]
fn tray_set_show_menu_on_left_click(
    extension_id: String,
    tray_id: String,
    enable: bool,
) -> PyResult<()> {
    ensure_python_permission(&extension_id, "tray.create")?;
    core_tray::set_show_menu_on_left_click(&tray_id, enable)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))
}

#[pyfunction]
fn register_tray_icon_click_handler(
    extension_id: String,
    handler: PyObject,
) -> PyResult<()> {
    ensure_python_permission(&extension_id, "tray.create")?;
    let ext_err = extension_id.clone();
    let scope_id = extension_id.clone();
    let handler = Arc::new(handler);
    let cb: Arc<dyn Fn(Vec<String>) + Send + Sync> = Arc::new(move |args: Vec<String>| {
        let _scope = python_scope::PythonExtensionScope::enter(scope_id.clone());
        let h = Arc::clone(&handler);
        Python::with_gil(|py| {
            let tray_id = args.get(0).cloned().unwrap_or_default();
            let button = args.get(1).cloned().unwrap_or_default();
            if let Err(e) = h.call1(py, (tray_id, button)) {
                eprintln!("tray icon click handler ({ext_err}): {e}");
            }
        });
    });
    python_registry::register_tray_icon_click_handler(extension_id, cb);
    Ok(())
}

#[pyfunction]
fn tray_remove(extension_id: String, tray_id: String) -> PyResult<()> {
    ensure_python_permission(&extension_id, "tray.create")?;
    core_tray::remove_item(&tray_id).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))
}

/// Returns `(x, y, width, height, pixels_per_point)` for the tray item, or `None`.
/// See `arcadia_core::modules::tray::icon_screen_bounds` for coordinate semantics.
#[pyfunction]
fn tray_icon_screen_bounds(
    extension_id: String,
    tray_item_id: String,
) -> PyResult<Option<(f64, f64, f64, f64, f64)>> {
    ensure_python_permission(&extension_id, "tray.create")?;
    Ok(core_tray::icon_screen_bounds(&tray_item_id))
}

#[pyfunction]
fn cursor_position(extension_id: String) -> PyResult<Option<(f64, f64)>> {
    ensure_python_permission(&extension_id, "cursor.global_position")?;
    Ok(core_cursor::position().map(|p| (p.x, p.y)))
}

/// `(x, y, left_down, right_down)` in global screen space; `None` if cursor backend unavailable.
#[pyfunction]
fn cursor_snapshot(extension_id: String) -> PyResult<Option<(f64, f64, bool, bool)>> {
    ensure_python_permission(&extension_id, "cursor.global_mouse_buttons")?;
    Ok(core_cursor::snapshot().map(|s| (s.x, s.y, s.left_button, s.right_button)))
}

#[pyfunction]
fn screen_size(extension_id: String) -> PyResult<Option<(u32, u32)>> {
    ensure_python_permission(&extension_id, "cursor.global_position")?;
    Ok(core_cursor::primary_screen_size().map(|s| (s.width, s.height)))
}

/// Primary display size for layout — gated on `overlay.hud` (no cursor permission required).
#[pyfunction]
fn overlay_display_size(extension_id: String) -> PyResult<Option<(u32, u32)>> {
    ensure_python_permission(&extension_id, "overlay.hud")?;
    Ok(core_cursor::primary_screen_size().map(|s| (s.width, s.height)))
}

#[pyfunction]
#[pyo3(signature = (extension_id, rgba, width, height, pad_right=None, pad_bottom=None, display_width=None, display_height=None))]
fn overlay_hud_set_sprite(
    extension_id: String,
    rgba: &Bound<'_, PyBytes>,
    width: u32,
    height: u32,
    pad_right: Option<f32>,
    pad_bottom: Option<f32>,
    display_width: Option<f32>,
    display_height: Option<f32>,
) -> PyResult<()> {
    ensure_python_permission(&extension_id, "overlay.hud")?;
    let payload = overlay_hud_sprite::OverlayHudSpritePayload {
        rgba: rgba.as_bytes().to_vec(),
        width,
        height,
        pad_right: pad_right.unwrap_or(24.),
        pad_bottom: pad_bottom.unwrap_or(24.),
        display_width,
        display_height,
    };
    overlay_hud_sprite::set_sprite(payload).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(e)
    })
}

#[pyfunction]
fn overlay_hud_clear_sprite(extension_id: String) -> PyResult<()> {
    ensure_python_permission(&extension_id, "overlay.hud")?;
    overlay_hud_sprite::clear_sprite();
    Ok(())
}

/// Start a tween. `callback(t: float)` is called on the Python timer lane every ~16 ms with
/// eased `t ∈ [0.0, 1.0]`. Returns a tween id that can be passed to `cancel_animation`.
///
/// `easing` is one of: `"linear"`, `"ease_out_cubic"`, `"ease_in_cubic"`,
/// `"ease_in_out_cubic"`, `"ease_out_elastic"`.
#[pyfunction]
#[pyo3(signature = (extension_id, duration_ms, easing, callback))]
fn animate(
    extension_id: String,
    duration_ms: u64,
    easing: String,
    callback: PyObject,
) -> PyResult<u64> {
    let easing_val = animation::Easing::from_str(&easing).ok_or_else(|| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Unknown easing '{}'. Choose from: linear, ease_out_cubic, ease_in_cubic, ease_in_out_cubic, ease_out_elastic",
            easing
        ))
    })?;
    let ext_id = extension_id.clone();
    let callback = Arc::new(callback);
    let id = animation::tween_with_completion(
        duration_ms,
        easing_val,
        move |t| {
            if !python_registry::extension_enabled(&ext_id) {
                return;
            }
            let _scope = python_scope::PythonExtensionScope::enter(ext_id.clone());
            let cb = Arc::clone(&callback);
            Python::with_gil(|py| {
                if let Err(e) = cb.call1(py, (t,)) {
                    eprintln!("animate({ext_id}) callback error: {e}");
                }
            });
        },
        None,
        Some(extension_id),
    );
    Ok(id.0)
}

#[pyfunction]
fn cancel_animation(tween_id: u64) {
    animation::cancel(animation::TweenId(tween_id));
}

#[pyfunction]
fn set_timer(extension_id: String, interval_ms: u64, callback: PyObject) -> PyResult<u64> {
    if interval_ms == 0 {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "interval_ms must be > 0",
        ));
    }
    let ext_id = extension_id.clone();
    let callback = Arc::new(callback);
    let task = scheduling::spawn_interval_on_lane(
        scheduling::TaskLane::PythonTimer,
        Duration::from_millis(interval_ms),
        move || {
            // Skip ticks for disabled or unloaded extensions to avoid stale callbacks chewing CPU
            // after the user disables a Python module without restarting.
            if !python_registry::extension_enabled(&ext_id) {
                return;
            }
            let _scope = python_scope::PythonExtensionScope::enter(ext_id.clone());
            let cb = Arc::clone(&callback);
            Python::with_gil(|py| {
                if let Err(e) = cb.call0(py) {
                    eprintln!("set_timer({ext_id}) callback error: {e}");
                }
            });
        },
    );
    Ok(task)
}

#[pyfunction]
fn cancel_timer(task_id: u64) {
    scheduling::cancel(task_id);
}

/// Absolute path to this extension's `Assets` directory (`<bundle>/Assets`), or `None` if the
/// extension has no on-disk entry (e.g. not discovered yet).
#[pyfunction]
fn extension_assets_path(extension_id: String) -> Option<String> {
    python_registry::extension_assets_dir(&extension_id).map(|p| p.to_string_lossy().into_owned())
}

/// Read a file from `<bundle>/Assets/<relative>`. `relative` must not be absolute or contain `..`.
#[pyfunction]
fn read_extension_asset<'py>(
    py: Python<'py>,
    extension_id: String,
    relative: String,
) -> PyResult<Bound<'py, PyBytes>> {
    let path = python_registry::resolve_extension_asset_path(&extension_id, &relative)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;
    let bytes = std::fs::read(&path).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyOSError, _>(format!("{}: {e}", path.display()))
    })?;
    Ok(PyBytes::new_bound(py, &bytes))
}

/// Read persisted token values declared with `register_tokens` for an extension.
/// Returns a dict mapping each token key to its current value (merged with the default).
/// Values are strings; the extension parses them to int/float/bool as appropriate using
/// the token kind it declared.
#[pyfunction]
fn read_tokens<'py>(py: Python<'py>, module: String) -> PyResult<Bound<'py, PyDict>> {
    let file = extension_tokens::load_module_tokens(&module).unwrap_or_default();
    let specs = python_registry::list_style_tokens()
        .into_iter()
        .find(|(m, _)| m == &module)
        .map(|(_, s)| s)
        .unwrap_or_default();
    let result = PyDict::new_bound(py);
    for spec in specs {
        let display =
            extension_tokens::merged_display_for_key(&spec.key, &spec.default_value, &file);
        result.set_item(spec.key, display)?;
    }
    Ok(result)
}

/// Declare that this extension's tokens should appear in the Editor settings panel
/// rather than as a standalone settings page in the sidebar.
#[pyfunction]
fn register_editor_token_module(extension_id: String) {
    python_registry::register_editor_token_module(extension_id);
}

/// Register a syntax highlight provider for `language` (e.g. `"rust"`).
///
/// `handler(text: str) -> list[tuple[int, int, str]]` — returns (start_byte, end_byte, token_name)
/// tuples covering the document. Overlapping spans are not defined; non-overlapping and ordered
/// is simplest. The editor calls this once per render cycle for the active tab.
#[pyfunction]
fn register_highlight_provider(
    extension_id: String,
    language: String,
    handler: PyObject,
) {
    let handler = Arc::new(handler);
    python_registry::register_highlight_provider(
        language,
        extension_id,
        Arc::new(move |text: &str| {
            let text = text.to_string();
            let h = Arc::clone(&handler);
            Python::with_gil(|py| -> Vec<python_registry::HighlightSpan> {
                let result = match h.call1(py, (&text,)) {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("highlight_provider error: {e}");
                        return Vec::new();
                    }
                };
                let Ok(list) = result.downcast_bound::<pyo3::types::PyList>(py) else {
                    return Vec::new();
                };
                list.iter()
                    .filter_map(|item| {
                        let tup = item.downcast::<pyo3::types::PyTuple>().ok()?;
                        let start: usize = tup.get_item(0).ok()?.extract().ok()?;
                        let end: usize = tup.get_item(1).ok()?.extract().ok()?;
                        let token: String = tup.get_item(2).ok()?.extract().ok()?;
                        Some(python_registry::HighlightSpan { start, end, token })
                    })
                    .collect()
            })
        }),
    );
}

/// Register a decoration provider that adds coloured boxes behind each line.
///
/// `handler(line: str, line_idx: int) -> list[dict]` — each dict must contain:
/// `col_start` (int), `col_width` (int), `r` (int 0-255), `g`, `b`, `a`.
/// Multiple decoration providers can coexist (one per extension).
#[pyfunction]
fn register_decoration_provider(extension_id: String, handler: PyObject) {
    let handler = Arc::new(handler);
    python_registry::register_decoration_provider(
        extension_id,
        Arc::new(move |line: &str, line_idx: usize| {
            let line = line.to_string();
            let h = Arc::clone(&handler);
            Python::with_gil(|py| -> Vec<python_registry::DecorationRect> {
                let result = match h.call1(py, (&line, line_idx)) {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("decoration_provider error: {e}");
                        return Vec::new();
                    }
                };
                let Ok(list) = result.downcast_bound::<pyo3::types::PyList>(py) else {
                    return Vec::new();
                };
                list.iter()
                    .filter_map(|item| {
                        let d = item.downcast::<PyDict>().ok()?;
                        let col_start: usize = d.get_item("col_start").ok()??.extract().ok()?;
                        let col_width: usize = d.get_item("col_width").ok()??.extract().ok()?;
                        let r: u8 = d.get_item("r").ok()??.extract().ok()?;
                        let g: u8 = d.get_item("g").ok()??.extract().ok()?;
                        let b: u8 = d.get_item("b").ok()??.extract().ok()?;
                        let a: u8 = d.get_item("a").ok()??.extract().ok()?;
                        Some(python_registry::DecorationRect { col_start, col_width, r, g, b, a })
                    })
                    .collect()
            })
        }),
    );
}

#[pymodule]
pub fn arcadia(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(register_module, m)?)?;
    m.add_function(wrap_pyfunction!(register_command, m)?)?;
    m.add_function(wrap_pyfunction!(register_style, m)?)?;
    m.add_function(wrap_pyfunction!(register_tokens, m)?)?;
    m.add_function(wrap_pyfunction!(register_shortcut_json, m)?)?;
    m.add_function(wrap_pyfunction!(execute, m)?)?;
    m.add_function(wrap_pyfunction!(tray_register, m)?)?;
    m.add_function(wrap_pyfunction!(tray_set_image, m)?)?;
    m.add_function(wrap_pyfunction!(tray_set_tooltip, m)?)?;
    m.add_function(wrap_pyfunction!(tray_set_menu, m)?)?;
    m.add_function(wrap_pyfunction!(tray_set_show_menu_on_left_click, m)?)?;
    m.add_function(wrap_pyfunction!(register_tray_icon_click_handler, m)?)?;
    m.add_function(wrap_pyfunction!(tray_remove, m)?)?;
    m.add_function(wrap_pyfunction!(tray_icon_screen_bounds, m)?)?;
    m.add_function(wrap_pyfunction!(cursor_position, m)?)?;
    m.add_function(wrap_pyfunction!(cursor_snapshot, m)?)?;
    m.add_function(wrap_pyfunction!(screen_size, m)?)?;
    m.add_function(wrap_pyfunction!(overlay_display_size, m)?)?;
    m.add_function(wrap_pyfunction!(overlay_hud_set_sprite, m)?)?;
    m.add_function(wrap_pyfunction!(overlay_hud_clear_sprite, m)?)?;
    m.add_function(wrap_pyfunction!(animate, m)?)?;
    m.add_function(wrap_pyfunction!(cancel_animation, m)?)?;
    m.add_function(wrap_pyfunction!(set_timer, m)?)?;
    m.add_function(wrap_pyfunction!(cancel_timer, m)?)?;
    m.add_function(wrap_pyfunction!(read_tokens, m)?)?;
    m.add_function(wrap_pyfunction!(extension_assets_path, m)?)?;
    m.add_function(wrap_pyfunction!(read_extension_asset, m)?)?;
    m.add_function(wrap_pyfunction!(register_editor_token_module, m)?)?;
    m.add_function(wrap_pyfunction!(register_highlight_provider, m)?)?;
    m.add_function(wrap_pyfunction!(register_decoration_provider, m)?)?;
    Ok(())
}
