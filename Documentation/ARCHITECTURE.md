# Architecture

## Philosophy

**Fat core, thin shells.**

`Shared/ArcadiaCore` owns everything. Desktop and iOS read registries, render what those registries say, and `execute_command` back into core. They do not re-implement module graphs or navigation trees.

**Single sources of truth — enforced, not hoped for.**

| Domain | Authority | Never duplicated in |
|--------|-----------|---------------------|
| Module manifests + deps | `MODULE_REGISTRY` · `config/modules.rs` | surface state booleans |
| Navigation pages + groups | `PAGE_DEFINITIONS` / `GROUP_DEFINITIONS` · `navigation.rs` | surface match arms |
| Serializable nav for snapshots | `NavigationRegistryOwned` · embedded in `surface.snapshot` | hardcoded Swift arrays |
| Desktop theme tokens | `gui/theme/` | inline `rgb(0x...)` in views |
| iOS theme tokens | `AppTheme.swift` | inline `Color(hex:)` in views |
| Config schema | `ModulesConfig` · `config/modules.rs` | per-platform config parsers |

**Extend the registry, not scatter `if pageId == …`.**
See `AGENTS.md` for the full list of anti-patterns we refuse to write.

**Discipline at the core. Chaos at the edges. On purpose.**

The architectural discipline of `arcadia-core` — registries, schemas, canonical state, no hardcoded IDs — exists to make the extension layer *safe to be chaotic*. Strict boundaries in the core mean extensions don't need to be strict. An extension can be messy, experimental, surface-specific, fast-moving, structurally impure, and weird. It won't corrupt the runtime underneath it.

Most software chooses: freedom without structure, or structure without freedom. Arcadia is attempting both at different layers simultaneously. The core enforces coherence. The extension layer is where experimentation, exceptions, and "this only exists here" decisions belong.

**Personal tool energy, public repo.**
If Arcadia helps others, great — that's bonus. The goal is a system you own, can fork, and can route across machines you trust.

---

## Command model

All execution flows through a single entry point:

```
execute_command(token: &str, args: &str, context: ExecutionContext) -> String
```

- **Tokens** follow `module.command` format: `shell.execute`, `lan.scan`, `surface.snapshot`, `surface.patch`, etc.
- **`ExecutionContext`** carries `net_as` (optional LAN routing, e.g. `lan:192.168.1.10`) and `net_timeout_ms`.
- When `net_as` is set, `execute_command` forwards the token + args over UDP to the target peer instead of dispatching locally. The peer runs the command under its own module rules.
- LAN forwarding requires local `remote-session`, `lan`, and `net` modules enabled; the peer enforces its own module requirements for the token.
- FFI exposes this identically to iOS and Desktop — same logical API, same routing semantics.

---

## Module system

Modules are entries in `MODULE_REGISTRY` (`config/modules.rs`). Each entry is a `ModuleManifest`:

```rust
pub struct ModuleManifest {
    pub name: &'static str,          // unique key, e.g. "shell"
    pub version: &'static str,
    pub description: &'static str,
    pub required_modules: &'static [&'static str], // dependency enforcement
}
```

`ModulesConfig` (TOML-backed) maps module names to enabled state. Key behaviors:

- `enable_with_requirements(name)` — transitively enables all deps before the target.
- `missing_requirements_for(name)` — returns unmet deps (used for UI requirement prompts).
- `merge_defaults()` — config migration entry point; handles legacy renames (e.g. `LEGACY_LAN_MODULE_NAME`).
- Changes write to `~/Arcadia/Configuration/modules.toml` (Desktop) or the app container path (iOS).

Every surface calls `list_modules()` → `Vec<ModuleStatus>` and renders whatever comes back. No surface hardcodes module names in layout logic.

---

## Navigation system

Navigation structure lives entirely in `navigation.rs` as two static slices:

**`PAGE_DEFINITIONS`** — pages:

| ID | Title | Required Module |
|----|-------|-----------------|
| `utility.shell` | Terminal | `terminal` |
| `utility.services` | Services | _service-driven (any service active)_ |
| `global.dashboard` | Dashboard | — |
| `global.logs` | Logs | — |
| `global.settings` | Settings | — |
| `global.modules` | Modules | — |
| `network.nodes` | Nodes | `lan` |

**`GROUP_DEFINITIONS`** — groups:

| ID | Label | Pages |
|----|-------|-------|
| `utilities` | Utilities | `utility.shell`, `utility.services` |
| `network` | Network | `network.nodes` |
| `social` | Social | `late.now_playing`, `late.experimental` |

**`SERVICE_DEFINITIONS`** (`services.rs`) — modules register here to advertise long-running services on a service-host page. A page becomes service-driven as soon as any entry targets its `page_id`: it is visible iff at least one of its services has its `required_module` enabled, regardless of the page's own `required_module`. Each service may carry `controls` (function pointers for `start` / `stop` / `status_detail`) which the desktop Services panel renders directly; iOS dispatches the equivalent FFI calls per service.

| Service | Page | Required Module | Controls |
|---------|------|-----------------|----------|
| `lan.discovery` | `utility.services` | `lan` | start, stop, status_detail |

**`GLOBAL_PAGE_IDS`** — pages rendered in the sidebar global section: `global.dashboard`, `global.settings`.

**`TOP_BAR_PAGE_IDS`** — pages rendered as compact controls in the surface top bar: `global.logs`, `global.modules`. Each surface chooses how to render them (Desktop pill, iOS toolbar item) — registry stays the source of truth.

`NavigationPageDefinition.required_module` drives visibility for non-service-host pages — surfaces query `is_module_enabled(page.required_module)`, never hardcode per-page logic. Service-host pages (any page that has at least one entry in `SERVICE_DEFINITIONS` targeting it) are visible iff one of their services' `required_module` is enabled; surfaces use the central helper `arcadia_core::navigation::is_page_visible_with` (or `is_page_visible_in_owned` for thin clients consuming a remote registry) so the rule stays in one place. The full registry serializes to JSON via `default_navigation_registry_json()` for:

- iOS FFI: `navigation_registry_json()` → deserializes into `NavigationRegistry` Swift struct
- Thin-client: embedded in `surface.snapshot` extra field so remote clients get host's nav without a local copy

Lookup helpers: `page_by_id(id)`, `group_by_id(id)`.

---

## Thin-client and LAN routing

Arcadia supports a **headless host + GUI client** pattern over LAN:

```
[iOS or Desktop GUI]  ──── surface.snapshot ───►  [headless arcadia host]
                      ◄─── surface.patch    ─────
                      ──── execute_command("lan:IP") ──► (routed command)
```

**`surface.snapshot`** — host serializes current state:
```json
{
  "modules": [{"name": "shell", "enabled": true}, ...],
  "revision": 7,
  "extra": {
    "navigation_registry": "{ ...full nav JSON... }"
  }
}
```

**`surface.patch`** — client pushes changes back:
```json
{
  "client_id": "uuid-from-thin-client.toml",
  "ops": [{"type": "modules_set", "name": "lan", "enabled": true}]
}
```

**`lan.session_targets`** — returns JSON list of approved peers for the session picker UI.

**`thin-client.toml`** persists:
- `preferred_remote_route` — remembered LAN target (e.g. `lan:192.168.1.5`)
- `surface_client_id` — UUID for patch attribution

**`ARCADIA_NET_AS`** env var bootstraps `net_as` on startup, overriding `thin-client.toml`.

**Multi-client caveat:** `modules.toml` is a single file on the host. Concurrent edits are last-writer-wins with no merge semantics. See [ROADMAP.md](ROADMAP.md).

---

## Remote mirror

When this machine executes an inbound `NODE_EXEC` for a remote peer, `modules/remote_mirror.rs` enqueues transcript lines plus a `sync_local_surface` flag. Surfaces drain this via `drain_remote_mirror_batch()` (FFI) on a timer (iOS: 250ms) to:

1. Display remote command output locally.
2. Trigger a `reload_modules()` when `sync_local_surface` is true (host state changed).

---

## Theme system

**Desktop** (`Desktop/src/gui/theme/`):
- Named color constants and helper functions — never inline `rgb(0x...)` in view files.
- `icon_path(glyph: &str) -> &str` — maps glyph keys to SVG asset paths.
- `nav_accents/` — per-accent palettes (amber, cyan, emerald, fuchsia, indigo, orange, sky, teal, violet).
- Component tokens under `modules/` — buttons, panels, rows, toggles, typography.

**iOS** (`AppTheme.swift`):
- All colors as computed properties on `AppTheme(isDark:)`.
- No `Color(hex:)` inline anywhere in view files.

---

## FFI bridge

`ffi.rs` is the UniFFI boundary. All iOS ↔ Rust communication goes through it. Key exports:

**Setup:**
- `set_config_root_path(path: String)` — must be called first on iOS (app sandbox path)

**Command execution:**
- `execute_command(token, args, context: ExecutionContextFfi) -> String`
- `list_commands() -> Vec<CommandInfo>`

**Module control:**
- `list_modules() -> Vec<ModuleStatus>`
- `set_module_enabled(name, enabled) -> String`
- `set_module_enabled_with_requirements(name, enabled) -> String`
- `probe_module_toggle(name, enabled) -> ModuleToggleResult` — preflight check, returns missing deps

**Navigation:**
- `navigation_registry_json() -> String`
- `platform_name() -> String`

**Thin-client:**
- `thin_client_surface_client_id() -> String`
- `thin_client_preferred_route_get() -> Option<String>`
- `thin_client_preferred_route_set(route: String) -> String`

**LAN:**
- `lan_start()`, `lan_stop()`

**Mirror:**
- `drain_remote_mirror_batch() -> RemoteMirrorDrain`

After any change to `ffi.rs` or exported types, run:
```sh
bash Shared/Scripts/Builds/build-ios-framework.sh
```
This regenerates `Mobile/iOS/ArcadiaCore/Generated/` and rebuilds `ArcadiaCore.xcframework`.
