//! Example Arcadia WASM module, built with `arcadia-module-sdk`.
//!
//! The `register_module!` macro emits the entire ABI (manifest custom section, allocator,
//! `arcadia_dispatch`); the author writes only handler functions.
//!
//! ```sh
//! cargo build --release --target wasm32-unknown-unknown \
//!   --manifest-path ModuleSDK/examples/hello/Cargo.toml
//! ```
//!
//! then drop the resulting `hello_wasm.wasm` into `~/Arcadia/Modules/`.

use arcadia_module_sdk::register_module;

fn greet(args: Vec<String>) -> String {
    let who = args.first().map(String::as_str).unwrap_or("world");
    arcadia_module_sdk::log(&format!("hello-wasm: greeting {who}"));
    format!("Hello, {who}")
}

register_module! {
    name: "hello-wasm",
    version: "0.1.0",
    description: "Example WASM module.",
    commands: [
        { verb: "greet", description: "Greet someone: hello-wasm.greet <name>", handler: greet },
    ],
}
