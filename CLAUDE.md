# Arcadia — Claude Code Guide

## What Arcadia Is

Multi-platform runtime and shell: one Rust core (`Shared/ArcadiaCore`) consumed by a single GPUI-style UI layer (`Libraries/OpenFrame`) rendered on three surfaces — desktop (`Desktop/` GUI feature), iOS (Metal-hosted OpenFrame via `--features ios-gui`), and a headless CLI. The core owns all logic; the UI layer renders; the surfaces only initialize and dispatch input.

**Arcadia is a permission and privacy control layer.** Giving the user full, visible control over every permission and privacy-sensitive capability is the core purpose of the app. Every OS capability the app touches is represented in `PERMISSION_REGISTRY` and gated on it — see "OS capabilities outside the permission system" under What Not to Do. This is not an add-on; it is what Arcadia is for.

**Key invariant:** if you find yourself writing the same logic in more than one place, it belongs in `arcadia-core` (logic) or `gui/app/` (presentation) — not in surface entrypoints.

---

## Repository Layout

```
Shared/ArcadiaCore/src/
  lib.rs              module declarations (config, modules, navigation, platform, services)
  navigation.rs       PAGE_DEFINITIONS, GROUP_DEFINITIONS, NavigationRegistryOwned
  services.rs         SERVICE_DEFINITIONS, service-host helpers
  config/
    mod.rs            ConfigFile trait, CONFIG_ROOT_OVERRIDE (iOS sets this)
    modules.rs        MODULE_REGISTRY, ModulesConfig, merge_defaults() migrations
    workspace.rs      WorkspacesConfig, WorkspaceEntry, WorkspacePermissionDef → workspace.toml
    appearance.rs     appearance.toml (theme tokens)
    commandline.rs    CLI preferences
    extension_tokens.rs  extension theme/glyph token resolution
    late.rs           late.toml (WS chat config)
    thin_client.rs    thin-client.toml (preferred_remote_route, surface_client_id, navigation_from_host_only)
  modules/
    mod.rs            execute_command dispatcher, module_commands() lookup
    shell.rs          shell.execute + shell.internal
    shell_motd.rs     MOTD banner (terminal-motd module)
    surface.rs        surface.snapshot / surface.patch / revision counter
    remote_session.rs routing manifest only (no standalone commands)
    remote_mirror.rs  host transcript queue + drain (for surface mirroring)
    net.rs            networking foundation
    lan/              LAN subsystem (discovery, peers, protocol, handlers)
    late.rs           WebSocket chat / now-playing / votes module
    python_host.rs    Python extension host
    python_registry.rs Python extension manifest registry
    workspace.rs      workspace.list/add/remove/grant/revoke/check commands
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
        workspace_panel.rs    Workspaces page — list + search
        workspace_row.rs      per-workspace row with permission toggles
        workspace_create_modal.rs  create-workspace modal (label + path fields)
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

Configuration/  (runtime root: $HOME/Arcadia/Configuration on Desktop, app Documents on iOS)
  modules.toml        module enable/disable state
  commandline.toml    CLI settings
  thin-client.toml    preferred_remote_route, surface_client_id (UUID), navigation_from_host_only
  appearance.toml     theme tokens
  late.toml           late module state
  workspace.toml      registered workspace directories + per-workspace permission grants

Documentation/Features/   in-depth feature docs — keep in sync with code changes
  README.md           index table
  AI.md               AI providers, tools, sandbox, allowlist
  Appearance.md       token system, accent palettes, extension style tokens
  CodeEditor.md       syntax highlight + decoration providers
  Extensions.md       Python host lifecycle, style tokens, overlay companion
  LAN_and_Remote.md   UDP protocol, routing, surface snapshot/patch, thin client
  Late.md             domain types, LateState, GUI panels
  Modules.md          full module table, manifest fields, dep enforcement, migrations
  Navigation.md       all pages/groups, special lists, visibility rule
  Overlay_and_Cursor.md  stacking tokens, cursor backend
  Permissions.md      subject model, grants, global vs workspace permissions
  Services.md         ServiceDefinition, controls, port collision
  Shortcuts.md        trigger types, overrides, custom, OS-global consent
  Terminal.md         PTY vs shell.execute, mirror mode, platform matrix
  Workspaces.md       entry schema, commands, AI context, scope check
```

---

## Documentation Maintenance

`Documentation/Features/` contains in-depth feature docs derived from the source. **Update the relevant file(s) whenever you:**

- Add, rename, or remove a module — update `Modules.md` module table.
- Add or change a navigation page or group — update `Navigation.md`.
- Add a new AI provider or change the sandbox/allowlist — update `AI.md`.
- Change workspace permission model or commands — update `Workspaces.md`.
- Change LAN protocol, surface snapshot schema, or thin-client config — update `LAN_and_Remote.md`.
- Add or change extension capabilities (style tokens, overlay, highlight) — update `Extensions.md`.
- Change Late.sh domain types or state shape — update `Late.md`.
- Change shortcut types, actions, or OS-global flow — update `Shortcuts.md`.
- Add a service to `SERVICE_DEFINITIONS` — update `Services.md`.
- Change overlay stacking tokens or cursor backend — update `Overlay_and_Cursor.md`.
- Change appearance config or accent palettes — update `Appearance.md`.
- Change permissions model or subject types — update `Permissions.md`.
- Change terminal PTY or shell execution — update `Terminal.md`.
- Change code editor extension APIs — update `CodeEditor.md`.

The docs are written from the source. If source and doc disagree, **the source is authoritative** — fix the doc.

---

## Module Quick Reference

All modules live in `MODULE_REGISTRY` (`config/modules.rs`). Commands follow `module.verb` format.

| Module | Constant | Requires | Commands / Purpose |
|--------|----------|----------|--------------------|
| `animation` | `ANIMATION_MODULE_NAME` | — | Shared tween engine; 16 ms driver loop |
| `net` | `NET_MODULE_NAME` | — | Networking foundation |
| `lan` | `LAN_MODULE_NAME` | `net` | `lan.scan`, `lan.node`, `lan.session_targets` — LAN discovery + pairing |
| `surface` | `SURFACE_MODULE_NAME` | — | `surface.snapshot`, `surface.patch`, `surface.revision` |
| `remote-session` | `REMOTE_SESSION_MODULE_NAME` | `net`, `lan` | Routing gate for `net_as` forwarding; no standalone commands |
| `terminal` | `TERMINAL_MODULE_NAME` | — | `shell.execute` (routable), `shell.internal` (REPL), PTY/TUI on Desktop |
| `terminal-motd` | `TERMINAL_MOTD_MODULE_NAME` | `terminal` | Fastfetch-style banner on terminal open |
| `late` | `LATE_MODULE_NAME` | — | `late.*` — chat rooms, music stream, reactions |
| `python-host` | `PYTHON_HOST_MODULE_NAME` | — | Python extension loader; scans `~/Arcadia/Extensions/` |
| `permissions` | `PERMISSIONS_MODULE_NAME` | — | Permission catalog, grants, `permit`/`list` commands |
| `tray` | `TRAY_MODULE_NAME` | — | Menu-bar (macOS) / system-tray (Windows/Linux) |
| `cursor` | `CURSOR_MODULE_NAME` | — | OS-global cursor position + display size (desktop only) |
| `overlay` | `OVERLAY_MODULE_NAME` | — | Always-on-top transparent HUD (desktop only) |
| `workspace` | `WORKSPACE_MODULE_NAME` | — | `workspace.add/remove/list/grant/revoke/check` — directory registry + permissions |

---

## Navigation Quick Reference

All pages in `PAGE_DEFINITIONS` (`navigation.rs`). Visibility from `required_module` — never hardcode.

| Page ID | Title | Placement | Required Module |
|---------|-------|-----------|-----------------|
| `utility.shell` | Terminal | `utilities` group | `terminal` |
| `utility.services` | Services | `utilities` group | _service-driven_ |
| `global.dashboard` | Dashboard | sidebar global | — |
| `global.settings` | Settings | sidebar global + hub root | — |
| `global.logs` | Logs | app-title context menu | — |
| `global.modules` | Modules | top bar | — |
| `global.appearance` | Appearance | settings hub | — |
| `global.permissions` | Permissions | settings hub | — |
| `global.shortcuts` | Shortcuts | settings hub | — |
| `global.workspaces` | Workspaces | settings hub | `workspace` |
| `network.nodes` | Nodes | `network` group | `lan` |
| `late.now_playing` | Late.sh | `social` group | `late` |
| `late.experimental` | Experimental | `social` group | `late` |
| `late.settings` | Late.sh settings | settings hub | `late` |
| `python.settings` | Extensions | top bar | `python-host` |

**Groups:** `utilities` (shell, services) · `network` (nodes) · `social` (late pages)

**Special lists:**
- `GLOBAL_PAGE_IDS` → sidebar global: `global.dashboard`, `global.settings`
- `TOP_BAR_PAGE_IDS` → top bar: `python.settings`, `global.modules`
- `SETTINGS_HUB_ROOT_PAGE_ID` → `global.settings`
- `SETTINGS_HUB_PAGE_IDS` → `global.permissions`, `global.shortcuts`, `global.appearance`, `global.workspaces`, `late.settings`

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
| Workspace registry | `config/workspace.rs` `WorkspacesConfig` | workspace module + GUI panel |

### Thin surfaces, fat core, single UI layer

- `arcadia-core` owns business logic, state, commands.
- `Desktop/src/gui/` (rendered via OpenFrame) is the UI. It runs identically on desktop and iOS — only the platform shell differs.
- `Desktop/src/main.rs` and `Mobile/iOS/ArcadiaApp/` only initialize the runtime and dispatch native input.

Surface entrypoints must NOT:
- Re-implement business logic that belongs in `arcadia_core`.
- Hard-code module names, page IDs, or feature flags.
- Add per-module booleans (`shell_enabled`, `net_enabled`) — query dynamically via `is_module_enabled(name)`.
- Duplicate navigation structure that already exists in `navigation.rs`.

---

## How to Add Things

### New module

1. Add `pub const X_MODULE_NAME: &str = "x";` to `Shared/ArcadiaCore/src/config/modules.rs`.
2. Add a `ModuleManifest` entry to `MODULE_REGISTRY` in the same file (all 6 fields — see struct below).
3. Create `Shared/ArcadiaCore/src/modules/x.rs` with a `commands()` fn returning `&[ModuleCommand]`.
4. Register in `Shared/ArcadiaCore/src/modules/mod.rs`: `pub mod x;` + add to `module_commands()` match.
5. Done — GUI and CLI pick it up from the registry automatically. No surface edits required.

### New navigation page

1. Add a `NavigationPageDefinition` entry to `PAGE_DEFINITIONS` in `navigation.rs`. Set `required_module` if the page depends on a module.
2. Add the page ID to the relevant `NavigationGroupDefinition.pages` slice, or add to `SETTINGS_HUB_PAGE_IDS`, `GLOBAL_PAGE_IDS`, or `TOP_BAR_PAGE_IDS` as appropriate.
3. Implement the page panel under `Desktop/src/gui/app/` (single implementation — renders on desktop and iOS).
4. Route in the surface content switch — derive visibility from `required_module`, **never** add a hardcoded match arm.

### New icon/glyph

1. Add SVG to `Desktop/assets/icons/`.
2. Add a match arm to `icon_path()` in `Desktop/src/gui/theme/icons.rs`.
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
// call as: self.is_module_enabled(TERMINAL_MODULE_NAME)
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

### Secrets in channel payloads or enum variants

```rust
// BAD — API key travels through mpsc channel, lives on heap unencrypted,
// appears in core dumps and channel debug output
enum ProviderRouting {
    OpenAi { api_key: String, model_id: String },
}

// GOOD — routing carries only non-secret routing metadata;
// the inference thread loads the key directly at point of use
enum ProviderRouting {
    OpenAi { model_id: String },  // key loaded in inference thread via OpenAiConfig::load_or_create()
}
```

**Rule:** No credential (API key, token, password) in any `enum` variant, struct field that crosses a thread boundary, or message payload. Load secrets at the point of use.

### Path scope checks using string prefix matching

```rust
// BAD — @/workspace/../../../etc/passwd bypasses this check
pub fn is_path_in_scope(&self, path: &str) -> bool {
    path.starts_with(&self.workspace_path)
}

// GOOD — canonicalize resolves symlinks and .. before comparing
pub fn is_path_in_scope(&self, path: &str) -> bool {
    let Ok(canon_ws) = std::fs::canonicalize(&self.workspace_path) else { return false; };
    let canon = if let Ok(c) = std::fs::canonicalize(path) { c } else {
        let p = std::path::Path::new(path);
        let Ok(cp) = std::fs::canonicalize(p.parent().unwrap_or(p)) else { return false; };
        cp.join(p.file_name().unwrap_or_default())
    };
    canon.starts_with(&canon_ws)
}
```

**Rule:** Never use `starts_with()` on raw path strings for security decisions. Always canonicalize first.

### Unrestricted `sh -c` execution

```rust
// BAD — grants AI ability to run rm -rf, curl | sh, sudo, etc.
Command::new("sh").arg("-c").arg(model_supplied_cmd).output()

// GOOD — allowlist enforced in ai_sandbox::sandboxed_exec before sh -c
// EXEC_ALLOWLIST in ai_sandbox.rs defines permitted binary names
```

**Rule:** Any execution of user- or model-supplied shell commands must go through `sandboxed_exec()` in `ai_sandbox.rs`. The allowlist there is the single gate. Do not bypass it.

### OS capabilities outside the permission system

**Arcadia is a permission and privacy control layer.** Giving the user full, visible control over every permission and privacy-sensitive capability is the entire point of the app. The permission system (`PERMISSION_REGISTRY`) is not a feature *of* Arcadia — it *is* Arcadia. Therefore: **every OS capability the app touches is represented in `PERMISSION_REGISTRY`, by definition.** There is no such thing as a correct ungated OS-capability call.

```rust
// BAD — CGWindowListCopyWindowInfo silently requires macOS Screen Recording.
// Nothing in PERMISSION_REGISTRY represents it → the user has no visibility
// or control over a privacy-sensitive capability the app just used. This
// defeats the product's entire purpose.
let windows = unsafe { CGWindowListCopyWindowInfo(ON_SCREEN_ONLY, 0) };

// GOOD — the capability lives in the permission system.
// 1. PERMISSION_REGISTRY entry (config/permissions.rs), with `system_grant`
//    when an OS-level grant flow is involved.
// 2. The call site checks the permission before invoking the OS API.
// 3. The GUI exposes the toggle (+ Grant/Revoke for the OS grant), so the
//    user sees and controls it.
```

**Rule:** Any OS API that depends on a system-level permission or touches a privacy-sensitive capability — macOS Accessibility, Screen Recording, Camera, Microphone, Location, Input Monitoring, Automation, Full Disk Access, Contacts, Calendar, etc. — MUST be represented by an entry in `PERMISSION_REGISTRY` (`Shared/ArcadiaCore/src/config/permissions.rs`) and gated on it. Add the permission first; set `system_grant` (add a `SystemGrant` variant if none fits) when an OS grant flow is needed. Calling such an API ungated is not a missed checklist item — it is a direct violation of what Arcadia exists to do. If a capability cannot be registered and surfaced to the user, the app does not use it.

### HTTP calls without timeouts

```rust
// BAD — server hang = UI hangs forever; inference thread blocks indefinitely
ureq::post(&url).set("Content-Type", "application/json").send_json(&body)

// GOOD — 120s ceiling; thread unblocks, error propagates to UI
const HTTP_TIMEOUT: Duration = Duration::from_secs(120);
let agent = ureq::AgentBuilder::new().timeout(HTTP_TIMEOUT).build();
agent.post(&url).set("Content-Type", "application/json").send_json(&body)
```

**Rule:** Every outbound HTTP call (AI providers, LAN APIs, webhooks) must use an `AgentBuilder` with an explicit timeout. `ureq::post()` with no agent is banned in production paths.

### `expect()` / `unwrap()` in production code paths

```rust
// BAD — panics on thread creation failure, crashes the whole process
std::thread::spawn(move || work()).expect("failed to spawn thread")

// GOOD — degrade gracefully; surface error to user if appropriate
match std::thread::Builder::new().spawn(move || work()) {
    Ok(_) => { /* started */ }
    Err(e) => { eprintln!("thread spawn failed: {e}"); /* show error in UI */ }
}
```

**Rule:** `expect()` and `unwrap()` are only permitted inside `#[cfg(test)]` blocks and in static initializers that provably cannot fail. Everywhere else: propagate `Result`/`Option` or degrade gracefully with a user-visible error.

### Silent error swallowing with `.ok()`

```rust
// BAD — config corruption silently disables tools with no feedback to user
WorkspacesConfig::load_or_create().ok()
    .and_then(|cfg| cfg.workspaces.into_iter().find(...))

// GOOD — surface the error; log it, show a banner, or push an error message
match WorkspacesConfig::load_or_create() {
    Ok(cfg) => cfg.workspaces.into_iter().find(...),
    Err(e) => { eprintln!("config load failed: {e}"); None }
}
```

**Rule:** `.ok()` on a `Result` that represents a user-visible operation is not acceptable. Either propagate the error or log it explicitly.

### Monolith files over 400 lines

Files over 400 lines in `Shared/ArcadiaCore/src/modules/` or `Desktop/src/gui/app/` are a split candidate. Files over 600 lines are blocked without a documented split plan. The current known violations are tracked in `Documentation/ROADMAP.md`.

---

## Naming and Generality Rules

These rules exist because Arcadia is a general-purpose platform. Every name is a contract — a name that references a specific service, language, OS, or UI shape leaks an assumption that will break when the platform grows.

### No platform names in cross-platform types

Never add a variant, field, or function name that references a specific OS to any type in `Shared/ArcadiaCore/`. Platform-specific behaviour lives only in `platform/*.rs` and surface entrypoints.

```rust
// BAD — macOS concept as a first-class variant in a cross-platform enum
pub enum OverlayStackingToken {
    Hud,
    BelowMenuBar,  // macOS menu bar doesn't exist on iOS or Windows
}

// GOOD — semantic name; the macOS backend maps it to NSWindowLevel 24 internally
pub enum OverlayStackingToken {
    Hud,
    SystemEdge,  // "the edge owned by the platform's system chrome"
}
```

### No service or brand names in module manifests or navigation

Module names, page IDs, page titles, config keys, and field names must describe **what the feature does**, not which external service it connects to.

```rust
// BAD — brand name in navigation title and module description
NavigationPageDefinition { id: "late.now_playing", title: "Late.sh", … }
ModuleManifest { description: "Native late.sh client — chat rooms, …", … }

// GOOD — describes the capability
NavigationPageDefinition { id: "social.now_playing", title: "Social", … }
ModuleManifest { description: "Real-time chat rooms, music stream, and reactions.", … }
```

### No language name in the extension namespace

Extension page ID prefixes, config field names, and directory names must not embed a language name. The extension host is language-agnostic at the platform layer.

```rust
// BAD — language name in page ID prefix and config field
pub const EXTENSION_TOKEN_SETTINGS_PAGE_PREFIX: &str = "python.extension_tokens|";
pub python_extensions: BTreeMap<String, bool>,

// GOOD — language-neutral
pub const EXTENSION_TOKEN_SETTINGS_PAGE_PREFIX: &str = "extension.tokens|";
pub extension_state: BTreeMap<String, bool>,
```

### Feature-prefixed flat root fields are banned

Top-level view state structs (`ArcadiaRoot`) must not accumulate `feature_*` flat fields. New subsystem state goes in a dedicated nested struct.

```rust
// BAD — every new feature adds N more fields to the root; 95+ fields today
pub code_editor_focus: FocusHandle,
pub code_editor_tabs: Vec<CodeEditorTab>,
pub code_editor_undo: EditorUndoMap,
// … 21 more code_editor_* fields …

// GOOD — nested substruct; ArcadiaRoot stays slim
pub code_editor: CodeEditorUiState,

pub struct CodeEditorUiState {
    pub focus: FocusHandle,
    pub tabs: Vec<CodeEditorTab>,
    pub undo: EditorUndoMap,
    // …
}
```

### Theme functions named after consumers are banned

Theme colour/style helpers must express **semantic purpose**, never the specific feature currently using them.

```rust
// BAD — name leaks the consumer; visual editor can't reuse without confusion
pub fn code_explorer_sidebar_bg(is_dark: bool) -> Hsla { … }
pub fn late_bonsai_well_bg(is_dark: bool) -> Hsla { … }

// GOOD — semantic; any feature with a sidebar or decorative inset can use it
pub fn explorer_sidebar_bg(is_dark: bool) -> Hsla { … }
pub fn decorative_well_bg(is_dark: bool) -> Hsla { … }
```

### Magic string enum values are banned

When a field accepts a finite set of values, use a typed enum. Magic strings are undiscoverable and break silently.

```rust
// BAD — 8 undocumented magic values; typos silently fall back to default
pub struct OverlayHudSpritePayload {
    pub anchor: String,   // "top-right" | "bottom-center" | "top-full" | …
    pub stacking: String, // "hud" | "below_menu_bar"
}

// GOOD — exhaustive, compiler-checked
pub enum SpriteAnchor {
    TopRight, TopLeft, TopCenter, TopFull,
    BottomRight, BottomLeft, BottomCenter, BottomFull,
}
pub struct OverlayHudSpritePayload {
    pub anchor: SpriteAnchor,
    pub stacking: OverlayStackingToken,
}
```

### Provider/editor-specific duplicate state patterns are banned

Adding a new AI provider, editor type, or collaboration service must not require adding new named fields to a root struct. Use a keyed collection.

```rust
// BAD — 9 fields per provider; adding a 4th provider adds 9 more root fields
pub openai_create_draft: OpenAiCreateDraft,
pub openai_create_name_focus: FocusHandle,
pub llama_cpp_create_draft: LlamaCppCreateDraft,
pub llama_cpp_create_name_focus: FocusHandle,

// GOOD — one field handles all providers
pub provider_create_drafts: HashMap<String, ProviderCreateDraft>,
pub provider_focus_handles: HashMap<(String, FieldKey), FocusHandle>,
```

### Hardcoded page ID comparisons for routing/layout are banned

Visibility rules, layout decisions, and sidebar expansion state must come from `NavigationPageDefinition` metadata — not from `if active_page_id == "some.page"` chains. New pages must not require editing routing functions.

```rust
// BAD — every new full-height page requires adding another arm here
fn content_padding(&self) -> Option<Padding> {
    if self.active_page_id == "editor.main" { return None; }
    if self.active_page_id == "editor.visual" { return None; }
    if self.active_page_id == "utility.shell" { return None; }
    Some(Padding::default())
}

// GOOD — declared once in NavigationPageDefinition; routing reads it
pub struct NavigationPageDefinition {
    pub layout_kind: PageLayoutKind, // FullHeight, Standard, etc.
    // …
}

fn content_padding(&self) -> Option<Padding> {
    let page = navigation::page_by_id(&self.active_page_id)?;
    if page.layout_kind == PageLayoutKind::FullHeight { None } else { Some(Padding::default()) }
}
```

---

## AI Module Patterns

### Adding a new AI provider

1. Add provider module to `MODULE_REGISTRY` with `AI_MODULE_NAME` as a dependency.
2. Add a `ProviderRouting` variant with only non-secret routing metadata (endpoint, model ID). **No API keys.**
3. Load credentials in the inference thread via `ProviderConfig::load_or_create()` at call time.
4. Add the binary names your provider uses to `EXEC_ALLOWLIST` in `ai_sandbox.rs` if needed.
5. Use `ureq::AgentBuilder::new().timeout(HTTP_TIMEOUT).build()` for all HTTP calls.
6. Route all file/exec operations through `sandboxed_read` / `sandboxed_write` / `sandboxed_exec` in `ai_sandbox.rs` — never directly call `std::fs` or `Command` from the provider.

### AI sandbox architecture

```
ai_chat_panel.rs        — builds TextGenerationRequest, resolves workspace context
ai_runtime.rs           — inference thread; dispatches to provider fns
  └── run_ollama()      — HTTP, 120s timeout, no credentials in routing
  └── run_openai()      — loads API key from config; HTTP, 120s timeout
  └── run_llama_cpp()   — local model; lazy-loads via LlamaCppConfig path
ai_sandbox.rs           — SINGLE gate for all file/exec ops from AI
  └── sandboxed_read()  — checks scope + workspace.read permission
  └── sandboxed_write() — checks scope + workspace.write permission
  └── sandboxed_exec()  — checks EXEC_ALLOWLIST + workspace.execute permission
  └── EXEC_ALLOWLIST    — permitted binary names (cargo, npm, python, git, …)
ai_context.rs           — @mention parsing, file context injection
ai_types.rs             — AiWorkspaceContext (is_path_in_scope uses canonicalize)
ai_tools.rs             — tool definitions and execution dispatch
```

**Never** call `std::fs::read_to_string` or `Command::new` directly from a provider or tool handler. Always go through `ai_sandbox`.

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
Host returns: { modules: [...], revision: N, extra: { schema_version: 1, navigation_registry: "..." } }

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
    pub name: &'static str,                                    // unique key, e.g. "terminal"
    pub version: &'static str,
    pub description: &'static str,
    pub required_modules: &'static [&'static str],             // transitive deps enforced on enable
    pub required_permissions: &'static [&'static str],         // global permissions needed
    pub workspace_permissions: &'static [WorkspacePermissionDef], // per-workspace grant toggles
    pub supported_platforms: &'static [&'static str],          // empty = all platforms
}
```

`workspace_permissions` advertises per-directory grants this module needs. The workspace panel iterates all enabled modules and renders their entries as per-workspace toggle rows.

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
    pub glyph: &'static str,                    // key into icon_path() in theme/icons.rs
    pub system_image: &'static str,             // SF Symbol name for iOS native contexts
    pub accent: &'static str,                   // accent palette key
    pub required_module: Option<&'static str>,  // drives visibility; None = always visible
}
```

### Key functions

```rust
page_by_id(id: &str) -> Option<&'static NavigationPageDefinition>
group_by_id(id: &str) -> Option<&'static NavigationGroupDefinition>
default_navigation_registry_json() -> String  // emitted via surface.snapshot.extra
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

- `surface.revision` advances on every `ModulesConfig::save` — all write paths bump it, including CLI. GUI polls `surface.revision` on a timer when a remote route is active and shows a stale banner + Reload when it diverges.
- Multiple concurrent GUIs on the same host = last-write-wins on `modules.toml`. No merge semantics.
- LAN forwarding requires `remote-session`, `lan`, and `net` enabled locally. The peer checks its own module rules for the forwarded token.
- `ARCADIA_NET_AS` env var overrides `thin-client.toml` `preferred_remote_route` on startup.
- `thin-client.toml` flag `navigation_from_host_only = true`: when set and a LAN route is active, the GUI uses only host `surface.snapshot` navigation — no fallback to compiled-in nav until the snapshot supplies `navigation_registry`.
- iOS surface is OpenFrame rendered into a `CAMetalLayer`, not SwiftUI. The Swift host is a ~50-line UIKit shell (`ArcadiaApp.swift` + `MetalHostView.swift`) whose only job is to provide a Metal layer and forward `UITouch` events. All UI logic lives in `Desktop/src/gui/app/` (via OpenFrame on iOS).
- `surface.patch` `client_id` is attribution only — not authentication or authorization. Do not build access control on it.
- The UniFFI / `Generated/` / `ArcadiaCore.xcframework` path is dead. iOS uses `libarcadia_ios.a` via a 2-function C ABI in `ios_lib.rs`. Do not reintroduce it.
