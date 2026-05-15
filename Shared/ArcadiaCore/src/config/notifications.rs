use serde::{Deserialize, Serialize};
use std::io;

use crate::config::{write_config_toml, ConfigFile};

const FILE_NAME: &str = "notifications.toml";
const DEFAULT_MAX_COUNT: usize = 200;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationDestination {
    /// Post to the host operating system's native notification system.
    System,
}

impl NotificationDestination {
    pub fn label(&self) -> &'static str {
        match self {
            NotificationDestination::System => "System",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            NotificationDestination::System => "Send via the host OS notification system.",
        }
    }
}

fn default_destinations() -> Vec<NotificationDestination> {
    vec![NotificationDestination::System]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationEntry {
    pub id: String,
    pub title: String,
    pub body: String,
    /// Module or extension id that posted this notification.
    pub source: String,
    /// Unix timestamp (seconds) when the notification was posted.
    pub timestamp: u64,
    pub read: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationsConfig {
    #[serde(default = "default_max_count")]
    pub max_count: usize,
    #[serde(default = "default_destinations")]
    pub destinations: Vec<NotificationDestination>,
    /// When true, all enabled modules and extensions may post without individual grants.
    #[serde(default)]
    pub trust_all_sources: bool,
    #[serde(default)]
    pub entries: Vec<NotificationEntry>,
}

fn default_max_count() -> usize {
    DEFAULT_MAX_COUNT
}

impl Default for NotificationsConfig {
    fn default() -> Self {
        Self {
            max_count: DEFAULT_MAX_COUNT,
            destinations: default_destinations(),
            trust_all_sources: false,
            entries: Vec::new(),
        }
    }
}

impl NotificationsConfig {
    pub fn unread_count(&self) -> usize {
        self.entries.iter().filter(|e| !e.read).count()
    }

    pub fn post(&mut self, id: String, title: String, body: String, source: String) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.entries.insert(
            0,
            NotificationEntry {
                id,
                title,
                body,
                source,
                timestamp,
                read: false,
            },
        );
        if self.entries.len() > self.max_count.max(1) {
            self.entries.truncate(self.max_count.max(1));
        }
    }

    pub fn mark_read(&mut self, id: &str) {
        if let Some(e) = self.entries.iter_mut().find(|e| e.id == id) {
            e.read = true;
        }
    }

    pub fn mark_all_read(&mut self) {
        for e in &mut self.entries {
            e.read = true;
        }
    }
}

impl ConfigFile for NotificationsConfig {
    fn file_name() -> &'static str {
        FILE_NAME
    }

    fn save(&self) -> io::Result<()> {
        write_config_toml(Self::file_name(), self)
    }
}
