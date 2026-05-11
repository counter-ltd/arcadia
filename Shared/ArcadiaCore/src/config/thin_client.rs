//! Persisted thin-client preferences (every GUI peer shares optional defaults-remote-route + stable client id).

use std::io;

use serde::{Deserialize, Serialize};

use super::ConfigFile;

const FILE_NAME: &str = "thin-client.toml";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThinClientConfig {
    /// Preferred `ExecutionContext.net_as` shape (`lan:<ip>`).
    #[serde(default)]
    pub preferred_remote_route: Option<String>,
    /// Identifies this surface when emitting patches (`surface.patch.client_id`).
    #[serde(default)]
    pub surface_client_id: Option<String>,
    /// When `true` and a LAN route is active, navigation comes **only** from the host
    /// `surface.snapshot` (`extra.navigation_registry`) — never the shipped static navigation tables.
    #[serde(default)]
    pub navigation_from_host_only: bool,
}

impl ThinClientConfig {
    pub fn surface_client_id_or_generate(&mut self) -> Result<String, io::Error> {
        if let Some(ref id) = self.surface_client_id {
            return Ok(id.clone());
        }
        let id = uuid::Uuid::new_v4().to_string();
        self.surface_client_id = Some(id.clone());
        self.save()?;
        Ok(id)
    }

    pub fn load_surface_client_id() -> String {
        Self::load_or_create()
            .ok()
            .and_then(|mut c| c.surface_client_id_or_generate().ok())
            .unwrap_or_else(|| "unknown-client".to_string())
    }

    pub fn set_preferred_remote_route(route: Option<&str>) -> Result<(), io::Error> {
        let mut cfg = Self::load_or_create()?;
        cfg.preferred_remote_route = route.map(|s| s.to_string());
        cfg.save()
    }
}

impl ConfigFile for ThinClientConfig {
    fn file_name() -> &'static str {
        FILE_NAME
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thin_client_toml_round_trips_navigation_flag() {
        let mut c = ThinClientConfig::default();
        c.navigation_from_host_only = true;
        c.preferred_remote_route = Some("lan:192.168.1.10".into());
        let ser = toml::to_string(&c).unwrap();
        let back: ThinClientConfig = toml::from_str(&ser).unwrap();
        assert!(back.navigation_from_host_only);
        assert_eq!(
            back.preferred_remote_route.as_deref(),
            Some("lan:192.168.1.10")
        );
    }
}
