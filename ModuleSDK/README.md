# Arcadia Module SDK

Tools for authoring **WASM modules** for Arcadia — Rust code compiled to
`wasm32-unknown-unknown`, dropped into `~/Arcadia/Modules/`, loaded and dispatched at
runtime by the `wasm-host` module.

This directory is author-facing tooling. It is **not** part of the Arcadia build — the
`Shared/` workspace does not include it.

```
ModuleSDK/
  arcadia-module-sdk/    the SDK crate — register_module! macro + host bindings
  examples/hello/        a minimal example module
```

## Writing a module

A module is a `cdylib` crate that depends on `arcadia-module-sdk` and calls
`register_module!`. The macro emits the entire host/guest ABI; you write only handler
functions.

```rust
use arcadia_module_sdk::register_module;

fn greet(args: Vec<String>) -> String {
    let who = args.first().map(String::as_str).unwrap_or("world");
    arcadia_module_sdk::log(&format!("greeting {who}"));
    format!("Hello, {who}")
}

register_module! {
    name: "hello-wasm",
    version: "0.1.0",
    description: "Example WASM module.",
    commands: [
        { verb: "greet", description: "Greet someone", handler: greet },
    ],
}
```

`Cargo.toml`:

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
arcadia-module-sdk = { path = "../../arcadia-module-sdk" }
```

## Building

```sh
cargo build --release --target wasm32-unknown-unknown \
  --manifest-path ModuleSDK/examples/hello/Cargo.toml
```

The resulting `<name>.wasm` goes into `~/Arcadia/Modules/`, either loose or as a folder
bundle `~/Arcadia/Modules/<name>/module.wasm` (with an optional `Assets/` directory).

> **Do not run `wasm-opt --strip*` or `wasm-strip`** on the output. The module's manifest
> lives in a wasm *custom section* (`arcadia.manifest`); stripping removes custom sections
> and the host will not see the module. A plain `cargo build --release` is correct.

## Host bindings

The SDK exposes functions the module calls back into Arcadia with:

- `arcadia_module_sdk::log(msg)` / `log_at(level, msg)` — write to the host log.
- `arcadia_module_sdk::execute_command(token, args)` — run an Arcadia command
  (`module.verb`) and get its result. Subject to the permissions the module declared.
  A module calling one of *its own* commands is rejected (same-module recursion).
- `arcadia_module_sdk::has_permission(id)` — whether the module was granted a permission
  it declared.

## ABI version

The SDK and host share `ABI_VERSION = 1`. A module built against a mismatched SDK is
rejected at load with a clear error.
