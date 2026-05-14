use std::collections::BTreeMap;
use std::io;

use serde::{Deserialize, Serialize};

use super::{write_config_toml, ConfigFile};
use crate::shortcuts::{KeyChordSpec, ShortcutAction, ShortcutTrigger};

const FILE_NAME: &str = "shortcuts.toml";

/// A fully user-defined shortcut stored in config (not a static registration override).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomShortcut {
    pub id: String,
    pub label: String,
    pub trigger: ShortcutTrigger,
    #[serde(default)]
    pub actions: Vec<ShortcutAction>,
}

/// User overrides for merged shortcuts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShortcutOverride {
    #[serde(default)]
    pub disabled: Option<bool>,
    #[serde(default)]
    pub chord: Option<KeyChordSpec>,
    /// Replaces a `Sequence` trigger with a custom step list.
    #[serde(default)]
    pub sequence: Option<Vec<KeyChordSpec>>,
    #[serde(default)]
    pub priority: Option<i16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutsConfig {
    #[serde(default)]
    pub overrides: BTreeMap<String, ShortcutOverride>,
    /// Shortcut ids user accepted for OS-global registration (ExecuteCommand / sensitive actions).
    #[serde(default)]
    pub system_wide_consented_ids: Vec<String>,
    /// Fully user-created shortcuts (not overrides of static registrations).
    #[serde(default)]
    pub custom: Vec<CustomShortcut>,
}

impl Default for ShortcutsConfig {
    fn default() -> Self {
        Self {
            overrides: BTreeMap::new(),
            system_wide_consented_ids: Vec::new(),
            custom: Vec::new(),
        }
    }
}

impl ConfigFile for ShortcutsConfig {
    fn file_name() -> &'static str {
        FILE_NAME
    }

    fn merge_defaults(&mut self) -> bool {
        false
    }

    fn save(&self) -> io::Result<()> {
        write_config_toml(Self::file_name(), self)
    }
}

pub fn record_system_wide_consent(shortcut_id: &str) -> io::Result<()> {
    let mut cfg = ShortcutsConfig::load_or_create()?;
    if !cfg
        .system_wide_consented_ids
        .iter()
        .any(|s| s == shortcut_id)
    {
        cfg.system_wide_consented_ids.push(shortcut_id.to_string());
        cfg.save()?;
    }
    Ok(())
}

pub fn has_system_wide_consent(shortcut_id: &str) -> bool {
    ShortcutsConfig::load_or_create()
        .ok()
        .map(|c| c.system_wide_consented_ids.iter().any(|s| s == shortcut_id))
        .unwrap_or(false)
}

pub fn revoke_system_wide_consent(shortcut_id: &str) -> io::Result<()> {
    let mut cfg = ShortcutsConfig::load_or_create()?;
    cfg.system_wide_consented_ids.retain(|s| s != shortcut_id);
    cfg.save()
}
