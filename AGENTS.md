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

```swift
// NEVER hardcode magic behavior on specific page IDs in generic handlers
.onChange(of: activePageID) { pageID in
    if pageID == "global.modules" { reloadModules() }
}
```

Lifecycle side-effects belong in the view for that page (e.g. `ModulesView.onAppear`), not in a global observer.

### 5. Inline raw colors in view/render code

```rust
// NEVER inline hex colors in app.rs or any view file
.bg(rgb(0x151a22))
.text_color(rgb(0x93c5fd))
```

```swift
// NEVER inline colors in SwiftUI views
Color(hex: "151a22")
```

Desktop colors: `Desktop/src/gui/theme/mod.rs` or component files under `theme/modules/`.
iOS colors: computed properties on `AppTheme` in `AppTheme.swift`.

### 6. Duplicating core logic in surface code

```
// NEVER write the same business logic in both Desktop/src/gui/app/ AND Mobile/iOS/ArcadiaApp/
// If it belongs to both, it belongs in arcadia-core.
```

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

### 9. FFI changes without rebuilding xcframework

```
// NEVER commit ffi.rs changes without running:
//   bash Shared/Scripts/Builds/build-ios-framework.sh
// and committing the updated Generated/ + ArcadiaCore.xcframework
```

---

## Correct Extension Patterns

### Adding a module (all platforms, zero surface edits required)

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
pub fn commands() -> &'static [ModuleCommand] {
    &[
        ModuleCommand { token: "foo.bar", description: "Does bar." },
    ]
}

// 3. Register in Shared/ArcadiaCore/src/modules/mod.rs
// Done — GUI, CLI, iOS module list updates automatically
```

### Adding a navigation page

```rust
// 1. Shared/ArcadiaCore/src/navigation.rs — add to PAGE_DEFINITIONS
NavigationPageDefinition {
    id: "utilities.foo",
    title: "Foo",
    description: "Foo does things.",
    glyph: "foo",              // must have a matching arm in icon_path()
    system_image: "star",      // SF Symbol for iOS
    accent: "emerald",
    required_module: Some(FOO_MODULE_NAME),  // or None if always visible
},

// 2. Add "utilities.foo" to GROUP_DEFINITIONS.pages for the relevant group

// 3. Desktop: add panel render + route via page ID (derive visibility from required_module)
// 4. iOS: add view + route in ContentView page dispatch
```

### Checking module state in surface code

```rust
// Rust (Desktop) — use MODULE_NAME constants, not string literals
fn is_module_enabled(&self, name: &str) -> bool {
    self.module_rows.iter()
        .find(|(n, _)| n == name)
        .map(|(_, enabled)| *enabled)
        .unwrap_or(false)
}
// Call as: self.is_module_enabled(SHELL_MODULE_NAME)
```

```swift
// Swift (iOS)
func isModuleEnabled(_ name: String) -> Bool {
    modules.first(where: { $0.name == name })?.enabled ?? false
}
// Call as: isModuleEnabled(ModuleNames.shell)
```

### Adding mirrored state to thin-client protocol

```rust
// 1. modules/surface.rs — extend SurfaceSnapshot.extra
// 2. modules/surface.rs — add SurfacePatch variant if clients push changes back
// 3. Desktop + iOS surfaces consume new extra field from snapshot result
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
2. **Am I adding a name check on a specific module or page ID in surface code?** → If yes, that logic belongs in the registry declaration or the core.
3. **Am I adding a new field/property that tracks a specific module's state?** → Use `is_module_enabled(name)` instead.
4. **Am I writing the same logic for both Desktop and iOS?** → Move it to `arcadia_core`.
5. **Am I inlining a color value?** → Put it in the theme layer.
6. **Did I change `ffi.rs` or any FFI-exported type?** → Run `Shared/Scripts/Builds/build-ios-framework.sh` before committing.
7. **Am I renaming a module?** → Add a `merge_defaults()` migration.
8. **Am I creating a new `remote-session.*` command for UI state?** → Use `surface.snapshot` / `surface.patch` instead.

---

## When Asked to "Just Make It Work" With a Hardcoded Value

Do not. If time is the constraint, implement the proper registry-driven pattern and leave a `// TODO: <why this is debt>` comment — do not leave hardcoded strings in surface logic. A hardcoded page ID match arm today becomes five hardcoded match arms after the next change touches the file.

---

## File Ownership

| File | Purpose | Agent rule |
|------|---------|------------|
| `config/modules.rs` | Module registry + config + migrations | Extend `MODULE_REGISTRY`; add migrations to `merge_defaults()`; never add per-module booleans |
| `navigation.rs` | Page/group registry + JSON serialization | Extend `PAGE_DEFINITIONS` / `GROUP_DEFINITIONS`; never add parallel lists |
| `ffi.rs` | UniFFI bridge | After changes: rebuild xcframework; commit `Generated/` |
| `modules/surface.rs` | Snapshot / patch / revision | Extend `extra` + `SurfacePatch`; do not create ad-hoc `remote-session.*` verbs |
| `modules/remote_mirror.rs` | Host transcript queue + FFI drain | For inbound NODE_EXEC mirroring only |
| `modules/shell.rs`, `modules/lan/`, etc. | Module command handlers | One file per module; no cross-module logic |
| `gui/app/mod.rs` | Desktop root state (`ArcadiaRoot`) | No per-module booleans; no hardcoded page IDs |
| `gui/theme/mod.rs` | Desktop icon + color helpers | All Desktop color/icon lookups; never inline in views |
| `gui/tui/` | PTY/TUI terminal emulator | Desktop-specific; no equivalent on iOS (shell.execute only) |
| `AppTheme.swift` | iOS color tokens | All iOS colors as computed properties |
| `ContentView.swift` | iOS coordinator | Thin — registry + module state consumer; no business logic |
| `NavigationModels.swift` | Swift nav types | Mirror of Rust `NavigationPageDefinition` / `NavigationGroupDefinition`; update after navigation.rs changes |
| `ModuleNames.swift` | iOS module name constants | Mirror of `MODULE_REGISTRY` name fields; update after adding modules |

---

## Production Readiness Checklist

Before marking a feature ready for production, verify:

- [ ] New capability registered in `MODULE_REGISTRY` (if module) or `PAGE_DEFINITIONS` (if page)
- [ ] No hardcoded module/page IDs in surface visibility or dispatch logic
- [ ] No per-module boolean fields added to surface state structs
- [ ] No inline colors in view/render code
- [ ] FFI changes accompanied by `xcframework` rebuild + `Generated/` commit
- [ ] Module rename includes `merge_defaults()` migration
- [ ] New mirrored state uses `surface.*` protocol, not ad-hoc verbs
- [ ] `cargo test -p arcadia-core` passes
- [ ] Known gap addressed or documented in `Documentation/GAPS.md` if not fully solved

---

## LAN / Thin-Client Rules

- LAN command forwarding requires `remote-session` + `lan` + `net` enabled locally. The peer checks its own module rules.
- `surface.revision` is not a reliable freshness signal yet — gap 1 in `Documentation/GAPS.md`. Do not build logic that assumes revision covers all write paths.
- `surface.patch` `client_id` is attribution only — not authentication. Do not build authorization logic on it.
- Multiple concurrent clients patching the same host = last-writer-wins. Do not imply merge semantics.

---

## Surface Parity Notes

| Capability | Desktop | iOS | Core |
|-----------|---------|-----|------|
| Shell (PTY/TUI) | Full PTY + TUI via `gui/tui/` | `shell.execute` only | `modules/shell.rs` |
| Shell MOTD | Yes | Yes (via execute) | `modules/shell_motd.rs` |
| Module toggles | GUI + CLI | SwiftUI `ModulesView` | FFI `set_module_enabled*` |
| LAN discovery | `lan_nodes/` panel | `LanNodesView` | `modules/lan/` |
| Surface snapshot | Yes | Yes | `modules/surface.rs` |
| Remote mirror drain | Yes (shell/mirror.rs) | Yes (250ms timer) | FFI `drain_remote_mirror_batch` |
| Thin-client route | Session chip in top bar | Route picker in sidebar | `ThinClientConfig` |
| Splash screen | Animated canvas (`splash/`) | `SplashView.swift` | — |

Divergence between surfaces is tracked in gap 10 of `Documentation/GAPS.md`. When implementing a new capability, prefer making it routable via `execute_command` so both surfaces can reach it over LAN without platform-specific implementations.
