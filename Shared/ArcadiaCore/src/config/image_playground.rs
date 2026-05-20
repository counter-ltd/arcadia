use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::config::ConfigFile;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ImagePlaygroundStyle {
    #[default]
    Animation,
    Illustration,
    Sketch,
}

impl ImagePlaygroundStyle {
    pub fn as_swift_token(&self) -> &'static str {
        match self {
            Self::Animation => "animation",
            Self::Illustration => "illustration",
            Self::Sketch => "sketch",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Animation => "Animation",
            Self::Illustration => "Illustration",
            Self::Sketch => "Sketch",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImagePlaygroundConfig {
    #[serde(default)]
    pub default_style: ImagePlaygroundStyle,
    /// Absolute path to the directory where generated PNGs are persisted.
    /// `None` resolves to `<config_root>/image_playground/cache` at runtime.
    #[serde(default)]
    pub cache_dir: Option<PathBuf>,
}

impl Default for ImagePlaygroundConfig {
    fn default() -> Self {
        Self {
            default_style: ImagePlaygroundStyle::default(),
            cache_dir: None,
        }
    }
}

impl ConfigFile for ImagePlaygroundConfig {
    fn file_name() -> &'static str {
        "image_playground.toml"
    }
}

impl ImagePlaygroundConfig {
    /// Resolve the effective cache directory, falling back to the configured
    /// override or `<config_root>/image_playground/cache`. Creates the directory
    /// if it does not exist.
    pub fn resolve_cache_dir(&self) -> std::io::Result<PathBuf> {
        let dir = match &self.cache_dir {
            Some(p) => p.clone(),
            None => {
                let mut p = crate::config::config_root_dir()?;
                p.push("image_playground");
                p.push("cache");
                p
            }
        };
        std::fs::create_dir_all(&dir)?;
        Ok(dir)
    }
}
