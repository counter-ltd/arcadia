# Arcadia — “Ultimate” thin-client / remote-surface gaps

This document tracks intentional limitations and follow-up work for the LAN-routed **`surface.snapshot`** / **`surface.patch`** model, multi-peer GUIs, related architecture, and the **iOS → OpenFrame** UI migration. It complements **`README.md`** (current behavior) and **`CLAUDE.md`** / **`AGENTS.md`** (contributor rules).

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

**Directions:** add targeted **`arcadia-core`** tests + workflow step that fails when Generated bindings / xcframework drift from **`ffi.rs`**. For the OpenFrame iOS shell, add **`cargo check --target aarch64-apple-ios`** (or the chosen package) in CI once the crate exists.

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

## 11. iOS OpenFrame migration

**Intent:** Replace the SwiftUI shell under **`Mobile/iOS/ArcadiaApp/`** with an **OpenFrame**-based shell (same UI stack family as Desktop’s GPUI fork in **`Libraries/OpenFrame/`**), while keeping **all business logic in `arcadia-core`** and **registry-driven** navigation (no new per-module booleans, no hardcoded page visibility — see **`AGENTS.md`**).

OpenFrame documents an **iOS platform backend** (UIKit, Metal, GCD, pasteboard, keyboard) and an **embedding model**: `UIApplicationMain` owns the run loop; Rust schedules on the main queue; host code bridges **touches → `PlatformInput`** and optional **`CAMetalLayer`** via **`IosWindow`**. **Arcadia has not yet shipped** a full OF iOS app — this section tracks **gaps and phases** until SwiftUI can be removed.

### 11.1 Migration-specific gaps

| Gap | Description | Directions |
|-----|-------------|------------|
| **Build / link model** | Today iOS links **`ArcadiaCore.xcframework`** (UniFFI from Swift). An OF shell needs a clear **Rust artifact** layout: e.g. **one iOS static lib** combining **`arcadia-core` + `openframe`**, with **minimal Swift** for `UIApplication` + Metal layer + input injection — or a deliberately designed **xcframework** story. Avoid **Rust → Swift → Rust** for hot paths; prefer the shell crate **calling `arcadia_core` directly** where possible. | Spike **Phase 0**; document the chosen **single entrypoint** and update **`build-ios-framework.sh`** / Xcode only as needed. |
| **OpenFrame iOS maturity** | Platform code exists under **`Libraries/OpenFrame/src/platform/ios/`** but is **unproven** at Arcadia’s scale (scenes, multitasking, safe areas, software keyboard, focus). | Time-boxed **device spike**; fix issues **in OpenFrame** when they are framework bugs. |
| **Accessibility** | SwiftUI provides **VoiceOver, Dynamic Type, system materials** with little effort. GPUI/OpenFrame on iOS must **explicitly** address **a11y and contrast** or accept regression until implemented. | Milestone before calling migration “done”: define **minimum a11y bar** (labels, focus, type scaling or explicit opt-out). |
| **Visual parity** | Current iOS UI uses **glass / `.ultraThinMaterial`** and custom gradients. OF may not match **pixel-parity**; need a product decision: **match desktop OF aesthetic** vs **iOS-specific OF theme**. | Centralize tokens in **Rust theme** (mirror Desktop **`gui/theme/`**), not inline colors in views. |
| **Dispatch anti-pattern** | **`ContentView+Layout`** uses **hardcoded `activePage.id` branches** for page content. The new shell must **not** transcribe this to Rust — use **registry-driven** routing and **`required_module`** for visibility. | Implement page dispatch via **navigation registry** + shared patterns with Desktop where reasonable. |
| **Icons** | iOS uses **SF Symbols** (`system_image` in JSON); Desktop uses **SVG** via **`icon_path()`**. | Choose one strategy: **ship SVG assets on iOS**, or **map glyphs to OF-rendered assets / labels** for consistency. |

### 11.2 Phased plan (reference)

1. **Phase 0 — Spike:** `cargo check --target aarch64-apple-ios` for OF + new shell crate; **minimal Swift** + **Metal + touch injection**; run on **device**.  
2. **Phase 1 — Infrastructure:** iOS **Rust shell crate** in repo; **CI** `cargo check` for `aarch64-apple-ios`; document **lifecycle + Metal** embedding.  
3. **Phase 2 — Chrome:** Sidebar, **remote route**, top-bar quick pages, **navigation registry** load (bundled JSON + **`surface.snapshot`** merge — same semantics as today’s **`reloadModules()`**).  
4. **Phase 3 — Features:** Migrate **Modules → Shell (`shell.execute`) → LAN / network → Late / experimental → Splash**, reusing Desktop OF patterns where applicable.  
5. **Phase 4 — Thin client:** Mirror drain timer, route persistence, align with **§1–2** (do not trust **`surface.revision`** alone).  
6. **Phase 5 — Cutover:** Remove SwiftUI views; **delete duplicated theme** once Rust theme owns tokens.

**Exit criteria:** Feature parity **no worse** than current SwiftUI app; **registry-driven** shell; **no duplicated core logic** in the surface; **`surface.*`** only for mirrored UI state.

### 11.3 Interaction with other gaps

- **§10** — OF iOS shell must use **`execute_command`** for capabilities and surface **“unavailable on this surface”** from core for PTY/TUI-class features.  
- **§6** — Any new mirrored fields stay under **`SurfaceSnapshot.extra`** / **`SurfacePatch`**.  
- **§8** — Extend CI with the OF iOS crate **after** the crate lands.

---

## Summary

The shipped model is **good enough for feature development** on a **trusted LAN** with **one logical host** and **thin clients** that periodically **`surface.snapshot`**. Closing gaps **1–10** moves toward **stronger freshness guarantees**, **safer multi-writer behavior**, and **production-grade sync/security**.

**§11** tracks the **iOS SwiftUI → OpenFrame** migration: embedding, parity, accessibility, and **registry-first** UI — until the SwiftUI shell can be **removed** without losing thin-client or module functionality.
