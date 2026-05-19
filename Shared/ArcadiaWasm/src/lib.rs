//! WASM module host — embeds `wasmi`, loads `.wasm` modules from `~/Arcadia/Modules/`, and
//! bridges them into `arcadia-core` via `wasm_registry`.
//!
//! Mirrors the Python host: discovery registers disabled stubs; the user opts in; the module
//! is instantiated on demand and its commands dispatch through `arcadia_core::modules`.

mod instance;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use arcadia_core::config::modules::ModulesConfig;
use arcadia_core::config::ConfigFile;
use arcadia_core::extension::wasm_manifest;
use arcadia_core::modules::wasm_registry;

use instance::LoadedModule;

/// Start the WASM host: install the registry callbacks and run an initial discovery scan.
/// `modules_dir` is `~/Arcadia/Modules/` (resolved by the caller).
pub fn start(modules_dir: PathBuf) {
    let dir_for_reload = modules_dir.clone();
    wasm_registry::set_reload_handler(Arc::new(move || {
        wasm_registry::clear();
        sync_modules(&dir_for_reload);
        Ok(())
    }));
    wasm_registry::set_load_one_handler(Arc::new(|stub_id, path| load_one(stub_id, path)));
    sync_modules(&modules_dir);
}

/// Scan `modules_dir` and register each module as a stub, instantiating any that are
/// persisted-enabled in `ModulesConfig.module_state`.
///
/// Two layouts are supported: a loose `Modules/<name>.wasm`, and a folder bundle
/// `Modules/<name>/module.wasm` (with an optional sibling `Modules/<name>/Assets/`).
pub fn sync_modules(modules_dir: &Path) {
    let entries = match std::fs::read_dir(modules_dir) {
        Ok(e) => e,
        Err(_) => return, // directory absent — nothing to load
    };
    let cfg = ModulesConfig::load_or_create().ok();

    for entry in entries.flatten() {
        let path = entry.path();
        let wasm_path = if path.is_dir() {
            // Folder bundle: Modules/<name>/module.wasm
            let candidate = path.join("module.wasm");
            if candidate.is_file() {
                candidate
            } else {
                continue;
            }
        } else if path.extension().and_then(|e| e.to_str()) == Some("wasm") {
            path
        } else {
            continue;
        };
        register_from_wasm(&wasm_path, cfg.as_ref());
    }
}

/// Read one `.wasm`, register it as a stub, and instantiate it if persisted-enabled.
fn register_from_wasm(wasm_path: &Path, cfg: Option<&ModulesConfig>) {
    let bytes = match std::fs::read(wasm_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("wasm-host: failed to read {}: {e}", wasm_path.display());
            return;
        }
    };
    let manifest = match wasm_manifest::parse(&bytes) {
        Ok(m) => m,
        Err(e) => {
            eprintln!(
                "wasm-host: {} has no valid manifest: {e}",
                wasm_path.display()
            );
            return;
        }
    };
    let enabled = cfg
        .map(|c| c.runtime_module_enabled(&manifest.name))
        .unwrap_or(false);

    wasm_registry::register_discovered(
        manifest.name.clone(),
        wasm_path.to_path_buf(),
        enabled,
        manifest.version.clone(),
        manifest.description.clone(),
        manifest.required_permissions.clone(),
        manifest.supported_platforms.clone(),
        manifest.tags.clone(),
        manifest.abi_version,
    );

    if enabled && !wasm_registry::module_body_loaded(&manifest.name) {
        if let Err(e) = load_one(manifest.name.clone(), wasm_path.to_path_buf()) {
            eprintln!("wasm-host: failed to load '{}': {e}", manifest.name);
        }
    }
}

/// Instantiate one `.wasm` module and register its commands. Returns the module's canonical
/// name. Invoked by `wasm-host.module-enable` and by [`sync_modules`].
pub fn load_one(stub_id: String, path: PathBuf) -> Result<String, String> {
    let bytes = std::fs::read(&path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    let manifest = wasm_manifest::parse(&bytes)?;

    let loaded =
        LoadedModule::instantiate(&bytes, &manifest.name, &manifest.required_permissions)?;
    // Per-module Mutex: a wasmi Store is not Sync, and dispatch must be serialized per module.
    let shared = Arc::new(Mutex::new(loaded));

    wasm_registry::register_module(manifest.name.clone());

    for cmd in &manifest.commands {
        let token = format!("{}.{}", manifest.name, cmd.verb);
        let verb = cmd.verb.clone();
        let module_for_closure = Arc::clone(&shared);
        let handler = Arc::new(move |args: Vec<String>| -> String {
            match module_for_closure.lock() {
                Ok(mut m) => m
                    .dispatch(&verb, &args)
                    .unwrap_or_else(|e| format!("error: {e}")),
                Err(_) => "error: module instance lock poisoned".to_string(),
            }
        });
        wasm_registry::register_command(
            token,
            cmd.description.clone(),
            handler,
            cmd.required_permissions.clone(),
        );
    }

    let _ = stub_id; // canonical name comes from the manifest, not the file-derived stub id
    Ok(manifest.name)
}
