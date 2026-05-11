# Architecture

## Philosophy

**Fat core, thin shells.**

`Shared/ArcadiaCore` owns everything. The Rust UI layer reads registries, renders what those registries say, and `execute_command`s back into core. Desktop and iOS use the **same** UI layer (`Desktop/src/gui/app/` rendered via OpenFrame) — they do not re-implement module graphs or navigation trees, and neither does the OS-specific shell around the UI.

**Single sources of truth — enforced, not hoped for.**

| Domain | Authority | Never duplicated in |
|--------|-----------|---------------------|
| Module manifests + deps | `MODULE_REGISTRY` · `config/modules.rs` | surface state booleans |
| Navigation pages + groups | `PAGE_DEFINITIONS` / `GROUP_DEFINITIONS` · `navigation.rs` | surface match arms |
| Serializable nav for snapshots | `NavigationRegistryOwned` · embedded in `surface.snapshot` | hardcoded Swift arrays |
| Theme tokens | `gui/theme/` | inline `rgb(0x...)` in views |
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
- Both surfaces call `execute_command` directly — iOS through the Rust UI running on top of OpenFrame, desktop through GPUI — so routing semantics are identical.

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

**`SERVICE_DEFINITIONS`** (`services.rs`) — modules register here to advertise long-running services on a service-host page. A page becomes service-driven as soon as any entry targets its `page_id`: it is visible iff at least one of its services has its `required_module` enabled, regardless of the page's own `required_module`. Each service may carry `controls` (function pointers for `start` / `stop` / `status_detail`) which the Services panel renders directly on whichever surface is hosting the GUI.

| Service | Page | Required Module | Controls |
|---------|------|-----------------|----------|
| `lan.discovery` | `utility.services` | `lan` | start, stop, status_detail |

**`GLOBAL_PAGE_IDS`** — pages rendered in the sidebar global section: `global.dashboard`, `global.settings`.

**`TOP_BAR_PAGE_IDS`** — pages rendered as compact controls in the surface top bar: `global.logs`, `global.modules`. Each surface chooses how to render them (Desktop pill, iOS toolbar item) — registry stays the source of truth.

`NavigationPageDefinition.required_module` drives visibility for non-service-host pages — surfaces query `is_module_enabled(page.required_module)`, never hardcode per-page logic. Service-host pages (any page that has at least one entry in `SERVICE_DEFINITIONS` targeting it) are visible iff one of their services' `required_module` is enabled; surfaces use the central helper `arcadia_core::navigation::is_page_visible_with` (or `is_page_visible_in_owned` for thin clients consuming a remote registry) so the rule stays in one place. The full registry serializes to JSON via `default_navigation_registry_json()` for the thin-client snapshot path — it is embedded in `surface.snapshot.extra` so remote clients get the host's nav without a local copy.

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

When this machine executes an inbound `NODE_EXEC` for a remote peer, `modules/remote_mirror.rs` enqueues transcript lines plus a `sync_local_surface` flag. The GUI drains this batch on a timer to:

1. Display remote command output locally.
2. Trigger a `reload_modules()` when `sync_local_surface` is true (host state changed).

---

## Theme system

`Desktop/src/gui/theme/`:
- Named color constants and helper functions — never inline `rgb(0x...)` in view files.
- `icon_path(glyph: &str) -> &str` — maps glyph keys to SVG asset paths.
- `nav_accents/` — per-accent palettes (amber, cyan, emerald, fuchsia, indigo, orange, sky, teal, violet).
- Component tokens under `modules/` — buttons, panels, rows, toggles, typography.

The same theme tokens drive both desktop and iOS rendering because both surfaces use the same Rust UI code (OpenFrame).

---

## iOS surface

iOS is not a separate UI implementation. The `arcadia` package builds a static lib (`libarcadia_ios.a`) with `--features ios-gui` that contains the same Rust GUI as the desktop binary, rendered through OpenFrame onto a `CAMetalLayer`. The Swift host is a thin UIKit shell.

**C ABI** (`Desktop/src/ios_lib.rs`, mirrored in `Mobile/iOS/ArcadiaApp/ArcadiaBridge.h`):

```c
void arcadia_ios_start(uintptr_t metal_layer_ptr, const char* config_root);
void arcadia_ios_inject_touch(float x, float y, uint8_t phase);
```

`arcadia_ios_start`:
1. Calls `arcadia_core::config::set_config_root(path)` with the app's Documents directory.
2. Calls `arcadia_core::modules::load_all()`.
3. Hands control to `gui::app::entry_ios::run(metal_layer_ptr)`, which boots OpenFrame against the supplied Metal layer.

`arcadia_ios_inject_touch` forwards `UITouch` phases (0=began, 1=moved, 2=ended, 3=cancelled) into OpenFrame's event loop.

**Build:** the Xcode project `Mobile/iOS/ArcadiaApp.xcodeproj` has a "Build Rust (cargo)" phase that runs `cargo build -p arcadia --features ios-gui --lib --target aarch64-apple-ios[-sim]`, with `--target-dir Builds/workspace` matching the repo-wide cargo config, and copies the resulting `libarcadia_ios.a` into `BUILT_PRODUCTS_DIR` for linking. No UniFFI, no Swift bindings, no `xcframework` rebuild step.

Extending the iOS surface usually means adding a panel under `Desktop/src/gui/app/` (which renders on both surfaces). Add to the C ABI only when you need a new platform-level capability that can't go through `execute_command`.
