# GAPS — WASM Module Loader Migration

Status of the dynamic WASM module loader (branch `development-modularity-loader`).
The MVP pipeline is shipped and verified end-to-end; this file tracks what is **not**
done so the loader is not mistaken for complete.

## What works (MVP — commit `2a528ca`)

Discover `.wasm` in `~/Arcadia/Modules/` → read the `arcadia.manifest` custom section →
register a disabled stub → user enables → instantiate under `wasmi` → dispatch a command
across the host/guest ABI → result returned. Verified:

```
wasm-host.module-enable hello-wasm   → WASM module 'hello-wasm' enabled.
hello-wasm.greet World               → Hello, World
```

3 surfaces build; 6 extension parity tests green; `wasmi` confirmed to build for
`aarch64-apple-ios`; the custom section survives release builds.

---

## Functional gaps

### `register_module!` SDK macro — highest priority
The example module `ModuleSDK/examples/hello` hand-writes the entire ABI
(`arcadia_alloc` / `arcadia_dealloc` / `arcadia_abi_version` / `arcadia_dispatch`, the
`arcadia.manifest` custom section). Module authors need an `arcadia-module-sdk` crate at
`ModuleSDK/` providing a `register_module!` macro that emits that boilerplate and safe
wrappers for host imports. Without it the loader is not usable by real authors.

### `host_execute_command` — highest priority
MVP host functions are `host_log` only. Modules cannot call back into Arcadia, so a module
can compute but cannot *do* anything. Needs the `arcadia.host_execute_command` import,
re-entering `arcadia_core::modules::execute_command`, plus a **same-module re-entry guard**
— a module calling a command on itself would deadlock its own per-module `Mutex`.

### Permission gating
`required_permissions` from the manifest are parsed and stored in `wasm_registry`, and
`wasm_registry::try_dispatch` checks the command's declared permissions. But:
- `HostState.granted_permissions` (in `arcadia-wasm`) is never populated — gated host
  functions (`host_execute_command`, future `host_config_get`) have nothing to check.
- No first-enable permission modal: enabling a module does not prompt for the permissions
  it declares.

### Reload
`wasm-host.reload` is wired (`wasm_registry::clear` + rescan) but untested. Reload drops
each module's in-memory state — acceptable, but undocumented.

---

## Surface gaps

### GUI settings page
The `wasm-modules.settings` nav page is declared by `WasmHostExtension` but is **orphaned**
— not in any navigation group or placement list, and there is no panel
(`Desktop/src/gui/app/wasm_settings/`). The GUI start path is not wired either: only the
headless binary starts the host (`Desktop/src/main.rs`, `not(feature = "gui")` block). The
GUI must start `WasmModuleHost` from `lifecycle.rs`, paralleling the Python host, and
render an enable/disable panel (copy `Desktop/src/gui/app/python_settings/`).

### iOS
`wasm-modules` is excluded from the `ios-gui` feature. `wasmi` is iOS-safe (pure
interpreter — the Phase 0 spike confirmed it builds for `aarch64-apple-ios`), so this is
wiring plus on-device verification, not new design. iOS is the headline use case — it bans
loading native dylibs, which is the entire reason the loader uses WASM — so this gap
matters more than its size suggests.

---

## Robustness gaps

- **Memory ABI** — `arcadia_alloc` / `arcadia_dealloc` are exercised by exactly one example
  module. Edge cases untested: zero-length payloads, large payloads, allocation failure,
  capacity-vs-length mismatch in the guest allocator.
- **Folder bundles** — only loose `.wasm` files are discovered. No
  `Modules/<name>/module.wasm` + `Assets/` bundle layout (the Python host supports the
  equivalent `main.py` + `Assets/`).
- **Multi-module** — untested with more than one module loaded; no inter-module
  dependencies.
- **Build-flag docs** — module authors must not run `wasm-opt --strip` or similar, which
  removes custom sections and would drop `arcadia.manifest`. This hard requirement is
  undocumented.
- **No `arcadia-wasm` tests** — the crate has zero unit or integration tests. Only
  `extension::wasm_manifest` (in `arcadia-core`) is tested.

---

## Smaller items

- **Two-run friction** — the `wasm-host` module must itself be enabled (config edit +
  restart) before `WasmModuleHost::start` runs discovery. A single-run path would be
  friendlier.
- **Concurrency** — dispatch is serialized per module (per-module `Mutex` around the
  non-`Sync` `wasmi` Store). A slow WASM command blocks other commands on the same module.
  Acceptable for v1; revisit if it bites.

---

## Suggested order

1. `register_module!` SDK macro — unblocks real module authors.
2. `host_execute_command` + permission gating — modules become capable, not just pure.
3. GUI settings page + GUI start path.
4. iOS wiring + on-device verification.
5. Robustness: `arcadia-wasm` tests, memory-ABI edge cases, build-flag docs.
6. Reload, folder bundles, multi-module.
