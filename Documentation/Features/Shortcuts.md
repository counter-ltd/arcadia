# Shortcuts

Config file: `shortcuts.toml`  
Source: `Shared/ArcadiaCore/src/shortcuts/`, `config/shortcuts.rs`  
Page: `global.shortcuts`  
Platform: all (OS-global shortcuts desktop-only)

---

## Overview

Arcadia has a layered keyboard shortcut system:

1. **Static registrations** — declared by modules/panels in code with a stable ID, trigger, and action list.
2. **User overrides** — stored in `shortcuts.toml`; can disable, rebind, or reprioritize any static shortcut.
3. **Custom shortcuts** — fully user-created entries with no static counterpart.
4. **OS-global shortcuts** — shortcuts consented by the user for system-wide registration (fire even when Arcadia is not focused).

---

## Core types (`shortcuts/model.rs`)

### `ShortcutTrigger`

Defines when a shortcut fires:

| Variant | Description |
|---------|-------------|
| `Chord(KeyChordSpec)` | Single key combination (e.g. `Ctrl+K`) |
| `Sequence(Vec<KeyChordSpec>)` | Multi-step chord sequence (e.g. `Ctrl+K` then `Ctrl+N`) |

### `KeyChordSpec`

Represents a single key combination:
- Modifier flags: `ctrl`, `alt`, `shift`, `meta` (Cmd on macOS)
- Key code: logical key name string

### `ShortcutAction`

What happens when the shortcut fires:

| Variant | Description |
|---------|-------------|
| `ExecuteCommand { token, args }` | Dispatch `execute_command(token, args)` |
| `NavigateTo { page_id }` | Switch active page to `page_id` |
| `ToggleModule { name }` | Toggle a module on/off |
| `OpenSettings` | Navigate to settings hub |

---

## Config (`shortcuts.toml`)

### `ShortcutsConfig`

| Field | Type | Purpose |
|-------|------|---------|
| `overrides` | `BTreeMap<id, ShortcutOverride>` | User overrides keyed by shortcut ID |
| `system_wide_consented_ids` | `Vec<String>` | Shortcut IDs the user has approved for OS-global registration |
| `custom` | `Vec<CustomShortcut>` | Fully user-created shortcuts |

### `ShortcutOverride`

| Field | Purpose |
|-------|---------|
| `disabled` | `Some(true)` to disable the shortcut |
| `chord` | Replace the trigger chord with a new key combination |
| `sequence` | Replace a `Sequence` trigger's step list |
| `priority` | Override resolution priority (higher = checked first) |

### `CustomShortcut`

| Field | Purpose |
|-------|---------|
| `id` | User-assigned stable identifier |
| `label` | Display name for the Shortcuts page |
| `trigger` | `ShortcutTrigger` (chord or sequence) |
| `actions` | List of `ShortcutAction` to execute |

---

## Shortcut merge (`shortcuts/merge.rs`)

`merge_shortcuts(static_registrations, config)` produces the final effective shortcut list:

1. Start with static registrations.
2. Apply `config.overrides` per ID — patch `disabled`, `chord`, `sequence`, `priority`.
3. Append `config.custom` entries.
4. Sort by priority (descending), then by registration order for ties.

The merged list is what the GUI and OS shortcut dispatcher consume.

---

## Shortcut registry (`shortcuts/registry.rs`)

The registry holds the current merged shortcut set. Modules register static shortcuts at startup; the registry merges with user config and makes the result available to surfaces.

Key functions:

| Function | Purpose |
|----------|---------|
| `register(shortcuts)` | Add static shortcut registrations |
| `merged()` | Return the full merged shortcut list |
| `dispatch(chord)` | Find matching shortcuts and return their actions |

---

## OS-global shortcuts

Shortcuts with sensitive actions (e.g. `ExecuteCommand`) require explicit user consent before being registered system-wide. The consent flow:

1. User opens the Shortcuts page and enables system-wide for a shortcut.
2. The shortcut ID is added to `system_wide_consented_ids` in `shortcuts.toml`.
3. The desktop surface registers it with the OS shortcut manager.

The `os_hotkey.rs` file in `Desktop/src/gui/app/shortcuts/` handles macOS/Windows/Linux global hotkey registration.

---

## GUI panel

Page: `global.shortcuts`  
Files: `Desktop/src/gui/app/shortcuts/`

| File | Purpose |
|------|---------|
| `mod.rs` | Panel entry, state |
| `os_hotkey.rs` | OS-level global hotkey registration/deregistration |

The panel shows:
- All registered shortcuts (static + custom) with their effective triggers.
- Override controls: disable toggle, rebind chord/sequence.
- Priority adjustment.
- OS-global consent toggle (where applicable).
- Custom shortcut creation form.
- Conflict detection (two shortcuts with the same trigger and overlapping scope).
