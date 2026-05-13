# Services

Source: `Shared/ArcadiaCore/src/services.rs`  
Page: `utility.services`  
Platform: all

---

## Overview

The Services page shows long-running background services advertised by enabled modules. Services are declared as static `ServiceDefinition` entries in `SERVICE_DEFINITIONS`. The page is service-driven — it is visible whenever at least one service has its `required_module` enabled, regardless of the page's own `required_module` field.

Currently, the LAN discovery service is the primary registered service.

---

## `ServiceDefinition`

```rust
pub struct ServiceDefinition {
    pub id: &'static str,
    pub page_id: &'static str,   // must match a PAGE_DEFINITIONS entry
    pub title: &'static str,
    pub description: &'static str,
    pub required_module: &'static str,
    pub glyph: &'static str,
    pub system_image: &'static str,
    pub accent: &'static str,
    pub controls: ServiceControls,  // not serialized
}
```

| Field | Purpose |
|-------|---------|
| `id` | Stable service identifier |
| `page_id` | Navigation page this service belongs to — makes that page service-driven |
| `required_module` | Module that must be enabled for this service to be considered active |
| `glyph` | Icon key for desktop |
| `system_image` | SF Symbol for iOS |
| `accent` | Accent palette key |
| `controls` | Runtime control function pointers (Start, Stop, status query, port info) |

---

## `ServiceControls`

Function pointers kept as `fn()` (no captures) so `ServiceDefinition` is `Copy` and can live in a `&'static [_]`.

| Field | Signature | Purpose |
|-------|-----------|---------|
| `status_detail` | `fn() -> ServiceRuntimeStatus` | Returns current runtime status snapshot. `None` = no detail; surfaces fall back to module-enabled signal |
| `start` | `fn() -> Result<(), String>` | Start the service background thread. Returns empty string on success, error message otherwise |
| `stop` | `fn()` | Stop the service. Idempotent. |
| `port_for_collision` | `fn() -> u16` | Returns the port the service binds. Surfaces use this to offer a "kill existing process on port and retry" recovery option. |

---

## `ServiceRuntimeStatus`

| Field | Purpose |
|-------|---------|
| `running` | `Option<bool>` — `Some(true)` = running, `Some(false)` = stopped, `None` = indeterminate |
| `detail` | Short status line (e.g. `"UDP :42424 · my-host"`) shown below the service title |

---

## Port collision handling

`is_port_collision_error(err)` is a centralized heuristic for detecting OS port-in-use errors:

```rust
lower.contains("address already in use")
    || lower.contains("already in use")
    || lower.contains("port")
```

When `start` returns an error matching this heuristic, the Services UI can show a "Kill process on port X and retry" button using `port_for_collision()` to know which port to clear.

---

## GUI panel

Files: `Desktop/src/gui/app/services/`

| File | Purpose |
|------|---------|
| `panel.rs` | Services page — lists active services with status badges and Start/Stop controls |
| `port_kill.rs` | Port-collision recovery: find and kill the process occupying a port |
| `mod.rs` | Module exports |

The panel iterates `SERVICE_DEFINITIONS`, filters by `required_module` enabled, and renders each service as a card showing:
- Title and description
- Runtime status badge (running / stopped / unknown)
- Detail line from `status_detail()`
- Start / Stop buttons (when controls provided)
- Port-kill recovery button (when port collision detected)

---

## Navigation visibility

The `utility.services` page is special: `is_page_visible_with` checks `page_has_services(page_id)` before falling through to the normal `required_module` check. If any service in `SERVICE_DEFINITIONS` maps to `utility.services` and has its `required_module` enabled, the page is visible — even if the page itself has no `required_module` set.

---

## Registered services

| Service ID | Page | Module | Description |
|------------|------|--------|-------------|
| `lan-discovery` | `utility.services` | `lan` | UDP LAN peer discovery listener on port 42424 |

Additional services can be registered by adding entries to `SERVICE_DEFINITIONS` in `services.rs`.

---

## Adding a service

1. Add a `ServiceDefinition` to `SERVICE_DEFINITIONS` in `services.rs`.
2. Set `page_id` to an existing page ID (or add a new page to `navigation.rs` first).
3. Implement `ServiceControls` function pointers in the module that owns the service.
4. Done — the Services page picks it up automatically from the registry.
