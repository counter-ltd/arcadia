mod cli;

use arcadia_core::modules;

fn main() {
    modules::load_all();
    arcadia_core::modules::shell::set_internal_executor(cli::handle_internal);

    #[cfg(feature = "python-extensions")]
    {
        use arcadia_core::config::modules::PYTHON_HOST_MODULE_NAME;
        use arcadia_core::config::{modules::ModulesConfig, ConfigFile};
        let python_host_on = ModulesConfig::load_or_create()
            .map(|cfg| cfg.modules.get(PYTHON_HOST_MODULE_NAME).copied().unwrap_or(false))
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

    #[cfg(feature = "gui")]
    {
        gui::run();
        modules::shutdown_all();
        return;
    }

    #[cfg(not(feature = "gui"))]
    {
        headless::run();
        modules::shutdown_all();
    }
}

#[cfg(feature = "gui")]
mod gui;

#[cfg(not(feature = "gui"))]
mod headless {
    use crate::cli;

    pub fn run() {
        cli::print_startup("headless");
        cli::start_loop(|| {});
    }
}
