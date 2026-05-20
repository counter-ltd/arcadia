# Module System

Config file: `modules.toml`  
Source: `Shared/ArcadiaCore/src/config/modules.rs`  
Platform: all

---

## Overview

All Arcadia features are gated by a module system. Every module has a name, a version, a description, optional required modules (dependency chain), optional required permissions, optional workspace permission declarations, and an optional platform allowlist. Modules are toggled on/off in `modules.toml` and the GUI Modules page.

The module registry (`MODULE_REGISTRY`) is the single source of truth for module availability. The GUI, CLI, and navigation system all derive feature availability from this registry — nothing is hardcoded at call sites.

---

## Module manifest

```rust
pub struct ModuleManifest {
    pub name: &'static str,
    pub version: &'static str,
    pub description: &'static str,
    pub required_modules: &'static [&'static str],
    pub required_permissions: &'static [&'static str],
    pub workspace_permissions: &'static [WorkspacePermissionDef],
    pub supported_platforms: &'static [&'static str],
}
```

| Field | Purpose |
|-------|---------|
| `name` | Unique stable key (e.g. `"terminal"`, `"lan"`) |
| `version` | Semver string for display |
| `description` | Short human description shown on the Modules page |
| `required_modules` | Modules that must be enabled before this one can be enabled. Checked transitively. |
| `required_permissions` | Global permissions this module needs |
| `workspace_permissions` | Per-workspace permission toggles advertised to the Workspaces page |
| `supported_platforms` | Empty = all platforms. Otherwise: `["macos", "windows", "linux", "ios"]` |

---

## All modules

| Name | Deps | Platforms | Purpose |
|------|------|-----------|---------|
| `animation` | — | all | Shared 16ms tween engine |
| `net` | — | all | Shared networking foundation |
| `lan` | `net` | all | UDP LAN discovery + peer communication |
| `remote-session` | `net`, `lan` | all | Gate for `execute_command` forwarding over LAN |
| `surface` | — | all | UI snapshot (`surface.snapshot`) + patch (`surface.patch`) |
| `terminal` | — | all | `shell.execute` + PTY/TUI on desktop |
| `terminal-motd` | `terminal` | all | Fastfetch-style MOTD banner |
| `late` | — | all | Late.sh client: chat, music, reactions, bonsai |
| `python-host` | — | all | Python extension loader from `~/Arcadia/Extensions/` |
| `permissions` | — | all | Permission catalog + `permit`/`list` CLI commands |
| `tray` | — | macOS, Windows, Linux | Menu-bar / system-tray icon |
| `cursor` | — | macOS, Windows, Linux | OS-global cursor position + screen size |
| `keyboard` | — | macOS | OS-global key / mouse / scroll event stream for extensions |
| `audio` | — | macOS, Windows, Linux | Low-latency audio output with DSP node graphs and sample playback |
| `overlay` | — | macOS, Windows, Linux | Always-on-top transparent HUD window |
| `workspace` | — | all | Workspace directory registry + scoped permissions |
| `code-editor` | — | all | Code editor panel |
| `visual-editor` | — | all | Scratch-style visual block editor for Python |
| `ai` | — | all | AI provider registry + base types |
| `ai-provider-llama-cpp` | `ai` | all | Local inference via llama.cpp |
| `ai-provider-ollama` | `ai` | all | Local inference via Ollama HTTP API |
| `ai-provider-openai` | `ai` | all | Cloud inference via OpenAI API |

---

## `ModulesConfig` methods

| Method | Purpose |
|--------|---------|
| `manifest_for(name)` | Lookup manifest by module name |
| `required_modules_for(name)` | Get direct declared dependencies |
| `missing_requirements_for(name)` | List unsatisfied deps before enabling |
| `enable_with_requirements(name)` | Enable module + all transitive deps |
| `set_module_state(name, enabled)` | Toggle with validation (checks deps, platform) |
| `merge_defaults()` | Config migration — legacy renames applied here |
| `set_python_extension_enabled(name, enabled)` | Toggle a Python extension's enabled state |

---

## Platform filtering

`supports_runtime_platform(supported_platforms)` checks against `runtime_platform_id()` which returns one of: `macos`, `windows`, `linux`, `ios`, `unknown`.

Empty `supported_platforms` = supported everywhere. Non-empty = must match at least one entry.

Platform-restricted modules (`tray`, `cursor`, `overlay`) are hidden from the Modules page on unsupported platforms. Attempting to enable them returns an error.

---

## Dependency enforcement

Before enabling module X:
1. `missing_requirements_for(X)` returns any required modules not currently enabled.
2. `enable_with_requirements(X)` enables the full transitive closure bottom-up.
3. The Modules page shows a requirements modal listing what will be co-enabled.

Disabling a module does not cascade-disable dependents automatically — the user must disable dependents first. The GUI shows a warning when attempting to disable a module that others depend on.

---

## Config file (`modules.toml`)

```toml
[modules]
terminal = true
lan = false
python-host = true

[python_extensions]
terminal-theme = true
my-extension = false
```

`[modules]` — map of module name → enabled bool.  
`[python_extensions]` — map of Python extension ID → enabled bool.

---

## Migrations (`merge_defaults`)

Legacy module name renames are handled in `ModulesConfig::merge_defaults()`:

```rust
// Pattern: move old key's value to new key (only if new key not already set)
if let Some(val) = self.modules.remove(LEGACY_LAN_MODULE_NAME) {
    self.modules.entry(LAN_MODULE_NAME.to_string()).or_insert(val);
}
```

Known migrations:

| Old name | New name |
|----------|----------|
| `"lan-module"` | `"lan"` |
| `"shell"` | `"terminal"` |
| `"shell-motd"` | `"terminal-motd"` |

Python extension ID migrations live in a parallel block using `LEGACY_TERMINAL_THEME_EXTENSION_IDS`:

| Old IDs | New ID |
|---------|--------|
| `"tui-style"`, `"shell-theme"`, `"flux-theme"` | `"terminal-theme"` |

---

## Modules page (GUI)

Page: `global.modules` (always visible, appears in top bar)  
Files: `Desktop/src/gui/app/modules_page/`

| Sub-file | Purpose |
|----------|---------|
| `panel.rs` | Root modules page — searchable module list |
| `row.rs` | Individual module row with toggle and metadata |
| `requirements_modal.rs` | Modal shown when enabling requires co-enabling deps |
| `permission_modal.rs` | Modal shown when a module declares required permissions |
| `mod.rs` | Module exports |

The panel iterates `MODULE_REGISTRY` filtered by `supports_runtime_platform`, renders rows with enable/disable toggles, and shows dependency and permission information inline.

---

## Adding a new module

1. Add `pub const X_MODULE_NAME: &str = "x";` to `config/modules.rs`.
2. Add a `ModuleManifest` entry to `MODULE_REGISTRY` (all fields required).
3. Create `Shared/ArcadiaCore/src/modules/x.rs` with a `commands()` fn.
4. Register in `Shared/ArcadiaCore/src/modules/mod.rs`: `pub mod x;` + dispatch arm in `module_commands()`.
5. Done — GUI and CLI pick it up automatically.

---

## Renaming a module

1. Edit the constant and `MODULE_REGISTRY` entry in `config/modules.rs`.
2. Add a migration in `ModulesConfig::merge_defaults()` following the established pattern.
3. Do not do ad-hoc renames at call sites — the migration is the single change point.

Skipping step 2 silently strands user settings (the old key remains in `modules.toml` with no effect; the new key starts as the default).
