# Extensions (Python Host)

Module name: `python-host`  
Extension directory: `~/Arcadia/Extensions/`  
Platform: all (platform filtering per extension via `supported_platforms`)

---

## Overview

The Python extension host scans `~/Arcadia/Extensions/` for `.py` files and loads each as an Arcadia extension. Extensions can register:

- **Module metadata** — name, version, description, platform requirements, permissions.
- **Commands** — callable via the `execute_command` dispatch path (e.g. `myext.action`).
- **Tray icon callbacks** — invoked when the user clicks the tray icon.
- **Reload hooks** — called when the extension is reloaded.
- **Syntax highlight providers** — used by the code editor.
- **Decoration providers** — background decoration rects for the code editor.
- **Style tokens** — extension-declared theme token overrides (see Appearance.md).
- **Overlay HUD** — extensions declaring `overlay.hud` in required permissions get the native `overlay` module enabled automatically.

---

## Extension lifecycle

### Discovery (stub registration)

On startup, `python_registry::register_discovered` scans `~/Arcadia/Extensions/` and creates a **stub** `PythonModuleInfo` for each `.py` file found. Stubs have `loaded = false`. The Extensions page shows these as "Disabled — not loaded" until the user enables them.

### Loading

When the user enables an extension, `load_one_fn` is invoked with the stub ID and on-disk path. The Python body runs, calls `register_module(name=…, …)`, and the registry merges the live entry with the stub. After loading, `loaded = true`.

Folder name and `register_module(name=…)` may differ — the registry handles the mismatch via `load_and_merge` in `arcadia-python`.

### Enable/disable persistence

`python_host::persist_extension_state(name, enabled)` toggles `python_extensions[name]` in `modules.toml`. Enabled state survives restarts.

---

## Registry data model (`PythonModuleInfo`)

| Field | Purpose |
|-------|---------|
| `name` | Extension identifier (from `register_module(name=…)`) |
| `version` | Semver string |
| `description` | Short description shown in Extensions UI |
| `enabled` | Whether the extension is currently enabled |
| `required_permissions` | Permissions declared at `register_module` — shown in first-enable UI |
| `supported_platforms` | `["macos", "windows", "linux", "ios"]` — empty = all |
| `path` | On-disk `.py` source path (set by `register_discovered`) |
| `loaded` | `true` once the Python body has run and `register_module` was called |

---

## Overlay companion auto-enable

Extensions declaring `overlay.hud` in `required_permissions` trigger `ensure_native_companions_for_loaded_extension`:

1. Check if extension declares `overlay.hud`.
2. If `overlay` module is already enabled → done.
3. Grant `overlay.hud` permission to the `overlay` module subject via `PermissionsConfig`.
4. Call `ModulesConfig::enable_with_requirements(OVERLAY_MODULE_NAME)`.
5. Save both configs.

This means enabling a HUD extension automatically enables the native overlay module without requiring manual user action.

---

## Extension navigation

`extension_nav_panel.rs` in the GUI renders navigation entries for extensions that declare nav pages. These appear in the sidebar alongside native pages and follow the same `required_module` visibility rules (always `python-host` as the gate module for extension pages).

---

## Style tokens (extension tokens)

Extensions can declare style token overrides via the token registry. The token resolution system in `modules/style_tokens.rs` provides:

| Type | Purpose |
|------|---------|
| `StyleTokenSpec` | Declares a token (kind, display, bounds, visibility) |
| `StyleTokenKind` | `Color`, `Numeric`, `Enum`, `Bool` |
| `StyleTokenNumericBounds` | Min/max/granularity for sliders |
| `StyleTokenVisibility` | Conditional visibility rules (`StyleTokenCompareOp`) |

Token kinds:
- **Color** — hex color values
- **Numeric** — floating-point with optional slider bounds and granularity
- **Enum** — fixed set of string variants
- **Bool** — toggle

`effective_token_display` resolves the current value for display; `snap_slider_value` snaps slider positions to granularity steps; `clamp_numeric_display_for_spec` clamps values within declared bounds.

Extension token settings are shown in `extension_token_settings.rs` and `appearance/extension_tokens.rs` in the GUI.

---

## Commands

| Command | Usage | Description |
|---------|-------|-------------|
| `python-host.list` | `python-host.list` | List all discovered extensions with name, version, state, and description. Also lists registered extension commands. |
| `python-host.enable` | `python-host.enable <name>` | Enable a discovered extension by name |
| `python-host.disable` | `python-host.disable <name>` | Disable an extension |
| `python-host.reload` | `python-host.reload <name>` | Trigger the registered reload hook for an extension |

---

## GUI — Extensions page

Page: `python.settings` (requires `python-host`)  
Files: `Desktop/src/gui/app/python_settings/`

Shows:
- All discovered extensions (stubs + loaded).
- Enable/disable toggles.
- Version, description, platform compatibility badge.
- Required permissions listed on first enable.
- Link to `~/Arcadia/Extensions/` for adding new extensions.

---

## Platform filtering

`supports_runtime_platform_owned(supported_platforms: &[String])` checks `supported_platforms` against the current platform ID (`macos`, `windows`, `linux`, `ios`, `unknown`). Empty list = all platforms. Filtered extensions are hidden from the Extensions page on incompatible platforms.

---

## Legacy ID migrations

`modules.toml` stores `python_extensions[<id>] = bool`. When extension IDs are renamed, `ModulesConfig::merge_defaults()` maps old IDs to new ones so user enable/disable state is preserved. The known migration pattern:

```toml
# Old: tui-style / shell-theme / flux-theme → new: terminal-theme
LEGACY_TERMINAL_THEME_EXTENSION_IDS = ["tui-style", "shell-theme", "flux-theme"]
```

All three collapse to `terminal-theme` during `merge_defaults`.
