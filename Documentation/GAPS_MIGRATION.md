# GAPS — WASM Module Loader Migration

Status of the dynamic WASM module loader (branch `development-modularity-loader`).

## Closed — the loader is feature-complete for v1

The MVP pipeline plus all v1 gap-closing work has landed:

- **SDK** — `ModuleSDK/arcadia-module-sdk`: a `register_module!` macro emits the whole
  host/guest ABI (manifest custom section, allocator, `arcadia_dispatch`). Authors write
  only handler functions. `ModuleSDK/README.md` documents authoring + the
  do-not-strip-custom-sections rule.
- **Host callbacks** — `host_execute_command` lets a module re-enter Arcadia's command
  dispatch; `host_has_permission` lets it introspect its grants. `ExecutionContext`
  carries `invoking_wasm_module` so nested permission checks accept `wasm:<id>` grants.
- **Re-entrancy** — a thread-local dispatch stack rejects same-module recursion (which
  would deadlock the per-module `Mutex`); cross-module chains (A→B→A) work.
- **Permission gating** — `HostState.granted_permissions` resolved at instantiation from
  `PermissionsConfig`. The GUI page shows a first-enable permission modal.
- **Folder bundles** — discovery handles both loose `Modules/<name>.wasm` and
  `Modules/<name>/module.wasm` + `Assets/`. `wasm_registry` exposes
  `module_bundle_root` / `resolve_module_asset_path`.
- **GUI** — a Modules settings page (`wasm-modules.settings`, copy of the Python
  extensions panel) with search, enable/disable toggles, and the permission modal. The
  WASM host starts from the GUI lifecycle (startup + runtime-enable), not just headless.
- **iOS** — `wasm-modules` is wired into the `ios-gui` feature; the `wasmi` staticlib
  builds for `aarch64-apple-ios`.
- **Tests** — `arcadia-wasm` has instantiate / dispatch / ABI-mismatch / missing-export
  tests (WAT fixtures); `extension::wasm_manifest` is tested in `arcadia-core`.

## Remaining — follow-ups, not v1 blockers

- **iOS on-device dispatch** — the staticlib compiles for iOS, but end-to-end command
  dispatch has only been verified on desktop. A simulator/device run is a manual
  follow-up.
- **Memory-ABI hardening** — happy path + bad-bytes / ABI-mismatch are tested. Large
  payloads, allocation failure under memory pressure, and adversarial guest pointers are
  not exhaustively fuzzed.
- **Reload** — `wasm-host.reload` works (`clear` + rescan) but has light coverage; a
  reloaded module loses in-memory state (acceptable, documented behavior).
- **Per-module GUI icons** — `resolve_module_asset_path` exists, but the settings page
  renders the generic `modules` glyph; wiring per-bundle `Assets/icon.svg` through the
  asset layer (a `module-icon/` prefix in `Desktop/src/gui/assets.rs`) is deferred.
- **Concurrency** — dispatch is serialized per module (per-module `Mutex` around the
  non-`Sync` `wasmi` Store). A slow command blocks other commands on the same module.
  Acceptable for v1.
