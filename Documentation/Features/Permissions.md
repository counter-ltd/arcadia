# Permissions

Module name: `permissions`  
Config file: `permissions.toml`  
Source: `Shared/ArcadiaCore/src/config/permissions.rs`  
Page: `global.permissions`  
Platform: all

---

## Overview

The permissions module provides a capability catalog and grant management layer. Permissions gate specific operations (network access, overlay HUD, system-wide shortcuts) at both the module and extension level. Each permission is associated with a subject (a module or extension identifier) and stored in `permissions.toml`.

---

## Concepts

### Permission subject

A `PermissionSubject` identifies who holds a permission grant:

```rust
PermissionSubject::module("overlay")        // native module
PermissionSubject::extension("my-ext")     // Python extension
```

### Permission grant

A grant is a `(subject, permission_id)` pair stored in `permissions.toml`. Grants are additive — revocation removes the entry.

### Effective check

`PermissionsConfig::effective_allowed(subject, permission_id)` returns `true` if the subject has been explicitly granted `permission_id`.

---

## `PermissionsConfig` methods

| Method | Purpose |
|--------|---------|
| `effective_allowed(subject, id)` | Check if a subject has a specific permission |
| `ensure_effective_grants(subject, ids)` | Grant all listed permissions to a subject if not already granted. Used by companion auto-enable. |
| `grant(subject, id)` | Add a grant |
| `revoke(subject, id)` | Remove a grant |

---

## Commands

| Command | Usage | Description |
|---------|-------|-------------|
| `permissions.permit` | `permissions.permit <subject> <permission_id>` | Grant a permission to a subject |
| `permissions.list` | `permissions.list [subject]` | List all grants, optionally filtered by subject |

---

## Module-declared permissions

Modules declare the global permissions they require via `required_permissions` in their `ModuleManifest`. The Modules page shows these when enabling a module and prompts the user to review them via `permission_modal.rs`.

---

## Workspace permissions vs global permissions

| Layer | Storage | Scope |
|-------|---------|-------|
| Global permissions | `permissions.toml` | Module/extension capabilities (overlay, network, system-wide shortcuts) |
| Workspace permissions | `workspace.toml` | Per-directory file/exec grants for AI and modules |

These are separate systems. Global permissions gate feature capabilities; workspace permissions gate filesystem and execution access within a registered directory.

---

## GUI panel

Page: `global.permissions`  
File: `Desktop/src/gui/app/permissions_panel.rs`

Shows all registered subjects and their current grants. Provides:
- Toggle rows for each known permission per subject
- Add/revoke UI
- Module and extension subject labels
