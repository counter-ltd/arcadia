# Module and Navigation Reference

## Module reference

| Module | Name constant | Requires | Description |
|--------|--------------|----------|-------------|
| `animation` | `ANIMATION_MODULE_NAME` | — | Shared tween engine; one 16 ms driver loop services all running animations |
| `net` | `NET_MODULE_NAME` | — | Networking foundation; bootstraps LAN service |
| `lan` | `LAN_MODULE_NAME` | `net` | LAN discovery via UDP; peer management; pairing |
| `surface` | `SURFACE_MODULE_NAME` | — | `surface.snapshot` and `surface.patch` host mirror channel |
| `remote-session` | `REMOTE_SESSION_MODULE_NAME` | `net`, `lan` | Routing gate for LAN command forwarding; no standalone verbs |
| `terminal` | `TERMINAL_MODULE_NAME` | — | `shell.execute` (routable), `shell.internal` (REPL), PTY/TUI on Desktop |
| `terminal-motd` | `TERMINAL_MOTD_MODULE_NAME` | `terminal` | Fastfetch-style banner on terminal open |
| `late` | `LATE_MODULE_NAME` | — | Native late.sh client — chat rooms, music stream, reactions, bonsai |
| `python-host` | `PYTHON_HOST_MODULE_NAME` | — | Python extension loader; scans `~/Arcadia/Extensions/` for `.py` files |
| `permissions` | `PERMISSIONS_MODULE_NAME` | — | Permission catalog, grants, and headless `permit`/`list` commands |
| `tray` | `TRAY_MODULE_NAME` | — | Menu-bar (macOS) and system-tray (Windows/Linux) with dynamic images and menus |
| `cursor` | `CURSOR_MODULE_NAME` | — | OS-global cursor position and primary display size (desktop only) |
| `overlay` | `OVERLAY_MODULE_NAME` | — | Always-on-top transparent HUD window with pointer pass-through (desktop only) |
| `workspace` | `WORKSPACE_MODULE_NAME` | — | Directory registry with scoped file and execution permissions |

### Workspace commands (`modules/workspace.rs`)

| Command | Description |
|---------|-------------|
| `workspace.list` | List all registered workspaces |
| `workspace.add <path> <label>` | Register a directory as a workspace (grants `workspace.read` by default) |
| `workspace.remove <id>` | Remove a workspace by id |
| `workspace.grant <id> <permission_id>` | Grant a permission in a specific workspace |
| `workspace.revoke <id> <permission_id>` | Revoke a permission in a specific workspace |
| `workspace.check <path> <permission_id>` | Check if any workspace covering `path` grants the given permission |

### Workspace permissions (contributed to `ModuleManifest.workspace_permissions`)

| Permission ID | Default | Description |
|--------------|---------|-------------|
| `workspace.read` | granted | Read files within the workspace |
| `workspace.write` | not granted | Create, modify, and delete files |
| `workspace.execute` | not granted | Run commands scoped to the workspace |

### LAN sub-system (`modules/lan/`)

| Component | File | Purpose |
|-----------|------|---------|
| Service entry | `mod.rs` | `start_service` / `stop_service`, command registry |
| Discovery | `discovery.rs` | Peer scan, node state tracking |
| Handlers | `handlers.rs` | `lan.scan`, `lan.node`, `lan.session_targets`, pairing approval |
| Config | `config.rs` | Approved peers persistence |
| Peers | `peers.rs` | Peer struct and list management |
| Protocol | `protocol.rs` | UDP `NODE_EXEC` and related definitions |

---

## Navigation reference

Pages live in `PAGE_DEFINITIONS` (`navigation.rs`). `service-driven` in the table below means
the page is visible iff at least one entry in `SERVICE_DEFINITIONS` targeting it has its
required module enabled — see `services.rs` for the registry. Never hardcode per-page logic
in surface match arms.

| Page ID | Title | Placement | Required Module | Glyph |
|---------|-------|-----------|-----------------|-------|
| `utility.shell` | Terminal | `utilities` group | `terminal` | `terminal` |
| `utility.services` | Services | `utilities` group | _service-driven_ | `services` |
| `global.dashboard` | Dashboard | sidebar global | — | `home` |
| `global.settings` | Settings | sidebar global + settings hub root | — | `settings` |
| `global.logs` | Logs | app-title context menu | — | `logs` |
| `global.modules` | Modules | top bar | — | `modules` |
| `global.appearance` | Appearance | settings hub | — | `appearance` |
| `global.permissions` | Permissions | settings hub | — | `permissions` |
| `global.shortcuts` | Shortcuts | settings hub | — | `shortcuts` |
| `global.workspaces` | Workspaces | settings hub | `workspace` | `folder` |
| `network.nodes` | Nodes | `network` group | `lan` | `nodes` |
| `late.now_playing` | Late.sh | `social` group | `late` | `music` |
| `late.experimental` | Experimental | `social` group | `late` | `flask` |
| `late.settings` | Late.sh | settings hub | `late` | `music` |
| `python.settings` | Extensions | top bar | `python-host` | `coffee` |

### Navigation groups (`GROUP_DEFINITIONS`)

| Group ID | Label | Pages |
|----------|-------|-------|
| `utilities` | Utilities | `utility.shell`, `utility.services` |
| `network` | Network | `network.nodes` |
| `social` | Social | `late.now_playing`, `late.experimental` |

### Special page ID lists

| Constant | Pages | Rendered as |
|----------|-------|-------------|
| `GLOBAL_PAGE_IDS` | `global.dashboard`, `global.settings` | Sidebar global section |
| `TOP_BAR_PAGE_IDS` | `python.settings`, `global.modules` | Compact top-bar controls |
| `SETTINGS_HUB_ROOT_PAGE_ID` | `global.settings` | Settings hub header row |
| `SETTINGS_HUB_PAGE_IDS` | `global.permissions`, `global.shortcuts`, `global.appearance`, `global.workspaces`, `late.settings` | Nested rows under settings hub |

### Services (`SERVICE_DEFINITIONS`)

| Service ID | Target Page | Required Module | Controls |
|------------|-------------|-----------------|----------|
| `lan.discovery` | `utility.services` | `lan` | start, stop, status_detail |
