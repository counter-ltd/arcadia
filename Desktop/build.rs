fn main() {
    // When python-extensions feature is enabled, add the Python framework rpath so the
    // binary can find libpython at runtime without requiring DYLD_LIBRARY_PATH.
    if std::env::var("CARGO_FEATURE_PYTHON_EXTENSIONS").is_ok() {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            if let Ok(output) = Command::new("python3")
                .args([
                    "-c",
                    "import sysconfig; v = sysconfig.get_config_var('PYTHONFRAMEWORKPREFIX') or sysconfig.get_config_var('LIBDIR'); print(v)",
                ])
                .output()
            {
                if output.status.success() {
                    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !path.is_empty() {
                        println!("cargo:rustc-link-arg=-Wl,-rpath,{path}");
                    }
                }
            }
        }
    }
}
