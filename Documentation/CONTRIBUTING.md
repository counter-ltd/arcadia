# Contributing

Read `AGENTS.md` — it has the registry-discipline rules and the full list of anti-patterns we refuse to write. Short version:

1. **Registry entry before surface code.** New module? `MODULE_REGISTRY` first. New page? `PAGE_DEFINITIONS` first.
2. **No per-module booleans in surface state.** One generic `is_module_enabled(name)` query.
3. **No hardcoded page ID match arms in visibility logic.** Derive from `required_module` in `PageDefinition`.
4. **No inline colors.** Theme layer only.
5. **Cross-platform logic belongs in core.** Desktop and iOS share the same Rust UI — if you're writing the same thing in `Desktop/src/main.rs` and `Mobile/iOS/ArcadiaApp/`, it almost certainly belongs in `arcadia-core` or `gui/app/`.
6. **iOS C ABI changes:** edit `Desktop/src/ios_lib.rs` and mirror in `Mobile/iOS/ArcadiaApp/ArcadiaBridge.h`. Commit both.

If something's missing: open a PR, draft a module, or file an issue with a concrete repro.

---

## Adding features

### New module

1. Add constant + `ModuleManifest` to `MODULE_REGISTRY` in `Shared/ArcadiaCore/src/config/modules.rs`.
2. Create `Shared/ArcadiaCore/src/modules/x.rs` with a `commands()` fn returning `&[ModuleCommand]`.
3. Register in `Shared/ArcadiaCore/src/modules/mod.rs`.
4. Done. GUI, CLI, and iOS module list updates automatically — no surface edits required.

### New navigation page

1. Add `NavigationPageDefinition` to `PAGE_DEFINITIONS` in `navigation.rs`. Set `required_module` if visibility depends on a module.
2. Add the page ID to the relevant `GROUP_DEFINITIONS.pages` slice, or create a new group.
3. Implement the page panel under `Desktop/src/gui/app/` — the same panel renders on desktop and iOS via OpenFrame.
4. Route it in the surface content switch via the page ID — derive visibility from `required_module`, not a hardcoded match.

### New icon/glyph

1. Add SVG to `Desktop/assets/icons/`.
2. Add match arm to `icon_path()` in `Desktop/src/gui/theme/mod.rs`.
3. Use the key in `NavigationPageDefinition.glyph` or `NavigationGroupDefinition.glyph`.

### New theme color

- Add named constant or helper fn to `Desktop/src/gui/theme/mod.rs` or the relevant component token file under `theme/modules/`.
- Never inline `rgb(0x...)` in view files.

### New mirrored state

Extend `SurfaceSnapshot.extra` and add a `SurfacePatch` variant in `modules/surface.rs`. Wire the consuming panel to read the new extra field from snapshot. Do not create ad-hoc `remote-session.*` verbs — keep the protocol under `surface.*`.

### Renaming a module

Edit `MODULE_REGISTRY` name and constant. Add a migration to `ModulesConfig::merge_defaults()` following the `LEGACY_LAN_MODULE_NAME` pattern. Do not rename at call sites.

---

## Testing

Current test coverage is sparse. Priority areas for expansion:

```sh
# Run existing tests
cd Shared && cargo test -p arcadia-core

# What to add:
# - surface.snapshot / parse_surface_snapshot round-trips
# - NavigationRegistryOwned JSON serialization/deserialization
# - ModulesConfig migration (merge_defaults with legacy keys)
# - thin-client preference persistence (set → get → re-load)
# - LAN routing integration (execute_command with net_as)
# - Module enable/disable with dependency enforcement
```

The iOS static lib is built automatically by the Xcode project's "Build Rust (cargo)" phase whenever you build `ArcadiaApp` — no separate rebuild step. CI replicates this via `xcodebuild` after installing the `aarch64-apple-ios-sim` target.
