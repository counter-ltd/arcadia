fn main() {
    // On macOS, add rpath so the binary can find the Python framework at runtime.
    // pyo3-build-config handles the link flags; we just need the rpath.
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("python3")
            .args(["-c", "import sysconfig; v = sysconfig.get_config_var('PYTHONFRAMEWORKPREFIX') or sysconfig.get_config_var('LIBDIR'); print(v)"])
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
