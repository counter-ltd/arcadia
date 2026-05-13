# Arcadia — Remote surface, thin client, and iOS constraints

Companion to **`README.md`** and contributor guides. Replaces the older open-ended gap list with explicit **product constraints** and **remaining roadmap**.

---

## Trust model and transport

- **Security posture:** Designed for **trusted LANs** with **locally approved peers**. Wire encryption, authenticated remote principals beyond peer approval, and scoped capability tokens for dangerous commands (`shell.execute`, etc.) are **not shipped** in-tree today.
- **Transport:** Command routing uses **discrete `execute_command` executions** (LAN `NODE_EXEC`-style request/response). There is **no** built-in long-lived ordered session, snapshot delta subscription channel, or unified back-pressure layer. A future optional WebSocket/TCP sidecar could sit beside this model while keeping **`execute_command`** as the logical API.

---

## Multi-writer host state

- **Permanent constraint:** `modules.toml` on the host is **single-file last-write-wins**. No merge semantics, locks, optimistic concurrency fields, or CRDTs are implemented.
- Multiple GUIs or CLI + GUI against one host remain **best-effort**; operators should avoid conflicting simultaneous edits.

---

## Identity (`surface.patch.client_id`)

- **`client_id`** is **attribution only** for patches — not authorization, rate limiting, or per-client sandboxes.
- Multi-tenant host policy belongs in a future **policy/capability** layer if needed.

---

## `surface.snapshot.extra` (schema version **1**)

- Documented in `Shared/ArcadiaCore/src/modules/surface.rs`: root keys **`schema_version`**, **`navigation_registry`**.
- Extend **`extra`** and **`SurfacePatch`** incrementally for new mirrored UI state; bump **`schema_version`** when semantics change.

---

## Renderer-only navigation SKU

- **`thin-client.toml`** flag **`navigation_from_host_only`** (default `false`): when `true` **and** a LAN route is active, the GUI uses **only** host `surface.snapshot` navigation — **no** fallback to compiled-in navigation tables until the snapshot supplies `navigation_registry`.

---

## Thin-client freshness

- **`surface.revision`** advances on every **`ModulesConfig::save`** (not only `surface.patch`).
- GUI polls **`surface.revision`** on a timer when a remote route is set; **stale banner + Reload** when the counter diverges from the last loaded snapshot.

---

## Cross-surface parity messages

- Stable strings live in **`arcadia_core::capabilities`** (e.g. local iOS `shell.execute` denial).
- **`shell.internal`** without a registered executor uses a shared runtime-unavailable message.

---

## iOS OpenFrame surface

**Shipped:** Rust/OpenFrame UI via **`libarcadia_ios.a`**, thin Swift Metal shell.

| Topic | Status |
|-------|--------|
| **Shell** | **`utility.shell`** uses **`shell_ios`** — `shell.execute` / LAN routing only (no PTY/TUI). |
| **Accessibility** | OpenFrame does not yet expose full VoiceOver/Dynamic Type parity with SwiftUI. Minimum bar remains **device-driven** testing + future framework hooks for labels/focus. |
| **Visual parity** | Desktop-first Rust theme tokens; iOS-native materials/haptics are **theme-layer** decisions, not per-view hex. |
| **OpenFrame iOS maturity** | Multitasking, keyboard, safe areas, gestures — fix upstream in **OpenFrame** when framework bugs surface. |

---

## Icons on iOS

- SVG via `icon_path()` — **no SF Symbols** requirement (Rust-rendered UI).

---

## Automated testing

- **`arcadia-core`**: `surface.revision` parse tests, snapshot / navigation registry round-trip, `thin-client.toml` serde flag round-trip.
- **LAN routing**: full end-to-end **`NODE_EXEC`** integration remains optional (would need network mocks or harness); exercise manually on trusted LAN peers.
