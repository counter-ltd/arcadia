# Arcadia — Gap Tracking

The canonical gap list lives in **[`ROADMAP.md`](ROADMAP.md)** — P0 through P3, AI system gaps, and security posture. This file covers topics that either don't fit neatly into the priority tiers or need a more narrative form.

---

## Thin-client / remote-surface status

| Topic | Where |
|-------|--------|
| **`surface.revision` tracks all `modules.toml` saves** | `surface_revision` + `ModulesConfig::save` override |
| **Thin-client stale detection** | Poll `surface.revision`; banner + Reload in GUI (`ArcadiaRoot`) |
| **`surface.revision` lightweight poll token** | `surface.revision` command |
| **`SurfaceSnapshot.extra` schema** | Documented `schema_version` + keys in `modules/surface.rs` |
| **Renderer-only nav SKU** | `thin-client.toml` → `navigation_from_host_only` |
| **iOS constrained shell** | `shell_ios` panel (`shell.execute` / LAN only) |
| **Stable "unavailable on this surface" strings** | `arcadia_core::capabilities` + shell handlers |
| **CI: iOS cross-check + C ABI header drift** | `stable-build-matrix.yml`, `Shared/Scripts/check-ios-bridge-header.sh` |
| **Tests** | `parse_surface_revision`, navigation snapshot round-trip, thin-client TOML serde |
| **Permission catalog + `permissions.toml`** | `config/permissions` + `modules/permissions`; `input.capture` is registered but **not yet enforced** at input injection sites (desktop / `arcadia_ios_inject_touch`) — toggles + CLI work; see P1.5 in ROADMAP. |

---

## Constraints & roadmap (not bugs)

Multi-writer LWW on `modules.toml`, discrete LAN transport, attribution-only `client_id`, trusted-LAN security model, and iOS accessibility/visual parity beyond current OpenFrame — see **`Documentation/REMOTE_AND_IOS_CONSTRAINTS.md`**.

**OpenFrame Wayland `WindowStacking`:** semantic stacking (`Hud`, `SystemUi`, etc.) is accepted and logged but not mapped to `zwlr_layer_shell_v1` yet; xdg-shell windows keep default compositor stacking until layer-shell is wired.

---

## WASM module loader

The dynamic WASM module loader (`Modules/` directory, branch
`development-modularity-loader`) is **feature-complete**:

- **SDK** — `ModuleSDK/arcadia-module-sdk`: a `register_module!` macro emits the whole
  host/guest ABI. Authoring guide in `ModuleSDK/README.md`.
- **Host callbacks** — `host_execute_command` (re-enters Arcadia dispatch),
  `host_has_permission`. `ExecutionContext::invoking_wasm_module` carries `wasm:<id>` for
  nested permission checks. A thread-local dispatch stack rejects same-module recursion.
- **Permission gating** — `HostState.granted_permissions` resolved at instantiation;
  GUI first-enable permission modal.
- **Folder bundles** — loose `Modules/<name>.wasm` and `Modules/<name>/module.wasm` +
  `Assets/`; per-module GUI icons via the `module-icon/<name>` asset prefix.
- **GUI** — `wasm-modules.settings` page (search, toggles, modal); host starts from the
  GUI lifecycle.
- **iOS** — `wasm-modules` in `ios-gui`; `wasmi` staticlib builds for `aarch64-apple-ios`.
- **Tests** — `arcadia-wasm` (instantiate / dispatch / ABI-mismatch / memory-ABI
  hardening), `wasm_registry` (lifecycle + recursion guard), `wasm_manifest`.

Remaining: **iOS on-device dispatch** is verified on desktop only — a simulator/device
run is a manual follow-up. Per-module dispatch is serialized (per-module `Mutex` around
the non-`Sync` `wasmi` Store) — an accepted tradeoff, not a defect.

---

## Contributing

When extending mirrored host state, keep **`surface.*`** as the protocol surface (`SurfaceSnapshot.extra`, `SurfacePatch`). Follow **`CLAUDE.md`** registry-driven rules.

For all other gap tracking — code quality, AI features, security, CLI parity — see **[`ROADMAP.md`](ROADMAP.md)**.
