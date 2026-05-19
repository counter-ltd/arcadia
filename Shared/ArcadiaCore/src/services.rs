//! Service registry — modules register here to advertise themselves on the **Services** page.
//!
//! Pages whose `page_id` appears in [`SERVICE_DEFINITIONS`] become *service-host pages*: they
//! are visible iff at least one of their registered services has its `required_module` enabled.
//! See [`crate::navigation::is_page_visible_with`] for the rule.

use serde::{Deserialize, Serialize};


/// Status snapshot returned by [`ServiceControls::status_detail`]: drives the badge + detail line.
#[derive(Clone, Debug)]
pub struct ServiceRuntimeStatus {
    /// Whether the service's background work is active right now (UDP listener up, etc.).
    /// `None` means the service has no runtime distinct from "module enabled".
    pub running: Option<bool>,
    /// Short single-line status (e.g. `"UDP :42424 · my-host"`).
    pub detail: String,
}

/// Optional controls a service exposes. Surfaces use these to render Start/Stop without
/// needing to special-case the service id. Function pointers (no captures) keep
/// [`ServiceDefinition`] `Copy` so it can live in `&'static [_]`.
#[derive(Clone, Copy, Default)]
pub struct ServiceControls {
    /// Returns a fresh status snapshot. `None` means "no runtime detail" — surfaces fall back
    /// to the module-enabled signal.
    pub status_detail: Option<fn() -> ServiceRuntimeStatus>,
    /// Start the service. Empty string on success, error message otherwise.
    pub start: Option<fn() -> Result<(), String>>,
    /// Stop the service. Idempotent — safe to call when already stopped.
    pub stop: Option<fn()>,
    /// Port the service tries to bind. When set, surfaces can offer a generic
    /// "kill existing process on this port and retry" recovery on collision errors.
    pub port_for_collision: Option<fn() -> u16>,
}

/// Heuristic check for OS port-collision messages from `start`. Centralized so every surface
/// recognizes the same error class without duplicating string matching.
pub fn is_port_collision_error(err: &str) -> bool {
    let lower = err.to_ascii_lowercase();
    lower.contains("address already in use")
        || lower.contains("already in use")
        || lower.contains("port")
}

pub fn service_by_id(id: &str) -> Option<&'static ServiceDefinition> {
    SERVICE_DEFINITIONS.iter().find(|s| s.id == id)
}

/// Static service descriptor; one entry per module-provided service.
///
/// `page_id` must match a page in [`crate::navigation::PAGE_DEFINITIONS`] — that page becomes
/// service-driven (its own `required_module` is ignored once any service registers to it).
#[derive(Clone, Copy, Serialize)]
pub struct ServiceDefinition {
    pub id: &'static str,
    pub page_id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    /// Module that must be enabled for this service to be considered active.
    pub required_module: &'static str,
    /// Theme glyph key for desktop; mapped via `Desktop/src/gui/theme/icons.rs`.
    pub glyph: &'static str,
    /// SF Symbol for iOS.
    pub system_image: &'static str,
    pub accent: &'static str,
    /// Runtime hooks; never serialized — control routes go through native calls per surface.
    #[serde(skip)]
    pub controls: ServiceControls,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceOwned {
    pub id: String,
    pub page_id: String,
    pub title: String,
    pub description: String,
    pub required_module: String,
    pub glyph: String,
    #[serde(rename = "system_image")]
    pub system_image: String,
    pub accent: String,
}

impl From<&ServiceDefinition> for ServiceOwned {
    fn from(s: &ServiceDefinition) -> Self {
        ServiceOwned {
            id: s.id.to_string(),
            page_id: s.page_id.to_string(),
            title: s.title.to_string(),
            description: s.description.to_string(),
            required_module: s.required_module.to_string(),
            glyph: s.glyph.to_string(),
            system_image: s.system_image.to_string(),
            accent: s.accent.to_string(),
        }
    }
}

/// Advertised services — built once from every extension's `services()`.
/// Replaces the former hand-written array; each module declares its own.
pub static SERVICE_DEFINITIONS: std::sync::LazyLock<Vec<ServiceDefinition>> =
    std::sync::LazyLock::new(|| {
        let providers = crate::extension::provider::default_providers();
        let collected = crate::extension::collector::collect(&providers)
            .expect("extension collector must produce a valid service set");
        collected.services().iter().map(|s| **s).collect()
    });

pub fn services_for_page(page_id: &str) -> Vec<&'static ServiceDefinition> {
    SERVICE_DEFINITIONS
        .iter()
        .filter(|s| s.page_id == page_id)
        .collect()
}

pub fn page_has_services(page_id: &str) -> bool {
    SERVICE_DEFINITIONS.iter().any(|s| s.page_id == page_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::modules::{ModulesConfig, LAN_MODULE_NAME};

    #[test]
    fn services_target_known_modules() {
        for service in SERVICE_DEFINITIONS.iter() {
            assert!(
                ModulesConfig::manifest_for(service.required_module).is_some(),
                "service '{}' requires module '{}' which is not in MODULE_REGISTRY",
                service.id,
                service.required_module
            );
        }
    }

    #[test]
    fn lan_service_targets_utility_services_page() {
        let services = services_for_page("utility.services");
        let first = services.first().expect("services_for_page must yield LAN");
        assert_eq!(first.id, "lan.discovery");
        assert_eq!(first.required_module, LAN_MODULE_NAME);
    }

    #[test]
    fn page_has_services_only_for_registered_pages() {
        assert!(page_has_services("utility.services"));
        assert!(!page_has_services("utility.shell"));
        assert!(!page_has_services("does.not.exist"));
    }

    #[test]
    fn service_owned_round_trips_through_json() {
        let owned: ServiceOwned = (&SERVICE_DEFINITIONS[0]).into();
        let json = serde_json::to_string(&owned).unwrap();
        let back: ServiceOwned = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, owned.id);
        assert_eq!(back.page_id, owned.page_id);
        assert_eq!(back.required_module, owned.required_module);
        assert_eq!(back.system_image, owned.system_image);
    }

    #[test]
    fn is_port_collision_error_matches_common_messages() {
        assert!(is_port_collision_error("Address already in use"));
        assert!(is_port_collision_error(
            "bind() failed: address already in use (os error 48)"
        ));
        assert!(is_port_collision_error("UDP port 42424 already in use"));
        assert!(!is_port_collision_error("permission denied"));
        assert!(!is_port_collision_error(""));
    }

    #[test]
    fn service_by_id_finds_lan_discovery() {
        let s = service_by_id("lan.discovery").expect("registered service must resolve");
        assert_eq!(s.required_module, LAN_MODULE_NAME);
        assert!(s.controls.port_for_collision.is_some());
    }

    #[test]
    fn service_by_id_unknown_returns_none() {
        assert!(service_by_id("does.not.exist").is_none());
    }
}
