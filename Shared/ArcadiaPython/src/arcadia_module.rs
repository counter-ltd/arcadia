use arcadia_core::modules::{python_registry, ExecutionContext};
use pyo3::prelude::*;
use pyo3::types::PyList;
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
    m.add_function(wrap_pyfunction!(execute, m)?)?;
    Ok(())
}
