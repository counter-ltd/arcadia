# Contributing

Read `AGENTS.md` — it has the registry-discipline rules and the full list of anti-patterns we refuse to write. Short version:

1. **Registry entry before surface code.** New module? `MODULE_REGISTRY` first. New page? `PAGE_DEFINITIONS` first.
2. **No per-module booleans in surface state.** One generic `is_module_enabled(name)` query.
3. **No hardcoded page ID match arms in visibility logic.** Derive from `required_module` in `PageDefinition`.
4. **No inline colors.** Theme layer only — `theme::ui_accent(cx)`, `p.content_title`, etc. No `rgb(0x...)` in view files.
5. **Cross-platform logic belongs in core.** Desktop and iOS share the same Rust UI — if you're writing the same thing in `Desktop/src/main.rs` and `Mobile/iOS/ArcadiaApp/`, it almost certainly belongs in `arcadia-core` or `gui/app/`.
6. **iOS C ABI changes:** edit `Desktop/src/ios_lib.rs` and mirror in `Mobile/iOS/ArcadiaApp/ArcadiaBridge.h`. Commit both.

### Security rules (mandatory — not optional style)

7. **No secrets in channel payloads or enum variants.** API keys, tokens, and passwords must be loaded at the point of use inside the consuming thread. Do not put them in `enum` variants, struct fields that cross thread boundaries, or mpsc messages.
8. **Canonicalize paths before scope checks.** `is_path_in_scope` must call `std::fs::canonicalize` on both paths. Raw `starts_with()` comparisons on path strings are vulnerable to `..` traversal attacks and are banned for security decisions.
9. **All model/user-supplied shell commands go through `sandboxed_exec`.** Never call `Command::new("sh")` with model-supplied input directly. `ai_sandbox::sandboxed_exec` enforces the `EXEC_ALLOWLIST`; bypass it and the PR is rejected.
10. **All outbound HTTP calls use a timeout.** Use `ureq::AgentBuilder::new().timeout(HTTP_TIMEOUT).build()`. Bare `ureq::post()` is not allowed in production paths — a hanging server will hang the inference thread permanently.

### Code health rules (mandatory)

11. **No `expect()` or `unwrap()` outside `#[cfg(test)]`.** Return `Result`/`Option` and propagate or show the error. Panicking production code is a crash, not error handling.
12. **No silent error swallowing.** `.ok()` on a user-visible operation discards the error silently. Either propagate with `?` or log + show to user. `.ok()` is only acceptable when a missing value is genuinely expected (e.g., optional config keys).
13. **File size limit: 600 lines.** Files over 400 lines need a note; files over 600 lines in `modules/` or `gui/app/` are blocked without a split plan attached to the PR.

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
