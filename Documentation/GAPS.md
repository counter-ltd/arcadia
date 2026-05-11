# Arcadia — Thin-client / remote-surface status

This file previously listed **open gaps**. Those items are now either **implemented**, **documented as permanent constraints**, or tracked as **explicit roadmap** in **`REMOTE_AND_IOS_CONSTRAINTS.md`**.

---

## Resolved in tree

| Topic | Where |
|-------|--------|
| **`surface.revision` tracks all `modules.toml` saves** | `surface_revision` + `ModulesConfig::save` override |
| **Thin-client stale detection** | Poll `surface.revision`; banner + Reload in GUI (`ArcadiaRoot`) |
| **`surface.revision` lightweight poll token** | `surface.revision` command |
| **`SurfaceSnapshot.extra` schema** | Documented `schema_version` + keys in `modules/surface.rs` |
| **Renderer-only nav SKU** | `thin-client.toml` → `navigation_from_host_only` |
| **iOS constrained shell** | `shell_ios` panel (`shell.execute` / LAN only) |
| **Stable “unavailable on this surface” strings** | `arcadia_core::capabilities` + shell handlers |
| **CI: iOS cross-check + C ABI header drift** | `stable-build-matrix.yml`, `Shared/Scripts/check-ios-bridge-header.sh` |
| **Tests** | `parse_surface_revision`, navigation snapshot round-trip, thin-client TOML serde |
| **Permission catalog + `permissions.toml`** | `config/permissions` + `modules/permissions`; `input.capture` is registered but **not yet enforced** at input injection sites (desktop / `arcadia_ios_inject_touch`) — toggles + CLI work; wire checks when those entrypoints gain a single gate. |

---

## Constraints & roadmap (not bugs)

Multi-writer LWW on `modules.toml`, discrete LAN transport, attribution-only `client_id`, trusted-LAN security model, and iOS accessibility/visual parity beyond current OpenFrame — see **`Documentation/REMOTE_AND_IOS_CONSTRAINTS.md`**.

**OpenFrame Wayland `WindowStacking`:** semantic stacking (`Hud`, `SystemUi`, etc.) is accepted and logged but not mapped to `zwlr_layer_shell_v1` yet; xdg-shell windows keep default compositor stacking until layer-shell is wired.

---

## Contributing

When extending mirrored host state, keep **`surface.*`** as the protocol surface (`SurfaceSnapshot.extra`, `SurfacePatch`). Follow **`CLAUDE.md`** / **`AGENTS.md`** registry-driven rules.
