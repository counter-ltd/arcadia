# Workspaces

Module name: `workspace`  
Config file: `workspace.toml`  
Platform: all

---

## Overview

Workspaces are registered directory entries that grant scoped file and execution permissions to modules and the AI layer. Each workspace has a stable ID, a human label, a filesystem path, and a list of explicitly granted permission IDs.

Workspaces serve two distinct roles:

1. **AI context** — the chat panel selects a workspace to scope AI file/exec tools to a project directory.
2. **Per-module grants** — enabled modules with `workspace_permissions` declarations advertise per-directory toggles in the Workspaces settings page.

---

## Data model

### `WorkspaceEntry`

```toml
[[workspaces]]
id = "ws_1715000000000"
label = "My Project"
path = "/Users/me/projects/myapp"
granted_permissions = ["workspace.read", "workspace.write"]
```

| Field | Purpose |
|-------|---------|
| `id` | Stable identifier — format `ws_<unix_ms_timestamp>`. Never reused. |
| `label` | Human-readable display name |
| `path` | Absolute filesystem path to the workspace root |
| `granted_permissions` | List of permission IDs granted for this directory |

### Permission IDs

Standard permissions gated by the workspace module and AI sandbox:

| ID | Meaning |
|----|---------|
| `workspace.read` | AI and modules may read files under this path |
| `workspace.write` | AI and modules may write files under this path |
| `workspace.execute` | AI may run commands with CWD set to this path |

Modules may declare additional custom workspace permission IDs via `WorkspacePermissionDef` in their `ModuleManifest`. The Workspaces page renders toggle rows for every permission declared by any enabled module.

---

## Commands

All commands live under the `workspace` module token:

| Command | Usage | Description |
|---------|-------|-------------|
| `workspace.list` | `workspace.list` | Print all registered workspaces with IDs, labels, paths, and granted permissions |
| `workspace.add` | `workspace.add <path> <label>` | Register a new workspace. Generates a `ws_<timestamp>` ID. Default grant: `workspace.read`. Rejects duplicate paths. |
| `workspace.remove` | `workspace.remove <id>` | Remove a workspace by ID |
| `workspace.grant` | `workspace.grant <id> <permission_id>` | Grant a permission to a workspace. No-op if already granted. |
| `workspace.revoke` | `workspace.revoke <id> <permission_id>` | Revoke a permission from a workspace |
| `workspace.check` | `workspace.check <path> <permission_id>` | Returns `"true"` / `"false"` — whether any workspace covers `path` and has the permission |

---

## GUI panel

Page: `global.workspaces` (requires `workspace` module)  
Files: `Desktop/src/gui/app/workspace_panel.rs`, `workspace_row.rs`, `workspace_create_modal.rs`

The Workspaces panel shows:
- List of registered workspaces with search.
- Per-workspace permission toggles for every `WorkspacePermissionDef` declared by any currently-enabled module.
- A create-workspace modal (label + path fields).
- Remove button per row.

---

## Permission check logic

`any_workspace_grants(path, permission_id)` in `config/workspace.rs`:

```rust
list_workspaces()
    .iter()
    .any(|w| path.starts_with(&w.path) && w.has_permission(permission_id))
```

Note: this uses raw string `starts_with` for the general config lookup. The **AI sandbox** (`ai_types.rs → is_path_in_scope`) uses canonicalization instead — path traversal safety is enforced at the sandbox boundary, not in the config lookup.

---

## AI workspace context

`AiWorkspaceContext` is derived from a `WorkspaceEntry` at request assembly time in the chat panel:

```rust
AiWorkspaceContext {
    workspace_id,
    workspace_path,
    granted_permissions,  // copied from WorkspaceEntry
}
```

Helper methods on `AiWorkspaceContext`:
- `can_read()` — `granted_permissions` contains `"workspace.read"`
- `can_write()` — contains `"workspace.write"`
- `can_execute()` — contains `"workspace.execute"`
- `is_path_in_scope(path)` — canonicalize-based scope check (see AI.md for detail)

---

## Module workspace permission declarations

Modules advertise per-workspace permission toggles via `workspace_permissions` in their `ModuleManifest`:

```rust
pub const MY_WORKSPACE_PERMS: &[WorkspacePermissionDef] = &[
    WorkspacePermissionDef {
        id: "mymodule.access",
        title: "My Module Access",
        description: "Allows My Module to read files in this workspace.",
        default_granted: false,
    },
];
```

The Workspaces GUI iterates all enabled modules and renders a toggle row for each declared permission per workspace.

---

## ID generation

`new_workspace_id()` uses `SystemTime::now().duration_since(UNIX_EPOCH).as_millis()` formatted as `ws_<ms>`. Collision probability at human interaction speed is negligible.
