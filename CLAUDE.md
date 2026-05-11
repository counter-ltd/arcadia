# Arcadia — Claude Code Guide

## What Arcadia Is

Multi-platform runtime and shell: one Rust core (`Shared/ArcadiaCore`) consumed by a single GPUI-style UI layer (`Libraries/OpenFrame`) rendered on three surfaces — desktop (`Desktop/` GUI feature), iOS (Metal-hosted OpenFrame via `--features ios-gui`), and a headless CLI. The core owns all logic; the UI layer renders; the surfaces only initialize and dispatch input.

**Key invariant:** if you find yourself writing the same logic in more than one place, it belongs in `arcadia-core` (logic) or `gui/app/` (presentation) — not in surface entrypoints.

---

## Repository Layout

```
Shared/ArcadiaCore/src/
  lib.rs              module declarations (config, modules, navigation, platform, services)
  navigation.rs       PAGE_DEFINITIONS, GROUP_DEFINITIONS, NavigationRegistryOwned
  services.rs         high-level service registry helpers
  config/
    mod.rs            ConfigFile trait, CONFIG_ROOT_OVERRIDE (iOS sets this)
    modules.rs        MODULE_REGISTRY, ModulesConfig, merge_defaults() migrations
    appearance.rs     appearance.toml (theme tokens)
    commandline.rs    CLI preferences
    extension_tokens.rs  extension theme/glyph token resolution
    late.rs           late.toml (WS chat config)
    thin_client.rs    thin-client.toml (preferred_remote_route, surface_client_id)
  modules/
    mod.rs            execute_command dispatcher, module_commands() lookup
    shell.rs          shell.execute + shell.internal
    shell_motd.rs     MOTD banner
    surface.rs        surface.snapshot / surface.patch / revision counter
    remote_session.rs routing manifest only
    remote_mirror.rs  host transcript queue + drain (for surface mirroring)
    net.rs            networking foundation
    lan/              LAN subsystem (discovery, peers, protocol, handlers)
    late.rs           WebSocket chat / now-playing / votes module
    python_host.rs    Python extension host
    python_registry.rs Python extension manifest registry
  platform/
    mod.rs, macos.rs, ios.rs, linux.rs, windows.rs, unknown.rs

Libraries/OpenFrame/        Rust GPU UI framework (forked from Zed GPUI; multi-platform incl. iOS)

Desktop/
  Cargo.toml          package `arcadia`:
                        [[bin]] arcadia (src/main.rs)            — desktop binary
                        [lib]   arcadia_ios (src/ios_lib.rs)     — staticlib for iOS host
                      features: headless (default), gui, ios-gui, python-extensions
  src/
    main.rs           binary entry — feature-gated gui vs headless
    ios_lib.rs        iOS C ABI entrypoints (arcadia_ios_start, arcadia_ios_inject_touch)
    cli/              REPL, args, completion, config + module commands
    gui/
      mod.rs, assets.rs
      app/
        mod.rs        ArcadiaRoot state struct
        entry.rs      desktop GPUI initialization + window setup
        entry_ios.rs  iOS entrypoint — boots OpenFrame on supplied CAMetalLayer
        lifecycle.rs  focus, resize, module state reload
        navigation.rs nav state, page routing
        root/, sidebar/, shell/, modules_page/, lan_nodes/, splash/,
        appearance/, late/, python_settings/, services/, list_panel_search.rs,
        text_input_caret.rs
      theme/          icon_path(), color tokens, accent palettes, component tokens
      tui/            PTY/TUI terminal emulator (desktop only)

Mobile/iOS/
  ArcadiaApp.xcodeproj  Xcode project — "Build Rust (cargo)" phase shells out to cargo
                        and copies libarcadia_ios.a into BUILT_PRODUCTS_DIR
  ArcadiaApp/
    ArcadiaApp.swift    UIKit @main; bootstraps Metal layer, calls arcadia_ios_start
    MetalHostView.swift CAMetalLayer host view, forwards UITouch → arcadia_ios_inject_touch
    ArcadiaBridge.h     C ABI declarations matching ios_lib.rs

Configuration/  (under $HOME/Arcadia/Configuration on desktop, app Documents on iOS)
  modules.toml        module enable/disable state
  commandline.toml    CLI settings
  thin-client.toml    preferred_remote_route, surface_client_id (UUID)
  appearance.toml     theme tokens
  late.toml           late module state
```

---

## Core Architecture Principles

### Single source of truth — never duplicate

| What | Lives in | Consumed by |
|------|----------|-------------|
| Module list + deps | `config/modules.rs` `MODULE_REGISTRY` | everything |
| Navigation pages/groups | `navigation.rs` `PAGE_DEFINITIONS` / `GROUP_DEFINITIONS` | desktop + iOS GUI |
| Serializable nav | `NavigationRegistryOwned` in `navigation.rs` | `surface.snapshot` payload |
| Theme tokens | `Desktop/src/gui/theme/` | every render path |
| Config schema | `ModulesConfig` in `config/modules.rs` | CLI, GUI |
| Config migrations | `ModulesConfig::merge_defaults()` | every load path |

### Thin surfaces, fat core, single UI layer

- `arcadia-core` owns business logic, state, commands.
- `Desktop/src/gui/` (rendered via OpenFrame) is the UI. It runs identically on desktop and iOS — only the platform shell differs.
- `Desktop/src/main.rs` and `Mobile/iOS/ArcadiaApp/` only initialize the runtime and dispatch native input.

Surface entrypoints must NOT:
- Re-implement business logic that belongs in `arcadia_core`.
- Hard-code module names, page IDs, or feature flags.
- Add per-module booleans (`shell_enabled`, `net_enabled`) — query the config dynamically via `is_module_enabled(name)`.
- Duplicate navigation structure that already exists in `navigation.rs`.

---

## How to Add Things

### New module

1. Add `pub const X_MODULE_NAME: &str = "x";` to `Shared/ArcadiaCore/src/config/modules.rs`.
2. Add a `ModuleManifest` entry to `MODULE_REGISTRY` in the same file.
3. Create `Shared/ArcadiaCore/src/modules/x.rs` with a `commands()` fn returning `&[ModuleCommand]` and a `pub const NAME: &str`.
4. Register the module in `Shared/ArcadiaCore/src/modules/mod.rs` (`pub mod x;` and add to `module_commands()` match).
5. Done — GUI and CLI pick it up from the registry automatically. No surface edits required.

### New navigation page

1. Add a `NavigationPageDefinition` entry to `PAGE_DEFINITIONS` in `navigation.rs`. Set `required_module` if the page depends on a module.
2. Add the page ID to the relevant `NavigationGroupDefinition.pages` slice, or create a new group.
3. Implement the page panel under `Desktop/src/gui/app/` (single implementation — renders on desktop and iOS).
4. Route in the surface content switch — derive visibility from `required_module`, **never** add a hardcoded match arm.

### New icon/glyph

1. Add SVG to `Desktop/assets/icons/`.
2. Add a match arm to `icon_path()` in `Desktop/src/gui/theme/mod.rs`.
3. Use the key in `NavigationPageDefinition.glyph` or `NavigationGroupDefinition.glyph`.

### New theme color

- Add a named constant or helper fn to `Desktop/src/gui/theme/mod.rs` or the appropriate component file under `theme/modules/`.
- Never inline `rgb(0x...)` in view code.

### New mirrored state (thin-client)

1. Extend `SurfaceSnapshot.extra` in `modules/surface.rs`.
2. Add the corresponding `SurfacePatch` variant if clients need to push changes back.
3. Wire the panel to consume the new extra field from the snapshot result.
4. Do not create ad-hoc `remote-session.*` verbs — keep the protocol under `surface.*`.

### Renaming a module

1. Edit `MODULE_REGISTRY` entry + constant in `config/modules.rs`.
2. Add a migration in `ModulesConfig::merge_defaults()` following the `LEGACY_LAN_MODULE_NAME` pattern.
3. Do not do ad-hoc renames at call sites.

---

## What Not to Do

### Named per-module booleans

```rust
// BAD — named module booleans in surface state
pub shell_enabled: bool,
fn net_enabled(&self) -> bool { … }

// GOOD — single generic query using MODULE_NAME constants
fn is_module_enabled(&self, name: &str) -> bool {
    self.module_rows.iter()
        .find(|(n, _)| n == name)
        .map(|(_, enabled)| *enabled)
        .unwrap_or(false)
}
// call as: self.is_module_enabled(SHELL_MODULE_NAME)
```

### Hardcoded page ID match arms in visibility logic

```rust
// BAD
match page_id {
    "utility.shell" => self.shell_enabled,
    "network.overview" => self.net_enabled(),
    _ => true,
}

// GOOD — derive from the page's declared required_module
let Some(page) = navigation::page_by_id(page_id) else { return false };
match page.required_module {
    Some(module_name) => self.is_module_enabled(module_name),
    None => true,
}
```

### Growing if-else chains for page content dispatch

```rust
// BAD — grows indefinitely as pages are added
if self.active_page_id == "utility.shell" { … }
else if self.active_page_id == "global.modules" { … }
```

Use the page registry / lookup; new pages should not require edits to dispatch.

### Inline raw colors in view code

```rust
// BAD
.bg(rgb(0x151a22))
.text_color(rgb(0x93c5fd))

// GOOD
.bg(theme::SURFACE_BG)
.text_color(theme::accent_color(accent))
```

### Duplicating core logic across the codebase

If the same logic lives in `arcadia-core` and `gui/app/`, move it into `arcadia-core`. If the same logic lives in `Desktop/src/main.rs` and `Mobile/iOS/ArcadiaApp/`, neither is the right home — put it in `arcadia-core` or `gui/app/entry*.rs`.

### Ad-hoc `remote-session.*` verbs

Do not create `remote-session.foo` commands for UI mirroring. Extend `surface.snapshot.extra` and `SurfacePatch` instead.

### Config renames without migration

Renaming a module name constant without a migration in `ModulesConfig::merge_defaults()` will silently strand user settings.

---

## LAN / Thin-Client Patterns

### Routing commands

```rust
// Local execution
execute_command("shell.execute", "ls -la", ExecutionContext::default())

// LAN-routed execution — peer enforces its own module rules
execute_command("shell.execute", "ls -la", ExecutionContext {
    net_as: Some("lan:192.168.1.10".to_string()),
    net_timeout_ms: Some(5000),
})
```

### Surface snapshot + patch flow

```
Client calls: execute_command("surface.snapshot", "", context_pointing_at_host)
Host returns: { modules: [...], revision: N, extra: { navigation_registry: "..." } }

Client calls: execute_command("surface.patch", json_ops, context_pointing_at_host)
Host applies: module toggle ops, bumps revision
```

### Remote mirror drain

The desktop GUI polls `remote_mirror::drain_*` on a timer when acting as a client; when state indicates a local-surface resync is needed, the panel calls back into `modules::reload_*` to refresh the GUI.

---

## Module System Details

### ModuleManifest fields

```rust
pub struct ModuleManifest {
    pub name: &'static str,                       // unique key
    pub version: &'static str,
    pub description: &'static str,
    pub required_modules: &'static [&'static str], // transitive deps enforced on enable
}
```

### ModulesConfig key methods

| Method | Purpose |
|--------|---------|
| `manifest_for(name)` | Lookup manifest by name |
| `required_modules_for(name)` | Get declared deps |
| `missing_requirements_for(name)` | Validate preconditions before enable |
| `enable_with_requirements(name)` | Enable transitively |
| `set_module_state(name, enabled)` | Toggle with validation |
| `merge_defaults()` | Config migration — add legacy renames here |

### Adding a command to an existing module

```rust
// In Shared/ArcadiaCore/src/modules/yourmodule.rs
pub const NAME: &str = "yourmodule";

pub fn commands() -> &'static [ModuleCommand] {
    &[
        ModuleCommand { name: "yourmodule.thing", description: "Does thing.", run: run_thing },
    ]
}
```

---

## Navigation System Details

### Page definition fields

```rust
pub struct NavigationPageDefinition {
    pub id: &'static str,                       // "group.name" format
    pub title: &'static str,
    pub description: &'static str,
    pub glyph: &'static str,                    // key into icon_path()
    pub accent: &'static str,                   // accent palette key
    pub required_module: Option<&'static str>,  // drives visibility
}
```

### Key functions

```rust
page_by_id(id: &str) -> Option<&'static NavigationPageDefinition>
group_by_id(id: &str) -> Option<&'static NavigationGroupDefinition>
default_navigation_registry_json() -> String  // emitted via surface.snapshot
```

---

## Configuration Details

### Config file path logic

Desktop: `$HOME/Arcadia/Configuration/` by default — see `config/mod.rs` `config_root_dir()`.

iOS: the iOS host must call `arcadia_core::config::set_config_root(path)` before any config reads. This happens inside `arcadia_ios_start` (called from `ArcadiaApp.swift`) with the app's Documents directory.

### Migration pattern

```rust
// In ModulesConfig::merge_defaults()
const LEGACY_LAN_MODULE_NAME: &str = "lan-module"; // old name
if let Some(val) = self.modules.remove(LEGACY_LAN_MODULE_NAME) {
    self.modules.entry(LAN_MODULE_NAME.to_string()).or_insert(val);
}
```

Follow this pattern for any module rename — one place, no ad-hoc patches.

---

## Key Invariants

- `MODULE_REGISTRY` drives module availability everywhere — don't bypass it.
- `NavigationRegistry` is the GUI navigation contract — never hardcode page/group lists in render code.
- `ConfigFile::merge_defaults()` handles migration — when renaming a module, add the migration there.
- All cross-platform logic lives in `arcadia_core`; all cross-platform UI lives in `Desktop/src/gui/app/`. Platform shells just boot the runtime and forward input.
- `surface.*` is the protocol namespace for UI mirroring — do not create ad-hoc `remote-session.*` verbs.

---

## Build Reference

```sh
# Desktop GUI
cargo build --manifest-path Desktop/Cargo.toml --features gui
cargo run   --manifest-path Desktop/Cargo.toml --features gui

# Desktop headless (CLI)
cargo build --manifest-path Desktop/Cargo.toml                # default = headless
cargo run   --manifest-path Desktop/Cargo.toml

# Core tests (artifacts under Builds/workspace via /.cargo/config.toml)
cargo test -p arcadia-core --manifest-path Shared/Cargo.toml

# iOS static lib (consumed by Mobile/iOS/ArcadiaApp.xcodeproj)
bash Shared/Scripts/Builds/build-ios-app.sh
# or just open Mobile/iOS/ArcadiaApp.xcodeproj — the "Build Rust (cargo)"
# phase runs cargo and copies libarcadia_ios.a automatically.

# Global CLI wrappers (macOS)
bash Shared/Scripts/Installers/install-global-commands-macos.sh
```

Cargo output is unified at `Builds/workspace/` via `/.cargo/config.toml`. Override with `CARGO_TARGET_DIR=... cargo ...` if needed.

---

## Known Gotchas

- `surface.revision` only advances on `surface.patch` — CLI writes bypass it. Do not rely on the revision as a freshness signal for all write paths.
- Multiple concurrent GUIs on the same host = last-write-wins on `modules.toml`. No merge semantics.
- LAN forwarding requires `remote-session`, `lan`, and `net` enabled locally. The peer checks its own module rules for the forwarded token.
- `ARCADIA_NET_AS` env var overrides `thin-client.toml` `preferred_remote_route` on startup.
- iOS surface is OpenFrame rendered into a `CAMetalLayer`, not SwiftUI. The Swift host is a ~50-line UIKit shell (`ArcadiaApp.swift` + `MetalHostView.swift`) whose only job is to provide a Metal layer and forward `UITouch` events. All UI logic lives in `Desktop/src/gui/app/` (via OpenFrame on iOS).
