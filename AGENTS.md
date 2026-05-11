# Arcadia — Agent Instructions

Read `CLAUDE.md` first. This file adds agent-specific rules, decision trees, and file ownership on top of it.

---

## Prime Directive: Registry-Driven, Not Hardcoded

Every module, every page, every group has **one registration point**. When asked to add, change, or remove any of these, touch the registry entry — let the rest of the system derive from it. Do not scatter the change across surface files.

**If the registry entry does not exist, create it before writing surface code.**

---

## Anti-Patterns — Refuse to Write These

### 1. Named module booleans

```rust
// NEVER add fields like these to ArcadiaRoot or any surface state
pub shell_enabled: bool,
pub lan_enabled: bool,
pub net_enabled: bool,

// NEVER add methods like these
fn shell_enabled(&self) -> bool { … }
fn net_enabled(&self) -> bool { … }
```

One method: `fn is_module_enabled(&self, name: &str) -> bool`. Query it with the `*_MODULE_NAME` constants from `config/modules.rs`.

### 2. Hardcoded page ID match arms in visibility logic

```rust
// NEVER write this pattern
match page_id {
    "utility.shell" => self.shell_enabled,
    "network.overview" => self.net_enabled(),
    _ => true,
}
```

Page visibility derives from `NavigationPageDefinition.required_module`, not surface-level match arms.

### 3. Growing if-else chains for page content dispatch

```rust
// NEVER grow this pattern
if self.active_page_id == "utility.shell" { … }
else if self.active_page_id == "global.modules" { … }
else if self.active_page_id == "network.overview" { … }  // don't add
```

If this pattern exists and you must add a page, flag it as technical debt before extending it.

### 4. Special-casing page IDs in generic event handlers

Lifecycle side-effects belong in the panel for that page, not in a global observer or dispatch match.

### 5. Inline raw colors in view/render code

```rust
// NEVER inline hex colors in app.rs or any view file
.bg(rgb(0x151a22))
.text_color(rgb(0x93c5fd))
```

All colors: `Desktop/src/gui/theme/mod.rs` or component files under `theme/modules/`.

### 6. Duplicating core logic across the codebase

If the same business logic ends up in `arcadia-core` and `Desktop/src/gui/app/`, it belongs in `arcadia-core` only. If the same logic ends up in `Desktop/src/main.rs` and `Mobile/iOS/ArcadiaApp/`, it belongs in `arcadia-core` or `gui/app/entry*.rs` — the platform shells must stay paper-thin.

### 7. Ad-hoc `remote-session.*` verbs

```
// NEVER create new remote-session.foo commands for UI mirroring.
// CORRECT — extend surface.snapshot extra fields and surface.patch ops.
```

The `remote-session` module is a routing gate only. `surface.*` is the protocol for UI state mirroring.

### 8. Config renames without migration

```
// NEVER rename a module name constant without adding a migration in:
//   ModulesConfig::merge_defaults() in config/modules.rs
// Follow the LEGACY_LAN_MODULE_NAME pattern.
```

### 9. Reviving the dead UniFFI/xcframework path

The previous `ffi.rs` + UniFFI + `Mobile/iOS/ArcadiaCore.xcframework` + `Generated/` flow has been removed. iOS now consumes `arcadia-core` directly via the `arcadia` package's `libarcadia_ios.a` (built with `--features ios-gui`), with a 2-function C ABI in `Desktop/src/ios_lib.rs`. Do not reintroduce UniFFI or the xcframework. Extend the C ABI in `ios_lib.rs` + `Mobile/iOS/ArcadiaApp/ArcadiaBridge.h` if more surface is required.

---

## Correct Extension Patterns

### Adding a module (zero surface edits required)

```rust
// 1. Shared/ArcadiaCore/src/config/modules.rs — add constant + registry entry
pub const FOO_MODULE_NAME: &str = "foo";

static MODULE_REGISTRY: &[ModuleManifest] = &[
    // … existing …
    ModuleManifest {
        name: FOO_MODULE_NAME,
        version: "1.0.0",
        description: "What foo does.",
        required_modules: &[],  // or &[NET_MODULE_NAME] etc.
    },
];

// 2. Create Shared/ArcadiaCore/src/modules/foo.rs
pub const NAME: &str = "foo";

pub fn commands() -> &'static [ModuleCommand] {
    &[
        ModuleCommand { name: "foo.bar", description: "Does bar.", run: run_bar },
    ]
}

// 3. Register in Shared/ArcadiaCore/src/modules/mod.rs:
//    `pub mod foo;` + add to module_commands() match
// Done — GUI + CLI module lists update automatically.
```

### Adding a navigation page

```rust
// 1. Shared/ArcadiaCore/src/navigation.rs — add to PAGE_DEFINITIONS
NavigationPageDefinition {
    id: "utilities.foo",
    title: "Foo",
    description: "Foo does things.",
    glyph: "foo",                            // must have a matching arm in icon_path()
    accent: "emerald",
    required_module: Some(FOO_MODULE_NAME),  // or None if always visible
},

// 2. Add "utilities.foo" to GROUP_DEFINITIONS.pages for the relevant group

// 3. Implement the panel under Desktop/src/gui/app/ — derive visibility from required_module
//    The panel renders on both desktop and iOS via OpenFrame.
```

### Checking module state in surface code

```rust
fn is_module_enabled(&self, name: &str) -> bool {
    self.module_rows.iter()
        .find(|(n, _)| n == name)
        .map(|(_, enabled)| *enabled)
        .unwrap_or(false)
}
// Call as: self.is_module_enabled(SHELL_MODULE_NAME)
```

### Adding mirrored state to thin-client protocol

```rust
// 1. modules/surface.rs — extend SurfaceSnapshot.extra
// 2. modules/surface.rs — add SurfacePatch variant if clients push changes back
// 3. The relevant panel consumes the new extra field from snapshot result
// 4. Do NOT create remote-session.foo verbs — keep protocol under surface.*
```

### Renaming a module

```rust
// 1. Edit MODULE_REGISTRY entry + constant in config/modules.rs
// 2. Add migration in ModulesConfig::merge_defaults():
const LEGACY_FOO_NAME: &str = "foo-old";
if let Some(val) = self.modules.remove(LEGACY_FOO_NAME) {
    self.modules.entry(FOO_MODULE_NAME.to_string()).or_insert(val);
}
// 3. Done — no ad-hoc renames at call sites
```

---

## Decision Tree Before Writing Code

Ask these questions. If any answer is "no," stop and fix it first.

1. **Does a registry entry exist for this?** → If not, create it before touching surface code.
2. **Am I adding a name check on a specific module or page ID in surface code?** → That logic belongs in the registry declaration or the core.
3. **Am I adding a new field/property that tracks a specific module's state?** → Use `is_module_enabled(name)` instead.
4. **Am I writing logic in `Desktop/src/main.rs` or `Mobile/iOS/ArcadiaApp/`?** → Almost certainly belongs in `arcadia-core` or `Desktop/src/gui/app/` instead.
5. **Am I inlining a color value?** → Put it in the theme layer.
6. **Did I change the iOS C ABI (`ios_lib.rs` exports)?** → Update `Mobile/iOS/ArcadiaApp/ArcadiaBridge.h` to match and commit both.
7. **Am I renaming a module?** → Add a `merge_defaults()` migration.
8. **Am I creating a new `remote-session.*` command for UI state?** → Use `surface.snapshot` / `surface.patch` instead.
9. **Am I about to reintroduce UniFFI / a `Generated/` directory / an `ArcadiaCore.xcframework`?** → No. That path is dead by design.

---

## When Asked to "Just Make It Work" With a Hardcoded Value

Do not. If time is the constraint, implement the proper registry-driven pattern and leave a `// TODO: <why this is debt>` comment — do not leave hardcoded strings in surface logic. A hardcoded page ID match arm today becomes five hardcoded match arms after the next change touches the file.

---

## File Ownership

| File | Purpose | Agent rule |
|------|---------|------------|
| `Shared/ArcadiaCore/src/config/modules.rs` | Module registry + config + migrations | Extend `MODULE_REGISTRY`; add migrations to `merge_defaults()`; never add per-module booleans |
| `Shared/ArcadiaCore/src/navigation.rs` | Page/group registry + JSON serialization | Extend `PAGE_DEFINITIONS` / `GROUP_DEFINITIONS`; never add parallel lists |
| `Shared/ArcadiaCore/src/modules/surface.rs` | Snapshot / patch / revision | Extend `extra` + `SurfacePatch`; do not create ad-hoc `remote-session.*` verbs |
| `Shared/ArcadiaCore/src/modules/remote_mirror.rs` | Host transcript queue + drain | For inbound NODE_EXEC mirroring only |
| `Shared/ArcadiaCore/src/modules/{shell,lan,net,late,...}.rs` | Module command handlers | One file (or one folder) per module; no cross-module logic |
| `Desktop/src/main.rs` | Desktop binary entry | Thin — pick GUI vs headless and hand off; no business logic |
| `Desktop/src/ios_lib.rs` | iOS C ABI surface | Keep small; mirror declarations in `Mobile/iOS/ArcadiaApp/ArcadiaBridge.h` |
| `Desktop/src/gui/app/mod.rs` | GUI root state (`ArcadiaRoot`) | No per-module booleans; no hardcoded page IDs |
| `Desktop/src/gui/app/entry.rs` | Desktop GPUI bootstrap | Window setup only |
| `Desktop/src/gui/app/entry_ios.rs` | iOS Metal-layer bootstrap | Wire `arcadia_ios_start` to OpenFrame on the supplied `CAMetalLayer` |
| `Desktop/src/gui/theme/mod.rs` | Icon + color helpers | All color/icon lookups; never inline in views |
| `Desktop/src/gui/tui/` | PTY/TUI terminal emulator | Desktop-feature only |
| `Mobile/iOS/ArcadiaApp/ArcadiaApp.swift` | UIKit @main | Only: configure Metal layer, call `arcadia_ios_start` with config root |
| `Mobile/iOS/ArcadiaApp/MetalHostView.swift` | `CAMetalLayer` host view | Only: forward `UITouch` events via `arcadia_ios_inject_touch` |
| `Mobile/iOS/ArcadiaApp/ArcadiaBridge.h` | C ABI declarations | Mirror of `Desktop/src/ios_lib.rs` exports |
| `Mobile/iOS/ArcadiaApp.xcodeproj/project.pbxproj` | Xcode project + cargo build phase | Build phase already shells out to cargo — do not duplicate it |

---

## Production Readiness Checklist

Before marking a feature ready for production, verify:

- [ ] New capability registered in `MODULE_REGISTRY` (if module) or `PAGE_DEFINITIONS` (if page)
- [ ] No hardcoded module/page IDs in surface visibility or dispatch logic
- [ ] No per-module boolean fields added to surface state structs
- [ ] No inline colors in view/render code
- [ ] iOS C ABI changes (`ios_lib.rs`) reflected in `ArcadiaBridge.h`
- [ ] Module rename includes `merge_defaults()` migration
- [ ] New mirrored state uses `surface.*` protocol, not ad-hoc verbs
- [ ] `cargo test -p arcadia-core --manifest-path Shared/Cargo.toml` passes
- [ ] Known gap addressed or documented in `Documentation/GAPS.md` if not fully solved

---

## LAN / Thin-Client Rules

- LAN command forwarding requires `remote-session` + `lan` + `net` enabled locally. The peer checks its own module rules.
- `surface.revision` is not a reliable freshness signal yet (see `Documentation/GAPS.md`). Do not build logic that assumes revision covers all write paths.
- `surface.patch` `client_id` is attribution only — not authentication. Do not build authorization logic on it.
- Multiple concurrent clients patching the same host = last-writer-wins. Do not imply merge semantics.

---

## Surface Parity Notes

The desktop and iOS surfaces render the same OpenFrame UI from `Desktop/src/gui/app/`. The only divergent code is:

| Concern | Desktop | iOS |
|---------|---------|-----|
| Entrypoint | `Desktop/src/main.rs` → `gui::app::entry::run()` | `ArcadiaApp.swift` → `arcadia_ios_start` → `gui::app::entry_ios::run()` |
| Window/layer | OS-native window via OpenFrame | `CAMetalLayer` hosted in `MetalHostView` |
| Input | OpenFrame's native event pump | `arcadia_ios_inject_touch` for `UITouch` |
| Shell (PTY/TUI) | Full via `Desktop/src/gui/tui/` (feature `gui`) | Not available (PTY deps gated behind desktop `gui` feature) — use `shell.execute` only |
| Config root | `$HOME/Arcadia/Configuration/` | App Documents dir set via `set_config_root` from `arcadia_ios_start` |

When implementing a new capability, prefer making it routable via `execute_command` so the same panel works on both surfaces and over LAN without platform-specific code.
