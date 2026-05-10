# Module and Navigation Reference

## Module reference

| Module | Name constant | Requires | Description |
|--------|--------------|----------|-------------|
| `net` | `NET_MODULE_NAME` | — | Networking foundation; bootstraps LAN service |
| `lan` | `LAN_MODULE_NAME` | `net` | LAN discovery via UDP; peer management; pairing |
| `surface` | `SURFACE_MODULE_NAME` | — | `surface.snapshot` and `surface.patch` host mirror channel |
| `remote-session` | `REMOTE_SESSION_MODULE_NAME` | `net`, `lan` | Routing gate for LAN command forwarding; no standalone verbs |
| `shell` | `SHELL_MODULE_NAME` | — | `shell.execute` (routable), `shell.internal` (REPL), PTY/TUI on Desktop |
| `shell-motd` | `SHELL_MOTD_MODULE_NAME` | `shell` | Fastfetch-style banner on shell open |

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

| Page ID | Title | Group | Required Module | Glyph | SF Symbol |
|---------|-------|-------|-----------------|-------|-----------|
| `utility.shell` | Terminal | `utilities` | `terminal` | `terminal` | `terminal` |
| `utility.services` | Services | `utilities` | _service-driven_ | `services` | `antenna.radiowaves.left.and.right` |
| `global.dashboard` | Dashboard | (sidebar global) | — | `home` | `house` |
| `global.logs` | Logs | (top bar) | — | `logs` | `doc.text.magnifyingglass` |
| `global.settings` | Settings | (sidebar global) | — | `settings` | `gearshape` |
| `global.modules` | Modules | (top bar) | — | `modules` | `switch.2` |
| `network.nodes` | Nodes | `network` | `lan` | `nodes` | `wifi` |

### Services (`SERVICE_DEFINITIONS`)

| Service ID | Target Page | Required Module | Controls |
|------------|-------------|-----------------|----------|
| `lan.discovery` | `utility.services` | `lan` | start, stop, status_detail |
