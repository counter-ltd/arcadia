use arcadia_core::config::extension_tokens;
use arcadia_core::config::permissions::{PermissionSubject, PermissionsConfig};
use arcadia_core::config::ConfigFile;
use arcadia_core::modules::python_registry::{StyleTokenKind, StyleTokenSpec};
use arcadia_core::modules::{
    cursor as core_cursor, python_registry, tray as core_tray, ExecutionContext,
};
use arcadia_core::scheduling;
use arcadia_core::shortcuts::ShortcutRegistrationOwned;
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
    let handler_fn: Arc<dyn Fn(Vec<String>) -> String + Send + Sync> =
        Arc::new(move |args: Vec<String>| {
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
        out.push(StyleTokenSpec {
            key,
            label,
            kind,
            default_value,
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
    let context = ExecutionContext::default();
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
#[pyo3(signature = (extension_id, label, tooltip=None))]
fn tray_register(extension_id: String, label: String, tooltip: Option<String>) -> PyResult<String> {
    ensure_python_permission(&extension_id, "tray.create")?;
    let id = core_tray::register_item(format!("python:{extension_id}"), label);
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

#[pyfunction]
fn screen_size(extension_id: String) -> PyResult<Option<(u32, u32)>> {
    ensure_python_permission(&extension_id, "cursor.global_position")?;
    Ok(core_cursor::primary_screen_size().map(|s| (s.width, s.height)))
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
    m.add_function(wrap_pyfunction!(tray_remove, m)?)?;
    m.add_function(wrap_pyfunction!(tray_icon_screen_bounds, m)?)?;
    m.add_function(wrap_pyfunction!(cursor_position, m)?)?;
    m.add_function(wrap_pyfunction!(screen_size, m)?)?;
    m.add_function(wrap_pyfunction!(set_timer, m)?)?;
    m.add_function(wrap_pyfunction!(cancel_timer, m)?)?;
    m.add_function(wrap_pyfunction!(read_tokens, m)?)?;
    Ok(())
}
