# GAPS — WASM Module Loader Migration

Status of the dynamic WASM module loader (branch `development-modularity-loader`).

## Closed — the loader is feature-complete

The MVP pipeline plus all gap-closing work has landed:

- **SDK** — `ModuleSDK/arcadia-module-sdk`: a `register_module!` macro emits the whole
  host/guest ABI (manifest custom section, allocator, `arcadia_dispatch`). Authors write
  only handler functions. `ModuleSDK/README.md` documents authoring + the
  do-not-strip-custom-sections rule.
- **Host callbacks** — `host_execute_command` lets a module re-enter Arcadia's command
  dispatch; `host_has_permission` lets it introspect its grants. `ExecutionContext`
  carries `invoking_wasm_module` so nested permission checks accept `wasm:<id>` grants.
- **Re-entrancy** — a thread-local dispatch stack rejects same-module recursion (which
  would deadlock the per-module `Mutex`); cross-module chains (A→B→A) work. Covered by
  `wasm_registry` tests.
- **Permission gating** — `HostState.granted_permissions` resolved at instantiation from
  `PermissionsConfig`. The GUI page shows a first-enable permission modal.
- **Folder bundles** — discovery handles both loose `Modules/<name>.wasm` and
  `Modules/<name>/module.wasm` + `Assets/`. `wasm_registry` exposes
  `module_bundle_root` / `resolve_module_asset_path`.
- **GUI** — a Modules settings page (`wasm-modules.settings`) with search, enable/disable
  toggles, the permission modal, and **per-module icons** (a `module-icon/<name>` asset
  prefix resolves `Assets/icon.svg`, falling back to the generic glyph). The WASM host
  starts from the GUI lifecycle (startup + runtime-enable), not just headless.
- **iOS** — `wasm-modules` wired into `ios-gui`; the `wasmi` staticlib builds for
  `aarch64-apple-ios`.
- **Tests** — `arcadia-wasm` covers instantiate / dispatch / ABI-mismatch /
  missing-export, plus memory-ABI hardening (bad result pointer, oversized input).
  `wasm_registry` covers discover/enable/clear/reload/unregister and the same-module
  recursion guard. `extension::wasm_manifest` is tested in `arcadia-core`.

## Remaining

- **iOS on-device dispatch** — the staticlib compiles for iOS; end-to-end command
  dispatch is verified on desktop only. A simulator/device run is a manual follow-up
  (deliberately out of scope for the current pass).
- **Concurrency** — dispatch is serialized per module (per-module `Mutex` around the
  non-`Sync` `wasmi` Store). A slow command blocks other commands on the same module.
  This is an accepted design tradeoff, not a defect.
