# Roadmap and Known Gaps

`Documentation/GAPS.md` tracks all deliberate limitations. Summary with priority ranking:

## P0 — Fix before trusting in production

| Gap | Problem | Direction |
|----|---------|-----------|
| **Revision coverage** | `surface.revision` only advances on `surface.patch`. CLI writes and FFI writes bypass it — clients can miss updates. | Bump revision from every `ModulesConfig::save`. |
| **Testing discipline** | No automated tests for snapshot round-trips, thin-client prefs, or LAN routing. | Add targeted `arcadia-core` unit + integration tests. |
| **FFI drift detection** | No CI check that `Generated/` matches `ffi.rs`. | Workflow step: rebuild and fail if diff. |

## P1 — Needed for real multi-user / multi-surface use

| Gap | Problem | Direction |
|----|---------|-----------|
| **Stale UI detection** | Desktop has `last_surface_revision` but never compares it — no "host changed under you" warning. | Compare revision on timer/focus/after routed command; optional banner + reload. |
| **Multi-writer** | Multiple GUIs on same host = last write wins, no merge, no locks. | Document as permanent constraint OR add optimistic concurrency (generation tokens on save). |
| **Transport** | Command routing is request/response UDP. No long-lived session, no ordering guarantees, no subscription for deltas. | Optional WebSocket/TCP sidecar for continuous thin-shell workflows. |

## P2 — Required before leaving trusted LAN

| Gap | Problem | Direction |
|----|---------|-----------|
| **Security posture** | No wire encryption, no auth beyond "approved node," no scoped capabilities. `shell.execute` routable to anyone approved. | Threat model doc + TLS or pairing secrets + capability tokens. |
| **Identity** | `client_id` is attribution only — no authz, no rate limits, no per-client filtering. | Host-side policy module or capability tokens if multi-tenant. |

## P3 — Polish and convergence

| Gap | Problem | Direction |
|----|---------|-----------|
| **iOS OpenFrame migration** | SwiftUI shell while Desktop uses OpenFrame; embedding/a11y/build story not done. | **§11** in `Documentation/GAPS.md` — spike, iOS shell crate, registry-driven OF UI, then cut SwiftUI. |
| **Surface parity** | Desktop has PTY/TUI paths; iOS is shell.execute only; not all panels are execute-only. | Converge per capability class with explicit "unavailable on this surface" from core. |
| **Renderer-only client** | Surfaces still bundle compiled nav — no enforced "remote-only" profile. | Optional build flag that refuses static nav when `remote_route` is mandatory. |
| **`extra` schema** | `extra.navigation_registry` is wired; broader extra buckets and corresponding `SurfacePatch` variants are undefined. | Define schema + version fields inside `extra`; extend `SurfacePatch` incrementally. |

---

## Security posture

Current trust model: **LAN pairing + locally approved peers.** Assume trusted network.

What this means in practice:
- Any approved LAN peer can execute any command the host has enabled, including `shell.execute`.
- `surface.patch` is unauthenticated beyond `client_id` (which is just a UUID, not a secret).
- There is no encryption on the wire.

**Do not expose Arcadia to untrusted networks without addressing P2 gaps above.** This is a home-network / trusted-LAN tool today. Production-grade multi-tenant use requires TLS, capability tokens, and a real threat model document first.

---

## CI

`.github/workflows/` — `stable-build-matrix.yml` builds Desktop targets and iOS simulator configs on selected branches. See individual workflow files for triggers and matrix.

Gaps in CI coverage: FFI drift detection, core integration tests. See [CONTRIBUTING.md](CONTRIBUTING.md).
