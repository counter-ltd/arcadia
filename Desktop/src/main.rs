mod cli;

use arcadia_core::modules;

fn main() {
    modules::load_all();
    arcadia_core::modules::shell::set_internal_executor(cli::handle_internal);

    // Desktop GUI boots the Python host from `ArcadiaRoot::new` after `tray_backend::install`
    // so `tray_register` / `run_on_main` see a live tray backend. Starting here too would run
    // every extension body twice (duplicate tray rows + interval timers) whenever `gui` is on.
    #[cfg(all(feature = "python-extensions", not(feature = "gui")))]
    {
        use arcadia_core::config::modules::PYTHON_HOST_MODULE_NAME;
        use arcadia_core::config::{modules::ModulesConfig, ConfigFile};
        let python_host_on = ModulesConfig::load_or_create()
            .map(|cfg| {
                cfg.modules
                    .get(PYTHON_HOST_MODULE_NAME)
                    .copied()
                    .unwrap_or(false)
            })
            .unwrap_or(false);
        if python_host_on {
            let ext_dir = arcadia_core::config::config_root_dir()
                .ok()
                .and_then(|d| d.parent().map(|p| p.join("Extensions")))
                .unwrap_or_else(|| std::path::PathBuf::from("Extensions"));
            if let Err(e) = arcadia_python::PythonExtensionHost::start(ext_dir) {
                eprintln!("python-host: {e}");
            }
        }
    }

    // WASM module host — desktop-first, headless path only for now. The GUI start path
    // (lifecycle.rs, paralleling the Python host) is a post-MVP follow-up.
    #[cfg(all(feature = "wasm-modules", not(feature = "gui")))]
    {
        use arcadia_core::config::modules::WASM_HOST_MODULE_NAME;
        use arcadia_core::config::{modules::ModulesConfig, ConfigFile};
        let wasm_host_on = ModulesConfig::load_or_create()
            .map(|cfg| {
                cfg.modules
                    .get(WASM_HOST_MODULE_NAME)
                    .copied()
                    .unwrap_or(false)
            })
            .unwrap_or(false);
        if wasm_host_on {
            let mod_dir = arcadia_core::config::config_root_dir()
                .ok()
                .and_then(|d| d.parent().map(|p| p.join("Modules")))
                .unwrap_or_else(|| std::path::PathBuf::from("Modules"));
            arcadia_wasm::start(mod_dir);
        }
    }

    #[cfg(feature = "gui")]
    {
        gui::run();
        modules::shutdown_all();
        return;
    }

    #[cfg(not(feature = "gui-any"))]
    {
        headless::run();
        modules::shutdown_all();
    }
}

#[cfg(feature = "gui-any")]
mod gui;

#[cfg(not(feature = "gui-any"))]
mod headless {
    use crate::cli;

    pub fn run() {
        cli::print_startup("headless");
        cli::start_loop(|| {});
    }
}
