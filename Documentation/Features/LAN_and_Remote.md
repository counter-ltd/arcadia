# LAN & Remote Sessions

Modules: `net`, `lan`, `remote-session`, `surface`  
Platform: all (LAN service runs on desktop; iOS connects as thin client)

---

## Overview

Arcadia's networking stack has three layers:

1. **`net`** — shared networking foundation; must be enabled before `lan` or `remote-session`.
2. **`lan`** — UDP-based peer discovery, pairing, and command routing over the local network.
3. **`remote-session`** — permission gate that allows `execute_command` calls to be forwarded to a peer via `ExecutionContext.net_as`. Requires `net` + `lan`.
4. **`surface`** — UI snapshot/patch protocol used for thin-client mirroring. Independent of the three above; used by surfaces that need to replicate host state.

---

## LAN discovery

### Service

`lan/discovery.rs` runs a UDP listener on port `42424` (constant `DISCOVERY_PORT`). The service is started/stopped via `start_service()` / `stop_service()` and exposes an atomic `SERVICE_RUNNING` flag polled by the Services page.

Local hostname resolution order:
1. `HOSTNAME_OVERRIDE` (set via `set_hostname_override()` at startup — headless uses this for a custom name)
2. `$HOSTNAME` env var
3. `$COMPUTERNAME` env var (Windows)
4. `hostname` subprocess output
5. Fallback: `"unknown-host"`

### Discovery protocol

All messages are plain UTF-8 UDP datagrams. Constants in `lan/protocol.rs`:

| Constant | Value/purpose |
|----------|---------------|
| `DISCOVERY_PORT` | `42424` — all LAN traffic |
| `DISCOVERY_REQUEST` | Probe sent during `lan.scan` |
| `DISCOVERY_RESPONSE_PREFIX` | Prefix of reply from a responding peer |
| `NODE_CONNECT_PREFIX` | Sent to request pairing |
| `NODE_ACCEPT_PREFIX` | Positive pairing response |
| `NODE_REJECT_PREFIX` | Negative pairing response |
| `NODE_EXEC_PREFIX` | Routed command payload |
| `NODE_EXEC_RESULT_PREFIX` | Response to a routed command |
| `DEFAULT_REMOTE_TIMEOUT_MS` | Default timeout for remote command roundtrips |
| `SCAN_WAIT_MS` | Time to collect probe responses |

---

## Commands

### `lan.scan`

Sends a discovery broadcast over a target range (default: local subnet broadcast, overridable with `--range <CIDR>`). Collects `DISCOVERY_RESPONSE_PREFIX` replies within `SCAN_WAIT_MS`. Updates in-memory peer table (`peers.rs`).

Supports `--self` flag to include the local host in the response (useful for debugging).

CIDR target parsing: splits on `/`, validates prefix length (0–32), broadcasts to the host-bits range via IPv4 arithmetic.

### `lan.status`

Returns LAN service status: running flag, port number, and resolved local hostname.

### `lan.node`

Manages peer relationships. Subcommands:

| Subcommand | Purpose |
|------------|---------|
| `pair <ip>` | Initiate pairing — sends `NODE_CONNECT_PREFIX` to peer |
| `connect <ip>` | Mark peer as connected (used after peer accepts) |
| `accept <ip>` | Accept an incoming pair request |
| `reject <ip>` | Reject an incoming pair request |
| `alias <ip> <name>` | Assign a human-readable alias to a peer IP |
| `save` | Persist current node config to `lan_nodes.toml` |
| `auto <ip> on/off` | Toggle auto-accept for a peer |
| `status` | Print all known peers with their current status |

### `lan.session_targets`

Returns a JSON array of connected, approved peers: `[{"ip": "…", "hostname": "…"}]`. Used by the remote-route picker in the GUI to populate the node selector.

---

## Peer state

`lan/peers.rs` holds an in-memory `NodeState` (mutex-guarded):

```
peers: HashMap<ip, PeerRecord>
  ip: String
  hostname: String
  status: PeerStatus (Discovered | Connected | Pending | Rejected)
```

`PeerStatus::Connected` + approved in `lan_nodes.toml` = eligible for remote session routing.

---

## Remote command execution

`execute_remote_command(target, token, args, timeout_ms)` in `lan/mod.rs`:

1. Load node config, resolve alias if target is a name.
2. Resolve target to `SocketAddrV4`.
3. Check peer exists in state and is `Connected` + approved.
4. Open a UDP socket, set read timeout.
5. Send `NODE_EXEC_PREFIX\t{token}\t{args…}`.
6. Await `NODE_EXEC_RESULT_PREFIX\t{result}`.

The `token` is the command name (e.g. `shell.execute`). Args are tab-separated after the token.

---

## Remote session routing

When `execute_command` is called with `ExecutionContext { net_as: Some("lan:192.168.1.10"), … }`:

1. Core checks that `remote-session` is enabled locally.
2. Resolves the `lan:` prefix to an IP.
3. Forwards via `execute_remote_command`.
4. The remote peer checks its own module rules for the command token.

`net_as` format: `lan:<ipv4-address>` or `lan:<alias>`.

`net_timeout_ms` in `ExecutionContext` overrides `DEFAULT_REMOTE_TIMEOUT_MS` for that call.

`ARCADIA_NET_AS` environment variable overrides `thin-client.toml`'s `preferred_remote_route` on startup.

---

## Surface snapshot & patch

The `surface` module provides a generic UI mirroring protocol independent of LAN specifics.

### `surface.snapshot`

Returns a JSON payload:

```json
{
  "modules": {"terminal": true, "lan": false, …},
  "revision": 42,
  "extra": {
    "schema_version": 1,
    "navigation_registry": {…}
  }
}
```

`extra.navigation_registry` is a serialized `NavigationRegistryOwned` — the host's full page/group/settings structure. Thin clients with `navigation_from_host_only = true` use this exclusively for navigation; they never fall back to their own compiled-in tables.

### `surface.patch`

Accepts a JSON op array. Current ops:

| Op | Fields | Effect |
|----|--------|--------|
| `modules_set` | `name`, `enabled`, optional `client_id` | Toggle a module on the host |

`client_id` is attribution only — not authentication. Do not build access control on it.

### `surface.revision`

Returns `{"revision": N}`. Revision is a `u64` counter incremented on every `ModulesConfig::save`. Thin-client GUIs poll this on a timer; when revision diverges from the last snapshot, a "stale" banner appears with a Reload button.

---

## Remote mirror drain

`modules/remote_mirror.rs` holds a transcript queue for surface mirroring. The desktop GUI polls `remote_mirror::drain_*` on a timer when acting as a thin client. On state change, panels call back into `modules::reload_*` to refresh the GUI without a full reconnect.

---

## Thin client config (`thin-client.toml`)

| Field | Default | Purpose |
|-------|---------|---------|
| `preferred_remote_route` | `null` | Default `net_as` target — `lan:<ip>` |
| `surface_client_id` | auto-generated UUID | Identifies this surface in patch ops |
| `navigation_from_host_only` | `false` | When true + LAN route active: use only host `surface.snapshot` navigation |

`surface_client_id` is lazy-generated on first use and persisted. Subsequent loads reuse the stored UUID.

---

## Node config (`lan_nodes.toml`)

Not a `ConfigFile` trait implementor — written directly by `lan/config.rs`. Stores:
- Approved peer identifiers (IP or hostname).
- Aliases (human name → IP mapping).
- Auto-accept flags per peer.

---

## Dependency chain

```
remote-session → lan → net
```

Enabling `remote-session` transitively enables `lan` and `net`.

---

## Known constraints

- LAN forwarding requires `remote-session`, `lan`, and `net` enabled **locally**. The remote peer enforces its own module rules.
- Multiple concurrent GUIs on the same host = last-write-wins on `modules.toml` (no merge semantics).
- `surface.patch` `client_id` is attribution only — not auth. Do not build access control on it.
- `navigation_from_host_only = true`: if LAN route is active but no snapshot yet available, GUI shows no navigation until the snapshot supplies `navigation_registry`.
