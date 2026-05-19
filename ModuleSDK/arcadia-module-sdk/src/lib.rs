//! SDK for authoring Arcadia WASM modules.
//!
//! A module crate depends on this, compiles to `wasm32-unknown-unknown`, and uses
//! [`register_module!`] to declare itself. The macro emits the whole host/guest ABI —
//! the `arcadia.manifest` custom section, the memory allocator, `arcadia_abi_version`,
//! and `arcadia_dispatch` — so authors write only command handler functions.
//!
//! ```ignore
//! use arcadia_module_sdk::register_module;
//!
//! fn greet(args: Vec<String>) -> String {
//!     format!("Hello, {}", args.first().map(String::as_str).unwrap_or("world"))
//! }
//!
//! register_module! {
//!     name: "hello-wasm",
//!     version: "0.1.0",
//!     description: "Example module.",
//!     commands: [
//!         { verb: "greet", description: "Greet someone", handler: greet },
//!     ],
//! }
//! ```
//!
//! ## Build requirement
//!
//! The manifest lives in a wasm custom section. Do **not** run `wasm-opt --strip*` or
//! `wasm-strip` on the output — that removes custom sections and the host will not see
//! the module. A plain `cargo build --release --target wasm32-unknown-unknown` is fine.

/// ABI version this SDK speaks. Must match the host's `HOST_ABI_VERSION`.
pub const ABI_VERSION: i32 = 1;

// ─── Host imports (module "arcadia") ────────────────────────────────────────

#[link(wasm_import_module = "arcadia")]
extern "C" {
    fn host_log(level: i32, ptr: i32, len: i32);
    fn host_has_permission(ptr: i32, len: i32) -> i32;
    fn host_execute_command(
        tok_ptr: i32,
        tok_len: i32,
        args_ptr: i32,
        args_len: i32,
    ) -> i64;
}

/// Log levels for [`log`].
pub mod log_level {
    pub const INFO: i32 = 1;
    pub const WARN: i32 = 2;
    pub const ERROR: i32 = 3;
}

/// Write a line to the host log.
pub fn log(message: &str) {
    log_at(log_level::INFO, message);
}

/// Write a line to the host log at an explicit level.
pub fn log_at(level: i32, message: &str) {
    unsafe { host_log(level, message.as_ptr() as i32, message.len() as i32) }
}

/// Whether the module was granted `permission_id` — one of the permissions it declared
/// in `register_module!`. Lets a module check a capability before relying on it.
pub fn has_permission(permission_id: &str) -> bool {
    let granted = unsafe {
        host_has_permission(permission_id.as_ptr() as i32, permission_id.len() as i32)
    };
    granted != 0
}

/// Run an Arcadia command (`module.verb`) and return its result. Lets a module act on
/// the host — subject to the permissions the module declared. A module calling one of
/// its own commands is rejected by the host (same-module recursion).
pub fn execute_command(token: &str, args: &[&str]) -> String {
    let args_json = json_string_array(args);
    let packed = unsafe {
        host_execute_command(
            token.as_ptr() as i32,
            token.len() as i32,
            args_json.as_ptr() as i32,
            args_json.len() as i32,
        )
    };
    let ptr = (packed >> 32) as i32;
    let len = (packed & 0xffff_ffff) as i32;
    if ptr == 0 || len <= 0 {
        return String::new();
    }
    let slice = unsafe { core::slice::from_raw_parts(ptr as *const u8, len as usize) };
    let out = String::from_utf8_lossy(slice).into_owned();
    // The host wrote the result through our own `arcadia_alloc`; free it.
    dealloc_impl(ptr, len);
    out
}

/// Encode `args` as a JSON string array, escaping `"` and `\`.
fn json_string_array(args: &[&str]) -> String {
    let mut s = String::from("[");
    for (i, a) in args.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push('"');
        for c in a.chars() {
            if c == '"' || c == '\\' {
                s.push('\\');
            }
            s.push(c);
        }
        s.push('"');
    }
    s.push(']');
    s
}

// ─── ABI helpers — called by the macro-emitted exports ──────────────────────

/// Allocate `len` bytes of guest memory for the host to write into. `vec![0u8; len]`
/// yields capacity exactly `len`, so [`dealloc_impl`] can reconstruct the `Vec` safely.
#[doc(hidden)]
pub fn alloc_impl(len: i32) -> i32 {
    let buf = vec![0u8; len.max(0) as usize];
    let ptr = buf.as_ptr() as i32;
    core::mem::forget(buf);
    ptr
}

/// Free memory previously handed out by [`alloc_impl`].
#[doc(hidden)]
pub fn dealloc_impl(ptr: i32, len: i32) {
    if ptr == 0 {
        return;
    }
    let len = len.max(0) as usize;
    unsafe {
        drop(Vec::from_raw_parts(ptr as *mut u8, len, len));
    }
}

/// Read a UTF-8 string the host wrote into guest memory.
#[doc(hidden)]
pub fn read_guest(ptr: i32, len: i32) -> String {
    if ptr == 0 || len <= 0 {
        return String::new();
    }
    let slice = unsafe { core::slice::from_raw_parts(ptr as *const u8, len as usize) };
    String::from_utf8_lossy(slice).into_owned()
}

/// Move a result string into guest memory; returns the packed `(ptr << 32) | len` the
/// host unpacks.
#[doc(hidden)]
pub fn pack(s: String) -> i64 {
    let bytes = s.into_bytes();
    let len = bytes.len() as i64;
    let ptr = bytes.as_ptr() as i64;
    core::mem::forget(bytes);
    (ptr << 32) | len
}

/// Decode a flat JSON string array (`["a","b"]`) into `Vec<String>`. Handles `\"` and
/// `\\` escapes; other content is taken literally. Dependency-free — modules stay small.
#[doc(hidden)]
pub fn parse_args(json: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = json.chars().peekable();
    // Skip to the opening bracket.
    for c in chars.by_ref() {
        if c == '[' {
            break;
        }
    }
    loop {
        // Advance to the next opening quote or the closing bracket.
        let mut in_string = false;
        for c in chars.by_ref() {
            if c == '"' {
                in_string = true;
                break;
            }
            if c == ']' {
                return out;
            }
        }
        if !in_string {
            return out;
        }
        let mut s = String::new();
        let mut escaped = false;
        for c in chars.by_ref() {
            if escaped {
                s.push(c);
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                break;
            } else {
                s.push(c);
            }
        }
        out.push(s);
    }
}

/// Internal: recursively joins command JSON objects with commas — no trailing comma,
/// so the result is strict JSON. `concat!` evaluates the nested expansion.
#[doc(hidden)]
#[macro_export]
macro_rules! __arcadia_cmds_json {
    () => { "" };
    ({ $v:literal, $d:literal }) => {
        concat!(
            r#"{"verb":""#, $v, r#"","description":""#, $d,
            r#"","required_permissions":[]}"#
        )
    };
    ({ $v:literal, $d:literal } $($rest:tt)+) => {
        concat!(
            r#"{"verb":""#, $v, r#"","description":""#, $d,
            r#"","required_permissions":[]},"#,
            $crate::__arcadia_cmds_json!($($rest)+)
        )
    };
}

/// Declares an Arcadia WASM module: emits the manifest custom section and the full ABI.
#[macro_export]
macro_rules! register_module {
    (
        name: $name:literal,
        version: $version:literal,
        description: $description:literal,
        commands: [
            $( { verb: $verb:literal, description: $cdesc:literal, handler: $handler:path $(,)? } ),* $(,)?
        ] $(,)?
    ) => {
        /// Manifest JSON, embedded as the `arcadia.manifest` custom section so the host
        /// reads it without instantiating the module.
        #[doc(hidden)]
        const __ARCADIA_MANIFEST_JSON: &str = concat!(
            r#"{"name":""#, $name,
            r#"","version":""#, $version,
            r#"","description":""#, $description,
            r#"","abi_version":1,"commands":["#,
            $crate::__arcadia_cmds_json!($( { $verb, $cdesc } )*),
            r#"]}"#
        );

        #[link_section = "arcadia.manifest"]
        #[used]
        static __ARCADIA_MANIFEST: [u8; __ARCADIA_MANIFEST_JSON.len()] = {
            let src = __ARCADIA_MANIFEST_JSON.as_bytes();
            let mut arr = [0u8; __ARCADIA_MANIFEST_JSON.len()];
            let mut i = 0;
            while i < arr.len() {
                arr[i] = src[i];
                i += 1;
            }
            arr
        };

        #[no_mangle]
        pub extern "C" fn arcadia_abi_version() -> i32 {
            $crate::ABI_VERSION
        }

        #[no_mangle]
        pub extern "C" fn arcadia_alloc(len: i32) -> i32 {
            $crate::alloc_impl(len)
        }

        #[no_mangle]
        pub extern "C" fn arcadia_dealloc(ptr: i32, len: i32) {
            $crate::dealloc_impl(ptr, len)
        }

        #[no_mangle]
        pub extern "C" fn arcadia_dispatch(
            verb_ptr: i32,
            verb_len: i32,
            args_ptr: i32,
            args_len: i32,
        ) -> i64 {
            let verb = $crate::read_guest(verb_ptr, verb_len);
            let args = $crate::parse_args(&$crate::read_guest(args_ptr, args_len));
            let result: String = match verb.as_str() {
                $( $verb => $handler(args), )*
                other => ::std::format!("unknown verb: {other}"),
            };
            $crate::pack(result)
        }
    };
}
