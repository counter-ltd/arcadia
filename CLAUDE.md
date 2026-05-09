# Arcadia — Claude Code Guide

## What Arcadia Is

Multi-platform runtime and shell: one Rust core (`Shared/ArcadiaCore`) consumed by two thin surface shells — a GPUI desktop app (`Desktop/`) and a SwiftUI iOS app (`Mobile/iOS/`) — plus a headless CLI. The core owns all logic; surfaces only render and dispatch.

**Key invariant:** if you find yourself writing the same logic in both `Desktop/src/gui/app/` and `Mobile/iOS/ArcadiaApp/`, it belongs in `arcadia-core` instead.

---

## Repository Layout

```
Shared/ArcadiaCore/src/
  lib.rs              root module, exports, UniFFI scaffolding
  ffi.rs              UniFFI bridge — Rust → Swift (all iOS calls go through here)
  navigation.rs       PAGE_DEFINITIONS, GROUP_DEFINITIONS, registry JSON serialization
  config/
    mod.rs            ConfigFile trait, config root path (~/ vs iOS sandbox)
    modules.rs        MODULE_REGISTRY, ModulesConfig, config migrations
    commandline.rs    CLI preferences scaffold
    thin_client.rs    ThinClientConfig → thin-client.toml
  modules/
    mod.rs            execute_command dispatcher, module lifecycle (load_all, shutdown_all)
    shell.rs          shell.execute + shell.internal; PTY integration
    shell_motd.rs     fastfetch-style MOTD banner (requires shell)
    surface.rs        surface.snapshot / surface.patch / revision counter
    remote_session.rs routing manifest only — no standalone commands
    remote_mirror.rs  host transcript queue + FFI drain (RemoteMirrorDrain)
    net.rs            networking foundation
    lan/              LAN subsystem — see module reference
      mod.rs, discovery.rs, handlers.rs, config.rs, peers.rs, protocol.rs
  platform/
    mod.rs, macos.rs, ios.rs, linux.rs, windows.rs, unknown.rs

Desktop/src/
  main.rs             binary entry — feature-gated gui vs headless
  cli/
    mod.rs            REPL loop, command dispatch, startup messages
    args.rs           CLI argument parsing
    completion.rs     shell completion helpers
    config_cmds.rs    module/config CLI commands
    module_cmds.rs    module shortcut commands
  gui/
    mod.rs
    assets.rs         embedded SVG asset loading
    app/
      mod.rs          ArcadiaRoot state struct, ShellMode enum
      entry.rs        GPUI initialization + window setup
      lifecycle.rs    focus, resize, module state reload
      navigation.rs   nav state, page routing, sidebar group logic
      root/           mod.rs, render.rs, top_bar.rs
      sidebar/        mod.rs, layout.rs, nav_items.rs
      shell/          mod.rs, panel.rs, execute.rs, keys.rs, tui_screen.rs, mirror.rs
      modules_page/   mod.rs, panel.rs, row.rs, requirements_modal.rs
      lan_nodes/      mod.rs, panel.rs
      splash/         mod.rs, view.rs, draw_*.rs, math.rs
    theme/
      mod.rs          icon_path(), color constants and helpers
      chrome.rs       window chrome styles
      icons.rs        icon metadata
      splash_colors.rs
      modules/        component tokens: buttons, panel, row_surface, toggle_states, typography
      nav_accents/    per-accent palettes (9 accents: amber, cyan, emerald, fuchsia, indigo, orange, sky, teal, violet)
    tui/
      mod.rs, session.rs   PTY session state + lifecycle
      ansi_line.rs         ANSI escape sequence parsing
      colors.rs            terminal color palette
      cd_builtin.rs        cd builtin (updates shell_working_dir)
      cwd.rs, env.rs       CWD tracking, env vars
      keys.rs              PTY keyboard events
      vt_history.rs        VT100 history buffer

Mobile/iOS/ArcadiaApp/
  ArcadiaApp.swift              @main, set_config_root_path early
  ContentView.swift             top-level coordinator
  ContentView+Actions.swift     action methods
  ContentView+Layout.swift      layout + composition
  ContentView+NavigationState.swift  navigation state helpers
  ContentView+Registry.swift    NavigationRegistry loading + JSON deserialization
  NavigationModels.swift        Swift structs mirroring NavigationRegistry
  AppTheme.swift                all iOS colors as computed properties
  SidebarView.swift             sidebar + remote session picker
  SplashView.swift              animated splash
  ShellView.swift               shell input + history
  ModulesView.swift             module toggle list
  LanNodesView.swift            LAN peer discovery + pairing
  ModuleNames.swift             string constants mirroring MODULE_REGISTRY
  GlassComponents.swift         reusable glassmorphism components

Configuration/
  modules.toml        module enable/disable state
  commandline.toml    CLI settings
  thin-client.toml    preferred_remote_route, surface_client_id (UUID)
```

---

## Core Architecture Principles

### Single source of truth — never duplicate

| What | Lives in | Consumed by |
|------|----------|-------------|
| Module list + deps | `config/modules.rs` `MODULE_REGISTRY` | everything |
| Navigation pages/groups | `navigation.rs` `PAGE_DEFINITIONS` / `GROUP_DEFINITIONS` | Desktop gui, iOS via JSON |
| Serializable nav | `NavigationRegistryOwned` in `navigation.rs` | `surface.snapshot`, FFI |
| Desktop theme | `gui/theme/` | view files (never inline) |
| iOS theme | `AppTheme.swift` | SwiftUI views (never inline) |
| Config schema | `ModulesConfig` in `config/modules.rs` | CLI, GUI, iOS |
| Config migrations | `ModulesConfig::merge_defaults()` | every load path |

### Non-monolithic — thin surfaces, fat core

Surface code (Desktop `gui/`, iOS `ArcadiaApp/`) must:
- **Read** from registries and configs
- **Render** what the registry says
- **Dispatch** user actions to `arcadia_core`

Surface code must NOT:
- Re-implement business logic that belongs in `arcadia_core`
- Hard-code module names, page IDs, or feature flags in render/layout logic
- Add per-module booleans (`shell_enabled`, `net_enabled`) — query the config dynamically
- Duplicate navigation structure that already exists in `navigation.rs`

---

## How to Add Things

### New module

1. Add `pub const X_MODULE_NAME: &str = "x";` to `Shared/ArcadiaCore/src/config/modules.rs`.
2. Add a `ModuleManifest` entry to `MODULE_REGISTRY` in the same file.
3. Create `Shared/ArcadiaCore/src/modules/x.rs` with a `commands()` fn returning `&[ModuleCommand]`.
4. Register in `Shared/ArcadiaCore/src/modules/mod.rs`.
5. Done — GUI, CLI, and iOS all pick it up from the registry automatically. No surface edits required.

### New navigation page

1. Add a `NavigationPageDefinition` entry to `PAGE_DEFINITIONS` in `navigation.rs`. Set `required_module` if the page depends on a module.
2. Add the page ID to the relevant `NavigationGroupDefinition.pages` slice, or create a new group.
3. Implement the page panel: Desktop → new file under `gui/app/`; iOS → new view file in `ArcadiaApp/`.
4. Route in the surface content switch — derive visibility from `required_module`, **never** add a hardcoded match arm.

### New icon/glyph

1. Add SVG to `Desktop/assets/icons/`.
2. Add a match arm to `icon_path()` in `Desktop/src/gui/theme/mod.rs`.
3. Use the key in `NavigationPageDefinition.glyph` or `NavigationGroupDefinition.glyph`.

### New theme color

- Desktop: add named constant or helper fn to `Desktop/src/gui/theme/mod.rs` or the appropriate component file under `theme/modules/`.
- iOS: add a computed property to `AppTheme` in `AppTheme.swift`.
- Never inline `rgb(0x...)` in Rust view code or `Color(hex:)` in Swift view files.

### New mirrored state (thin-client)

1. Extend `SurfaceSnapshot.extra` in `modules/surface.rs`.
2. Add the corresponding `SurfacePatch` variant if clients need to push changes back.
3. Wire Desktop + iOS surfaces to consume the new extra field from snapshot.
4. Do not create ad-hoc `remote-session.*` verbs — keep the protocol under `surface.*`.

### Renaming a module

1. Edit `MODULE_REGISTRY` name and constant in `config/modules.rs`.
2. Add a migration in `ModulesConfig::merge_defaults()` following the `LEGACY_LAN_MODULE_NAME` pattern.
3. Do not do ad-hoc renames at call sites.

### After FFI changes

Any edit to `ffi.rs` or exported FFI types requires:
```sh
bash Shared/Scripts/Builds/build-ios-framework.sh
```
This regenerates `Mobile/iOS/ArcadiaCore/Generated/` and rebuilds `ArcadiaCore.xcframework`. Commit both.

---

## What Not to Do

### Named per-module booleans

```rust
// BAD — named module booleans in surface state
pub shell_enabled: bool,
fn net_enabled(&self) -> bool { … }
fn remote_session_enabled(&self) -> bool { … }

// GOOD — single generic query using MODULE_NAME constants
fn is_module_enabled(&self, name: &str) -> bool {
    self.module_rows.iter()
        .find(|(n, _)| n == name)
        .map(|(_, enabled)| *enabled)
        .unwrap_or(false)
}
// call as: self.is_module_enabled(SHELL_MODULE_NAME)
```

```swift
// GOOD — Swift equivalent
func isModuleEnabled(_ name: String) -> Bool {
    modules.first(where: { $0.name == name })?.enabled ?? false
}
```

### Hardcoded page ID match arms in visibility logic

```rust
// BAD
fn is_page_visible(&self, page_id: &str) -> bool {
    match page_id {
        "utility.shell" => self.shell_enabled,
        "network.overview" => self.net_enabled(),
        _ => true,
    }
}

// GOOD — derive from the page's declared required_module
fn is_page_visible(&self, page_id: &str) -> bool {
    let Some(page) = navigation::page_by_id(page_id) else { return false };
    match page.required_module {
        Some(module_name) => self.is_module_enabled(module_name),
        None => true,
    }
}
```

### Growing if-else chains for page dispatch

```rust
// BAD — grows indefinitely as pages are added
if self.active_page_id == "utility.shell" { … shell panel … }
else if self.active_page_id == "global.modules" { … modules panel … }
else if self.active_page_id == "network.nodes" { … }  // don't add here

// GOOD — dispatch via page registry / lookup; new pages register themselves
```

### Special-casing page IDs in event handlers

```swift
// BAD — magic behavior on specific page ID in a generic handler
.onChange(of: activePageID) { pageID in
    if pageID == "global.modules" { reloadModules() }
}

// GOOD — modules page is just a page; reload via onAppear of ModulesView
```

### Inline colors in view code

```rust
// BAD — raw hex in app.rs or any render file
.bg(rgb(0x151a22))
.text_color(rgb(0x93c5fd))

// GOOD — named token from theme/
.bg(theme::SURFACE_BG)
.text_color(theme::accent_color(accent))
```

```swift
// BAD
Color(hex: "151a22")

// GOOD
theme.surfaceBackground
```

### Duplicating core logic across surfaces

```
// BAD — same logic in app.rs AND ContentView.swift
// GOOD — implement once in arcadia-core; both surfaces call execute_command or FFI
```

---

## LAN / Thin-Client Patterns

### Routing commands

```rust
// Local execution
execute_command("shell.execute", "ls -la", ExecutionContext::local())

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

iOS and Desktop poll `drain_remote_mirror_batch()` on a timer (iOS: 250ms). When `sync_local_surface` is true in the drain result, call `reload_modules()` to resync local UI with host state.

---

## Module System Details

### ModuleManifest fields

```rust
pub struct ModuleManifest {
    pub name: &'static str,                      // unique key
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
| `enable_with_requirements(name)` | Enable transitively (enables all deps first) |
| `set_module_state(name, enabled)` | Toggle with validation |
| `merge_defaults()` | Config migration — add legacy renames here |

### Adding a command to an existing module

```rust
// In Shared/ArcadiaCore/src/modules/yourmodule.rs
pub fn commands() -> &'static [ModuleCommand] {
    &[
        ModuleCommand { token: "yourmodule.thing", description: "Does thing." },
        // add new command here
    ]
}
```

---

## Navigation System Details

### Page definition fields

```rust
pub struct NavigationPageDefinition {
    pub id: &'static str,           // "group.name" format
    pub title: &'static str,
    pub description: &'static str,
    pub glyph: &'static str,        // key into icon_path() in theme.rs
    pub system_image: &'static str, // SF Symbol name for iOS
    pub accent: &'static str,       // accent palette key
    pub required_module: Option<&'static str>, // drives visibility on all surfaces
}
```

### Key functions

```rust
page_by_id(id: &str) -> Option<&'static NavigationPageDefinition>
group_by_id(id: &str) -> Option<&'static NavigationGroupDefinition>
default_navigation_registry_json() -> String  // for FFI + surface.snapshot
```

---

## Configuration Details

### Config file path logic

Desktop: `~/.config/Arcadia/Configuration/` or `~/Arcadia/Configuration/` — see `config/mod.rs` `config_root_path()` for resolution order.

iOS: caller must set path via `set_config_root_path(path)` before any config reads. Call this in `ArcadiaApp.swift` before first `execute_command`.

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
- `NavigationRegistry` (serialized to JSON in `navigation.rs`) is the iOS navigation contract — iOS deserializes it, never hardcodes page/group lists.
- `ConfigFile::merge_defaults()` handles migration — when renaming a module, add the migration there.
- All cross-platform logic lives in `arcadia_core` — if you find yourself writing the same logic in both `app.rs` and `ContentView.swift`, it belongs in the core instead.
- `surface.*` is the protocol namespace for UI mirroring — do not create ad-hoc `remote-session.*` verbs.
- After FFI changes: always rebuild `ArcadiaCore.xcframework` and commit `Generated/`.

---

## Build Reference

```sh
# Desktop GUI
cd Desktop && cargo build --features gui
cd Desktop && cargo run --features gui

# Desktop headless (CLI)
cd Desktop && cargo run

# Core tests (artifacts under Builds/workspace via /.cargo/config.toml)
cargo test -p arcadia-core --manifest-path Shared/Cargo.toml

# iOS framework rebuild (after ffi.rs changes)
bash Shared/Scripts/Builds/build-ios-framework.sh

# Global CLI wrappers (macOS)
bash Shared/Scripts/Installers/install-global-commands-macos.sh
```

---

## Known Gotchas

- `surface.revision` only advances on `surface.patch` — CLI/FFI writes bypass it. Do not use revision as a reliable freshness signal until gap 1 in `gaps.md` is resolved.
- Multiple concurrent GUIs on the same host = last-write-wins on `modules.toml`. No merge semantics.
- LAN forwarding requires `remote-session`, `lan`, and `net` enabled locally. The peer checks its own module rules for the forwarded token.
- iOS `ArcadiaCore.xcframework` must be manually rebuilt after `ffi.rs` changes — no CI automation yet.
- `ARCADIA_NET_AS` env var overrides `thin-client.toml` `preferred_remote_route` on startup.
