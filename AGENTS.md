# Arcadia — Agent Instructions

Full reference (architecture, patterns, anti-patterns, build, gotchas) is in `CLAUDE.md`. This file is the pre-task checklist: file ownership, decision tree, and production gate.

---

## File Ownership

| File | Purpose | Rule |
|------|---------|------|
| `Shared/ArcadiaCore/src/config/modules.rs` | Module registry + config + migrations | Extend `MODULE_REGISTRY`; add migrations to `merge_defaults()`; never add per-module booleans |
| `Shared/ArcadiaCore/src/config/workspace.rs` | Workspace directory registry | `WorkspacesConfig`, `WorkspaceEntry`, `WorkspacePermissionDef`; `workspace.toml` |
| `Shared/ArcadiaCore/src/navigation.rs` | Page/group registry + JSON serialization | Extend `PAGE_DEFINITIONS` / `GROUP_DEFINITIONS` / `SETTINGS_HUB_PAGE_IDS`; never add parallel lists |
| `Shared/ArcadiaCore/src/modules/surface.rs` | Snapshot / patch / revision | Extend `extra` + `SurfacePatch`; never create `remote-session.*` verbs |
| `Shared/ArcadiaCore/src/modules/remote_mirror.rs` | Host transcript queue + drain | For inbound NODE_EXEC mirroring only |
| `Shared/ArcadiaCore/src/modules/{shell,lan,net,late,workspace,...}.rs` | Module command handlers | One file (or folder) per module; no cross-module logic |
| `Desktop/src/main.rs` | Desktop binary entry | Thin — pick GUI vs headless and hand off; no business logic |
| `Desktop/src/ios_lib.rs` | iOS C ABI surface | Keep small; mirror declarations in `ArcadiaBridge.h` |
| `Desktop/src/gui/app/mod.rs` | GUI root state (`ArcadiaRoot`) | No per-module booleans; no hardcoded page IDs |
| `Desktop/src/gui/app/entry.rs` | Desktop GPUI bootstrap | Window setup only |
| `Desktop/src/gui/app/entry_ios.rs` | iOS Metal-layer bootstrap | Wire `arcadia_ios_start` to OpenFrame on the supplied `CAMetalLayer` |
| `Desktop/src/gui/app/workspace_panel.rs` | Workspaces page | List + search; loads `WorkspacesConfig`; renders `workspace_row` per entry |
| `Desktop/src/gui/app/workspace_row.rs` | Per-workspace row | Iterates enabled modules for `workspace_permissions`; grant/revoke toggles |
| `Shared/ArcadiaCore/src/modules/ai_sandbox.rs` | AI filesystem + exec sandbox | Only entry point for AI file/exec ops; `EXEC_ALLOWLIST` lives here; path canonicalization enforced here |
| `Shared/ArcadiaCore/src/modules/ai_types.rs` | AI type definitions + workspace scope | `AiWorkspaceContext.is_path_in_scope` uses `fs::canonicalize` — do not weaken to string comparison |
| `Desktop/src/gui/app/workspace_create_modal.rs` | Create-workspace modal | Label + path fields; calls `workspace.add` on confirm |
| `Desktop/src/gui/theme/icons.rs` | Icon path helper | `icon_path(glyph_key)` — all SVG lookups; never inline asset paths in views |
| `Desktop/src/gui/theme/mod.rs` | Color + accent helpers | All color constants; never inline `rgb(0x...)` in views |
| `Documentation/Features/*.md` | In-depth feature docs | Update the relevant file(s) when the feature's source changes — see doc maintenance rules in `CLAUDE.md` |
| `Desktop/src/gui/tui/` | PTY/TUI terminal emulator | Desktop-feature only |
| `Mobile/iOS/ArcadiaApp/ArcadiaApp.swift` | UIKit @main | Only: configure Metal layer, call `arcadia_ios_start` with config root |
| `Mobile/iOS/ArcadiaApp/MetalHostView.swift` | `CAMetalLayer` host view | Only: forward `UITouch` via `arcadia_ios_inject_touch` |
| `Mobile/iOS/ArcadiaApp/ArcadiaBridge.h` | C ABI declarations | Mirror of `Desktop/src/ios_lib.rs` exports — keep in sync |
| `Mobile/iOS/ArcadiaApp.xcodeproj/project.pbxproj` | Xcode project + cargo build phase | Build phase already shells out to cargo — do not duplicate it |

---

## Decision Tree — Before Writing Code

1. **Does a registry entry exist?** → Create it before touching surface code.
2. **Am I name-checking a specific module or page ID in surface code?** → Belongs in the registry declaration or core.
3. **Am I adding a field that tracks a specific module's state?** → Use `is_module_enabled(name)` with a `*_MODULE_NAME` constant.
4. **Am I writing logic in `Desktop/src/main.rs` or `Mobile/iOS/ArcadiaApp/`?** → Belongs in `arcadia-core` or `Desktop/src/gui/app/`.
5. **Am I inlining a color?** → Theme layer only.
6. **Did I change `ios_lib.rs` exports?** → Update `ArcadiaBridge.h` and commit both.
7. **Am I renaming a module?** → Add a `merge_defaults()` migration.
8. **Am I creating a `remote-session.*` command for UI state?** → Use `surface.snapshot` / `surface.patch`.
9. **Am I reintroducing UniFFI / `Generated/` / `ArcadiaCore.xcframework`?** → No. Dead by design.
10. **Am I passing credentials (API keys, tokens, secrets) through a channel or struct field?** → Load them at the point of use (in the inference thread, in the config read). Never put secrets in `enum` variants or message payloads.
11. **Am I checking whether a path is inside a workspace scope?** → Use `std::fs::canonicalize` on both paths before comparing. `starts_with()` on raw strings is bypassed by `..` traversal.
12. **Am I calling `sh -c` or any shell with user-supplied or model-supplied input?** → Check the binary against `EXEC_ALLOWLIST` in `ai_sandbox.rs`. If the binary isn't on the list, reject it.
13. **Am I making an outbound HTTP call (Ollama, OpenAI, any provider)?** → Use `ureq::AgentBuilder::new().timeout(HTTP_TIMEOUT).build()`. Never call `ureq::post()` directly — no timeout means the UI can hang forever.
14. **Is my file over 400 lines?** → Plan a split into submodules before the PR. Files over 600 lines are blocked. (Exception: generated or test files.)
15. **Am I returning `None` or a fallback when a user-visible operation fails?** → Return `Err(String)` with a specific message instead. The UI layer must show the error, not silently degrade.
16. **Did I add, rename, or change a feature (module, page, command, type, config field)?** → Update the corresponding `Documentation/Features/*.md` file. The mapping is in `CLAUDE.md` under "Documentation Maintenance".

---

## Production Readiness Checklist

- [ ] New module registered in `MODULE_REGISTRY` with all 7 fields (`name`, `version`, `description`, `required_modules`, `required_permissions`, `workspace_permissions`, `supported_platforms`)
- [ ] New page registered in `PAGE_DEFINITIONS` and added to the correct ID list (`GROUP_DEFINITIONS`, `SETTINGS_HUB_PAGE_IDS`, `GLOBAL_PAGE_IDS`, or `TOP_BAR_PAGE_IDS`)
- [ ] No hardcoded module/page IDs in surface visibility or dispatch logic
- [ ] No per-module boolean fields in surface state structs
- [ ] No inline `rgb(0x...)` in view/render code
- [ ] iOS C ABI changes in `ios_lib.rs` reflected in `ArcadiaBridge.h`
- [ ] Module rename includes `merge_defaults()` migration
- [ ] New mirrored state uses `surface.*` protocol, not ad-hoc verbs
- [ ] `cargo test -p arcadia-core --manifest-path Shared/Cargo.toml` passes
- [ ] No secrets (API keys, tokens) in enum variants, channel payloads, or struct fields that cross thread boundaries — load at point of use
- [ ] All workspace path checks use `fs::canonicalize` on both sides — no raw `starts_with()` comparisons
- [ ] All outbound HTTP calls go through an `AgentBuilder` with `HTTP_TIMEOUT` — no bare `ureq::post()`
- [ ] All `sandboxed_exec` calls route through the binary allowlist in `ai_sandbox.rs`
- [ ] No `expect()` or `unwrap()` in production paths (outside `#[cfg(test)]`) — use `Result` / `Option` with user-visible error messages
- [ ] No file in `Shared/ArcadiaCore/src/modules/` or `Desktop/src/gui/app/` exceeds 600 lines without a documented split plan
- [ ] New AI provider routes do not carry credentials in `ProviderRouting` — the inference thread loads config directly
- [ ] `Documentation/Features/*.md` updated for any changed module, page, command, config field, or type (see mapping in `CLAUDE.md` § Documentation Maintenance)
