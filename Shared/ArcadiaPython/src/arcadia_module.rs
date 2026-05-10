use arcadia_core::modules::{python_registry, ExecutionContext};
use arcadia_core::modules::python_registry::{StyleTokenKind, StyleTokenSpec};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::sync::Arc;

#[pyfunction]
fn register_module(name: String, version: String, description: String) {
    python_registry::register_module(name, version, description);
}

#[pyfunction]
fn register_command(
    token: String,
    description: String,
    handler: PyObject,
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
    python_registry::register_command(token, description, handler_fn);
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
    border = None,
    accent = None,
    border_chars = None,
    border_horizontal_pattern = None,
    border_vertical_pattern = None,
    border_font_family = None,
    border_font_size_rems = None,
    border_side_rail_px = None,
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
    border: Option<String>,
    accent: Option<String>,
    border_chars: Option<String>,
    border_horizontal_pattern: Option<String>,
    border_vertical_pattern: Option<String>,
    border_font_family: Option<String>,
    border_font_size_rems: Option<f32>,
    border_side_rail_px: Option<f32>,
    border_radius: f32,
) {
    let has_glyph = bg.is_some()
        || surface.is_some()
        || surface2.is_some()
        || text.is_some()
        || dim.is_some()
        || border.is_some()
        || accent.is_some()
        || border_chars.is_some()
        || border_horizontal_pattern.is_some()
        || border_vertical_pattern.is_some()
        || border_font_family.is_some()
        || border_font_size_rems.is_some()
        || border_side_rail_px.is_some();

    let glyph = if has_glyph {
        Some(python_registry::GlyphParams {
            bg,
            surface,
            surface2,
            text,
            dim,
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
    };

    python_registry::register_style(name, label, description, glyph, module);
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

#[pymodule]
pub fn arcadia(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(register_module, m)?)?;
    m.add_function(wrap_pyfunction!(register_command, m)?)?;
    m.add_function(wrap_pyfunction!(register_style, m)?)?;
    m.add_function(wrap_pyfunction!(register_tokens, m)?)?;
    m.add_function(wrap_pyfunction!(execute, m)?)?;
    Ok(())
}
