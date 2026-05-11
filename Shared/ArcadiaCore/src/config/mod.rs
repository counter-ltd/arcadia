pub mod appearance;
pub mod commandline;
pub mod extension_tokens;
pub mod late;
pub mod modules;
pub mod thin_client;

use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

static CONFIG_ROOT_OVERRIDE: OnceLock<PathBuf> = OnceLock::new();

pub fn set_config_root(path: PathBuf) {
    let _ = CONFIG_ROOT_OVERRIDE.set(path);
}

pub fn config_root_dir() -> io::Result<PathBuf> {
    if let Some(p) = CONFIG_ROOT_OVERRIDE.get() {
        return Ok(p.clone());
    }

    let home = env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "home directory not found"))?;

    let mut root = PathBuf::from(home);
    root.push("Arcadia");
    root.push("Configuration");
    Ok(root)
}

pub fn config_file_path(file_name: &str) -> io::Result<PathBuf> {
    if file_name.trim().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "config file name cannot be empty",
        ));
    }

    let mut path = config_root_dir()?;
    path.push(file_name);
    Ok(path)
}

pub trait ConfigFile: Default + Serialize + for<'de> Deserialize<'de> + Sized {
    fn file_name() -> &'static str;

    fn file_path() -> io::Result<PathBuf> {
        config_file_path(Self::file_name())
    }

    fn merge_defaults(&mut self) -> bool {
        false
    }

    fn load_or_create() -> io::Result<Self> {
        let root = config_root_dir()?;
        fs::create_dir_all(&root)?;

        let path = Self::file_path()?;

        if !path.exists() {
            let default = Self::default();
            let content = toml::to_string_pretty(&default).map_err(io::Error::other)?;
            fs::write(&path, content)?;
            return Ok(default);
        }

        let content = fs::read_to_string(&path)?;
        let mut cfg = toml::from_str::<Self>(&content).map_err(io::Error::other)?;
        if cfg.merge_defaults() {
            cfg.save()?;
        }
        Ok(cfg)
    }

    fn save(&self) -> io::Result<()> {
        let root = config_root_dir()?;
        fs::create_dir_all(&root)?;
        let path = Self::file_path()?;
        let content = toml::to_string_pretty(self).map_err(io::Error::other)?;
        fs::write(path, content)
    }
}
