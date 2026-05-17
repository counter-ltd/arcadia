mod arcadia_module;
mod python_scope;
use arcadia_module::arcadia as arcadia_pymodule;

use arcadia_core::config::modules::ModulesConfig;
use arcadia_core::config::ConfigFile;
use arcadia_core::modules::python_host;
use arcadia_core::modules::python_registry;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

static PYTHON_INITIALIZED: OnceLock<()> = OnceLock::new();

/// One on-disk extension found in `~/Arcadia/Extensions/`. `id` is derived from the file
/// name / directory and used as the persistence key in `ModulesConfig.extension_state`.
///
/// Bundle layout: `Extensions/<folder>/main.py` with static files under
/// `Extensions/<folder>/Assets/` (see `arcadia.extension_assets_path` / `read_extension_asset`).
#[derive(Clone, Debug)]
struct DiscoveredExtension {
    id: String,
    path: PathBuf,
}

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
            sync_extensions(&dir)
        }));
        // Loader for on-demand `python-host.extension-enable` once the user opts in via
        // the Extensions settings page — runs a single file body without re-scanning, and
        // collapses any stub-id / declared-name mismatch onto the body's canonical name.
        python_registry::set_load_one_handler(Arc::new(
            |stub_id: String, path: PathBuf| -> Result<String, String> {
                load_and_merge(&stub_id, &path)
            },
        ));

        sync_extensions(&extensions_dir)
    }
}

/// Discover all extensions on disk, register stubs in the python registry, and execute the
/// body of those marked enabled in `ModulesConfig.extension_state`. Extensions with no
/// recorded state (i.e. dropped in for the first time) start **disabled**: the user must
/// explicitly opt in from the Extensions settings page before the body runs. This is what
/// stops side-effectful scripts (tray creation, OS Accessibility prompts via cursor backend,
/// etc.) from firing on launch for extensions the user never enabled.
fn sync_extensions(dir: &Path) -> Result<(), String> {
    let discovered = scan_extensions(dir);
    let cfg = ModulesConfig::load_or_create().map_err(|e| e.to_string())?;

    let mut errors = Vec::new();
    let mut renames: Vec<(String, String)> = Vec::new();
    for ext in &discovered {
        let persisted_enabled = cfg.python_extension_enabled(&ext.id);
        // Pre-parse declared permissions from the source so the Extensions settings page can
        // show the first-enable permission modal *before* we run a body that would otherwise
        // immediately call a permission-protected API (e.g. `arcadia.tray_register`) and
        // crash mid-load.
        let declared_permissions = parse_declared_permissions(&ext.path).unwrap_or_default();
        let declared_platforms = parse_declared_platforms(&ext.path).unwrap_or_default();
        python_registry::register_discovered(
            ext.id.clone(),
            ext.path.clone(),
            persisted_enabled,
            declared_permissions,
            declared_platforms,
        );
        if persisted_enabled {
            if python_registry::extension_body_loaded(&ext.id) {
                continue;
            }
            if !python_registry::extension_supported_at_runtime(&ext.id) {
                continue;
            }
            match load_and_merge(&ext.id, &ext.path) {
                Ok(canonical) if canonical != ext.id => {
                    renames.push((ext.id.clone(), canonical));
                }
                Ok(_) => {}
                Err(e) => {
                    eprintln!("arcadia-python: {e}");
                    errors.push(e);
                }
            }
        }
    }

    // Persist any folder-id ↔ declared-name renames discovered during this sync so the
    // user's `extension_state[<old_id>] = true` doesn't get re-evaluated as the old id
    // on next launch. `set_python_extension_enabled` is additive; if both entries exist we
    // keep the new one and drop the old.
    if !renames.is_empty() {
        if let Ok(mut new_cfg) = ModulesConfig::load_or_create() {
            let mut dirty = false;
            for (old_id, new_id) in &renames {
                if new_cfg.extension_state.remove(old_id).is_some() {
                    dirty = true;
                }
                if !new_cfg.extension_state.contains_key(new_id) {
                    new_cfg.set_python_extension_enabled(new_id, true);
                    dirty = true;
                }
            }
            if dirty {
                let _ = new_cfg.save();
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

fn scan_extensions(dir: &Path) -> Vec<DiscoveredExtension> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |e| e == "py") {
            if let Some(id) = extension_id_from_path(&path, false) {
                out.push(DiscoveredExtension { id, path });
            }
        } else if path.is_dir() {
            let main = path.join("main.py");
            if main.exists() {
                if let Some(id) = extension_id_from_path(&path, true) {
                    out.push(DiscoveredExtension { id, path: main });
                }
            }
        }
    }
    out
}

/// Map a discovered path to its stable extension id. For directory-style extensions we use
/// the directory name (`googly_eyes`); for single-file ones we use the file stem. We then
/// kebab-case the result so it matches the conventional `register_module(name="…")` value
/// used by extension authors.
fn extension_id_from_path(path: &Path, is_dir: bool) -> Option<String> {
    let raw = if is_dir {
        path.file_name()?.to_string_lossy().into_owned()
    } else {
        path.file_stem()?.to_string_lossy().into_owned()
    };
    let id = raw.replace('_', "-").trim().to_lowercase();
    if id.is_empty() {
        None
    } else {
        Some(id)
    }
}

/// Naive scanner for `permissions=[ "a", "b", … ]` inside the first `register_module(...)` call
/// in a Python extension source file. This is a stand-in for proper static analysis — it is
/// good enough for the conventional declarative form extensions use (see `Extensions/*/main.py`)
/// and lets the host populate `PythonModuleInfo.required_permissions` before the body runs.
///
/// If the source ever uses a non-literal permission list (computed at runtime, etc.) the body
/// itself is still the ground truth — the parsed value is only used to seed the stub.
fn parse_declared_permissions(path: &Path) -> Option<Vec<String>> {
    let code = std::fs::read_to_string(path).ok()?;
    // Anchor on `register_module(` so we don't accidentally pick up a `permissions=` on an
    // unrelated call later in the file.
    let anchor = code.find("register_module(")?;
    let after_anchor = &code[anchor..];
    // No `permissions=` kwarg → the extension declared zero permissions; return an empty
    // list (still `Some`) so callers can distinguish "declared nothing" from "file failed
    // to parse".
    let Some(perm_kw) = after_anchor.find("permissions") else {
        return Some(Vec::new());
    };
    let after_kw = &after_anchor[perm_kw..];
    let bracket_open = after_kw.find('[')?;
    let bracket_close = after_kw[bracket_open..].find(']')?;
    let inner = &after_kw[bracket_open + 1..bracket_open + bracket_close];

    let mut out = Vec::new();
    let mut chars = inner.char_indices();
    while let Some((i, c)) = chars.next() {
        if c == '"' || c == '\'' {
            let quote = c;
            let start = i + 1;
            let mut end = start;
            let mut closed = false;
            for (j, cc) in chars.by_ref() {
                if cc == quote {
                    end = j;
                    closed = true;
                    break;
                }
            }
            if closed && end > start {
                out.push(inner[start..end].to_string());
            }
        }
    }
    Some(out)
}

/// Naive scanner for `platforms=[ "macos", … ]` inside the first `register_module(...)` call.
/// Same limitations as [`parse_declared_permissions`]. Use lowercase ids matching
/// `arcadia_core::platform::current().name()`.
fn parse_declared_platforms(path: &Path) -> Option<Vec<String>> {
    let code = std::fs::read_to_string(path).ok()?;
    let anchor = code.find("register_module(")?;
    let after_anchor = &code[anchor..];
    let Some(kw) = after_anchor.find("platforms") else {
        return Some(Vec::new());
    };
    let after_kw = &after_anchor[kw..];
    let bracket_open = after_kw.find('[')?;
    let bracket_close = after_kw[bracket_open..].find(']')?;
    let inner = &after_kw[bracket_open + 1..bracket_open + bracket_close];

    let mut out = Vec::new();
    let mut chars = inner.char_indices();
    while let Some((i, c)) = chars.next() {
        if c == '"' || c == '\'' {
            let quote = c;
            let start = i + 1;
            let mut end = start;
            let mut closed = false;
            for (j, cc) in chars.by_ref() {
                if cc == quote {
                    end = j;
                    closed = true;
                    break;
                }
            }
            if closed && end > start {
                out.push(inner[start..end].to_string());
            }
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;

    fn write_tmp(name: &str, contents: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("arcadia-python-tests");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join(name);
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(contents.as_bytes()).unwrap();
        path
    }

    #[test]
    fn parse_declared_permissions_multiline_register_module() {
        let path = write_tmp(
            "googly_like.py",
            r#"import arcadia

arcadia.register_module(
    name="googly-eyes",
    version="0.1.0",
    description="...",
    permissions=[
        "tray.create",
        "cursor.global_position",
        "cursor.global_mouse_buttons",
    ],
)
"#,
        );
        let perms = parse_declared_permissions(&path).unwrap();
        assert_eq!(
            perms,
            vec![
                "tray.create",
                "cursor.global_position",
                "cursor.global_mouse_buttons",
            ]
        );
    }

    #[test]
    fn parse_declared_permissions_no_permissions_kwarg() {
        let path = write_tmp(
            "hello_like.py",
            r#"import arcadia
arcadia.register_module(name="hello", version="0.1.0", description="...")
"#,
        );
        let perms = parse_declared_permissions(&path).unwrap();
        assert!(perms.is_empty());
    }

    #[test]
    fn parse_declared_permissions_single_quoted_entries() {
        let path = write_tmp(
            "single_quote.py",
            r#"arcadia.register_module(name='x', version='1', description='y', permissions=['a.b', 'c.d'])
"#,
        );
        let perms = parse_declared_permissions(&path).unwrap();
        assert_eq!(perms, vec!["a.b", "c.d"]);
    }

    #[test]
    fn parse_declared_platforms_kwarg() {
        let path = write_tmp(
            "plat_ext.py",
            r#"import arcadia
arcadia.register_module(
    name="desk-only",
    version="0.1.0",
    description="x",
    platforms=["macos", "linux"],
)
"#,
        );
        let plats = parse_declared_platforms(&path).unwrap();
        assert_eq!(plats, vec!["macos", "linux"]);
    }

    #[test]
    fn parse_declared_platforms_missing_kwarg() {
        let path = write_tmp(
            "no_plat.py",
            r#"arcadia.register_module(name="x", version="1", description="y")
"#,
        );
        let plats = parse_declared_platforms(&path).unwrap();
        assert!(plats.is_empty());
    }

    #[test]
    fn extension_id_from_directory_kebab_cases_underscores() {
        let path = PathBuf::from("/tmp/Arcadia/Extensions/googly_eyes");
        let id = extension_id_from_path(&path, true).unwrap();
        assert_eq!(id, "googly-eyes");
    }

    #[test]
    fn extension_id_from_single_file_uses_stem() {
        let path = PathBuf::from("/tmp/Arcadia/Extensions/Hello_World.py");
        let id = extension_id_from_path(&path, false).unwrap();
        assert_eq!(id, "hello-world");
    }
}

/// Run a single extension body and reconcile the canonical name in the python registry.
///
/// Folder-discovered ids (e.g. the kebab-cased directory `terminal_theme` → `terminal-theme`)
/// don't always match what the body itself declares via `register_module(name="…")`. Without
/// reconciliation the user sees a duplicate row: the stub under the folder id plus a fully
/// loaded entry under the declared name. This snapshots the registry before/after the body
/// runs, and if the body introduced exactly one new module whose name differs from the stub
/// id, the stub is removed and its on-disk path is transferred to the new entry so future
/// `python-host.extension-enable` / disable toggles still find the source.
///
/// Returns `Ok(canonical_name)` on success (equal to `stub_id` when no merge happened) or
/// `Err(message)` when the body failed to load.
fn load_and_merge(stub_id: &str, path: &Path) -> Result<String, String> {
    let before: std::collections::HashSet<String> =
        python_registry::module_names().into_iter().collect();
    load_extension(path, stub_id)?;
    let after: Vec<String> = python_registry::module_names();
    let new_modules: Vec<String> = after.into_iter().filter(|n| !before.contains(n)).collect();
    let canonical = if new_modules.len() == 1 && new_modules[0] != stub_id {
        let canonical = new_modules.into_iter().next().unwrap();
        python_registry::attach_path(&canonical, path.to_path_buf());
        python_registry::remove_module(stub_id);
        canonical
    } else {
        stub_id.to_string()
    };
    python_host::ensure_native_companions_for_loaded_extension(&canonical);
    Ok(canonical)
}

fn load_extension(path: &Path, stub_id: &str) -> Result<(), String> {
    let code = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read {}: {e}", path.display()))?;

    let _scope = python_scope::PythonExtensionScope::enter(stub_id.to_string());

    Python::with_gil(|py| -> PyResult<()> {
        let globals = PyDict::new_bound(py);
        let builtins = py.import_bound("builtins")?;
        globals.set_item("__builtins__", builtins)?;
        py.run_bound(&code, Some(&globals), None)?;
        Ok(())
    })
    .map_err(|e| format!("Error loading {}: {e}", path.display()))
}
