mod arcadia_module;
use arcadia_module::arcadia as arcadia_pymodule;

use arcadia_core::modules::python_registry;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

static PYTHON_INITIALIZED: OnceLock<()> = OnceLock::new();

pub struct PythonExtensionHost;

impl PythonExtensionHost {
    pub fn start(extensions_dir: PathBuf) -> Result<(), String> {
        PYTHON_INITIALIZED.get_or_init(|| {
            pyo3::append_to_inittab!(arcadia_pymodule);
            pyo3::prepare_freethreaded_python();
        });

        let dir = extensions_dir.clone();
        python_registry::set_reload_handler(Arc::new(move || {
            python_registry::clear();
            reload_extensions(&dir)
        }));

        reload_extensions(&extensions_dir)
    }
}

fn reload_extensions(dir: &Path) -> Result<(), String> {
    let paths = scan_extensions(dir);
    let mut errors = Vec::new();
    for path in &paths {
        if let Err(e) = load_extension(path) {
            eprintln!("arcadia-python: {e}");
            errors.push(e);
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

fn scan_extensions(dir: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else { return paths };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |e| e == "py") {
            paths.push(path);
        } else if path.is_dir() {
            let main = path.join("main.py");
            if main.exists() {
                paths.push(main);
            }
        }
    }
    paths
}

fn load_extension(path: &Path) -> Result<(), String> {
    let code = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read {}: {e}", path.display()))?;

    Python::with_gil(|py| -> PyResult<()> {
        let globals = PyDict::new_bound(py);
        let builtins = py.import_bound("builtins")?;
        globals.set_item("__builtins__", builtins)?;
        py.run_bound(&code, Some(&globals), None)?;
        Ok(())
    })
    .map_err(|e| format!("Error loading {}: {e}", path.display()))
}
