# Arcadia — “Ultimate” thin-client / remote-surface gaps

This document tracks intentional limitations and follow-up work for the LAN-routed **`surface.snapshot`** / **`surface.patch`** model, multi-peer GUIs, and related architecture. It complements **`README.md`** (current behavior) and **`CLAUDE.md`** / **`AGENTS.md`** (contributor rules).

---

## 1. `surface.revision` semantics

The revision counter advances only after a successful **`surface.patch`** batch on the host. Other writers can change **`modules.toml`** without bumping revision — for example:

- **`module …`** via CLI
- **`set_module_enabled`** / related paths through FFI

Clients that infer freshness **only** from **`surface.revision`** can miss updates until another **`surface.patch`** occurs or they reload from disk/snapshot for other reasons.

**Directions:** bump revision from every **`ModulesConfig::save`** (or equivalent), or stop promising revision as a global host-generation marker until coverage is complete.

---

## 2. Stale / concurrent UI

Desktop keeps **`last_surface_revision`** but does not use it for:

- “Host changed under you” detection  
- Auto-**`reload_modules`** or snapshot refresh  
- User-visible warnings

There is no periodic poll, focus hook, or push channel tied to revision.

**Directions:** compare **`revision`** on timer/focus/after each routed command; optional banner + reload.

---

## 3. Multi-writer model

The host exposes a **single** **`modules.toml`**. Multiple GUIs (or CLI + GUI) produce **last write wins** with no:

- Merge semantics  
- Locks  
- Optimistic concurrency (e.g. generation tokens on save)  
- CRDT / operational transforms

**Directions:** document as permanent constraint, or add explicit versioning / conflict errors on save.

---

## 4. Transport

Command routing still centers on **discrete remote executions** (e.g. LAN **`NODE_EXEC`** style request/response), not a **long-lived session** with:

- Ordering guarantees across unrelated commands  
- Low-latency subscriptions for snapshot deltas  
- Back-pressure

**Directions:** optional WebSocket/TCP sidecar for “thin shell” workflows while keeping **`execute_command`** as the logical API.

---

## 5. Identity beyond `client_id`

**`surface.patch`** may carry **`client_id`** (persisted per GUI in **`thin-client.toml`**). Today it is mainly for **attribution**, not:

- Authorization (“who may patch”)  
- Rate limits  
- Per-client sandbox or filtered views

**Directions:** host-side policy module or capability tokens if multi-tenant control matters.

---

## 6. `SurfaceSnapshot.extra` and patch vocabulary

**`extra.navigation_registry`** is populated; broader **`extra`** buckets (editors, arbitrary UI state) and corresponding **`SurfacePatch`** variants are **not** fully specified or wired through Desktop/iOS.

**Directions:** define schema/version fields inside **`extra`**, extend **`SurfacePatch`** incrementally, keep surfaces consuming **`surface.*`** instead of ad hoc modules.

---

## 7. Renderer-only client

Fallback navigation still lives in **compiled core / bundled JSON** on each surface. A pure **“no local nav table”** client that trusts **only** **`surface.snapshot`** for structure is not fully enforced or documented as a supported SKU.

**Directions:** optional build profile or runtime flag that refuses static nav when **`remote_route`** is mandatory.

---

## 8. Testing & CI

There is limited automated coverage for:

- **`parse_surface_snapshot`** / **`NavigationRegistryOwned`** round-trips  
- Thin-client preference persistence  
- LAN routing integration

iOS **`ArcadiaCore.xcframework`** rebuild after FFI changes is **manual** unless CI encodes **`Shared/Scripts/Builds/build-ios-framework.sh`**.

**Directions:** add targeted **`arcadia-core`** tests + workflow step that fails when Generated bindings / xcframework drift from **`ffi.rs`**.

---

## 9. Security posture

Trust model today assumes **LAN pairing + locally approved peers**. There is no documented story for:

- Encryption on the wire  
- Authenticated remote principals beyond “approved node”  
- Scoped capability for dangerous tokens (**`shell.execute`**, etc.)

**Directions:** threat model doc + optional TLS or pairing secrets if Arcadia leaves trusted LANs.

---

## 10. Cross-surface parity

Behavior differs across surfaces for historical reasons:

- Desktop **PTY / TUI** paths vs **generic** **`shell.execute`** when routed  
- iOS host constraints on shell / PTY  
- Not every panel is strictly **`execute_command`**-only

**Directions:** converge on one abstraction per capability class (shell, modules, nav) with explicit “unavailable on this surface” messages from core.

---

## Summary

The shipped model is **good enough for feature development** on a **trusted LAN** with **one logical host** and **thin clients** that periodically **`surface.snapshot`**. Closing gaps above moves toward **stronger freshness guarantees**, **safer multi-writer behavior**, and **production-grade sync/security**.
