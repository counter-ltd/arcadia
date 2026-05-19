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

/// Scan `modules_dir` for `.wasm` files, register each as a stub, and instantiate any that
/// are persisted-enabled in `ModulesConfig.module_state`.
pub fn sync_modules(modules_dir: &Path) {
    let entries = match std::fs::read_dir(modules_dir) {
        Ok(e) => e,
        Err(_) => return, // directory absent — nothing to load
    };
    let cfg = ModulesConfig::load_or_create().ok();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("wasm") {
            continue;
        }
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("wasm-host: failed to read {}: {e}", path.display());
                continue;
            }
        };
        let manifest = match wasm_manifest::parse(&bytes) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("wasm-host: {} has no valid manifest: {e}", path.display());
                continue;
            }
        };
        let enabled = cfg
            .as_ref()
            .map(|c| c.runtime_module_enabled(&manifest.name))
            .unwrap_or(false);

        wasm_registry::register_discovered(
            manifest.name.clone(),
            path.clone(),
            enabled,
            manifest.version.clone(),
            manifest.description.clone(),
            manifest.required_permissions.clone(),
            manifest.supported_platforms.clone(),
            manifest.tags.clone(),
            manifest.abi_version,
        );

        if enabled && !wasm_registry::module_body_loaded(&manifest.name) {
            if let Err(e) = load_one(manifest.name.clone(), path) {
                eprintln!("wasm-host: failed to load '{}': {e}", manifest.name);
            }
        }
    }
}

/// Instantiate one `.wasm` module and register its commands. Returns the module's canonical
/// name. Invoked by `wasm-host.module-enable` and by [`sync_modules`].
pub fn load_one(stub_id: String, path: PathBuf) -> Result<String, String> {
    let bytes = std::fs::read(&path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    let manifest = wasm_manifest::parse(&bytes)?;

    let loaded = LoadedModule::instantiate(&bytes, &manifest.name)?;
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
