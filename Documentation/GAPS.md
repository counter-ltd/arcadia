# Arcadia — “Ultimate” thin-client / remote-surface gaps

This document tracks intentional limitations and follow-up work for the LAN-routed **`surface.snapshot`** / **`surface.patch`** model, multi-peer GUIs, related architecture, and the **iOS → OpenFrame** UI migration. It complements **`README.md`** (current behavior) and **`CLAUDE.md`** / **`AGENTS.md`** (contributor rules).

---

## 1. `surface.revision` semantics

The revision counter advances only after a successful **`surface.patch`** batch on the host. Other writers can change **`modules.toml`** without bumping revision — for example:

- **`module …`** via CLI
- Direct **`ModulesConfig::save`** from any in-process call path

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

The iOS app build runs cargo automatically from the Xcode project's "Build Rust (cargo)" phase; CI exercises it via `xcodebuild` in `stable-build-matrix.yml`. There is no automated check that `Mobile/iOS/ArcadiaApp/ArcadiaBridge.h` stays in sync with the `extern "C"` exports in `Desktop/src/ios_lib.rs`.

**Directions:** add targeted **`arcadia-core`** tests and a workflow step that fails when the iOS C ABI surface drifts from `ArcadiaBridge.h`. Add **`cargo check -p arcadia --target aarch64-apple-ios --features ios-gui`** to a fast CI lane.

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

## 11. iOS OpenFrame surface

**Status:** The OpenFrame-on-iOS surface is wired up and shipping in the repo. iOS now links **`libarcadia_ios.a`** (built from the `arcadia` package with `--features ios-gui`), and the Swift host is a thin UIKit shell (~50 lines: `ArcadiaApp.swift` + `MetalHostView.swift` + 2-function `ArcadiaBridge.h`). All UI and business logic live in Rust — there is no SwiftUI surface left to migrate. The previous **`ArcadiaCore.xcframework`** + UniFFI + `Generated/` flow has been removed.

### 11.1 Remaining gaps

| Gap | Description | Directions |
|-----|-------------|------------|
| **OpenFrame iOS maturity** | Platform code exists under **`Libraries/OpenFrame/src/platform/ios/`** but is still being exercised at Arcadia’s scale (scenes, multitasking, safe areas, software keyboard, focus, gesture recognizers). | Continue device testing; fix issues **in OpenFrame** when they are framework bugs. |
| **Accessibility** | SwiftUI would have provided **VoiceOver, Dynamic Type, system materials** with little effort. OpenFrame on iOS must **explicitly** address **a11y and contrast**. | Define and meet a **minimum a11y bar** (labels, focus, type scaling). |
| **Visual parity vs iOS conventions** | The iOS surface renders the same Rust theme tokens as desktop. Where iOS conventions differ (haptics, native-feeling sheets, glass materials), the theme layer must decide between desktop-faithful and iOS-feeling rendering. | Centralize the decision in **Rust theme** (mirror Desktop **`gui/theme/`**), not inline colors in views. |
| **Shell on iOS** | `shell.execute` only — no PTY/TUI. The desktop `gui/tui/` paths depend on `portable-pty`/`vt100` which are not part of the `ios-gui` feature. | Either ship a constrained terminal panel on iOS that uses `shell.execute` round-trips, or keep the shell route LAN-routed by default. |
| **Icons** | SVG icons are loaded through `icon_path()` for both surfaces. | Continue using SVG; no SF Symbols path is needed since iOS renders the Rust UI directly. |

### 11.2 What changed from the previous plan

The earlier multi-phase migration plan (replace SwiftUI with OpenFrame in stages, retire `ArcadiaCore.xcframework`) is complete. The remaining work is incremental polish inside `Desktop/src/gui/app/entry_ios.rs`, OpenFrame's iOS platform layer, and the panels under `Desktop/src/gui/app/` as iOS-specific issues surface.

### 11.3 Interaction with other gaps

- **§10** — The OF iOS surface must use **`execute_command`** for capabilities and surface **"unavailable on this surface"** from core for PTY/TUI-class features.
- **§6** — Any new mirrored fields stay under **`SurfaceSnapshot.extra`** / **`SurfacePatch`**.
- **§8** — Extend CI with `cargo check -p arcadia --target aarch64-apple-ios --features ios-gui` plus an iOS C ABI drift check.

---

## Summary

The shipped model is **good enough for feature development** on a **trusted LAN** with **one logical host** and **thin clients** that periodically **`surface.snapshot`**. Closing gaps **1–10** moves toward **stronger freshness guarantees**, **safer multi-writer behavior**, and **production-grade sync/security**.

**§11** now tracks **iOS surface polish** — accessibility, native-feel decisions, and PTY/TUI parity — rather than a migration; the OpenFrame-on-iOS surface itself is in place.
