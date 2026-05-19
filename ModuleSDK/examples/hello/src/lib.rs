//! MVP example Arcadia WASM module — hand-written ABI (no SDK macro yet).
//!
//! Implements the host/guest contract `arcadia-wasm` expects: the `arcadia.manifest`
//! custom section, the memory allocator exports, `arcadia_abi_version`, and
//! `arcadia_dispatch`. Build with:
//!
//! ```sh
//! cargo build --release --target wasm32-unknown-unknown \
//!   --manifest-path ModuleSDK/examples/hello/Cargo.toml
//! ```
//!
//! then drop the resulting `hello-wasm.wasm` into `~/Arcadia/Modules/`.

// The manifest, embedded as a custom section so the host can read it without
// instantiating the module. `MANIFEST_BYTES.len()` is const-evaluable.
const MANIFEST_BYTES: &[u8] = include_bytes!("../manifest.json");

#[link_section = "arcadia.manifest"]
#[used]
static ARCADIA_MANIFEST: [u8; MANIFEST_BYTES.len()] = *include_bytes!("../manifest.json");

/// ABI version this module speaks. Must match the host.
#[no_mangle]
pub extern "C" fn arcadia_abi_version() -> i32 {
    1
}

/// Allocate `len` bytes of guest memory for the host to write into. `vec![0u8; len]`
/// gives capacity exactly `len`, so `arcadia_dealloc` can reconstruct the `Vec` safely.
#[no_mangle]
pub extern "C" fn arcadia_alloc(len: i32) -> i32 {
    let buf = vec![0u8; len.max(0) as usize];
    let ptr = buf.as_ptr() as i32;
    std::mem::forget(buf);
    ptr
}

/// Free memory previously handed out by [`arcadia_alloc`].
#[no_mangle]
pub extern "C" fn arcadia_dealloc(ptr: i32, len: i32) {
    if ptr == 0 {
        return;
    }
    let len = len.max(0) as usize;
    unsafe {
        drop(Vec::from_raw_parts(ptr as *mut u8, len, len));
    }
}

/// Command entry point. Returns a packed `(ptr << 32) | len` pointing at a UTF-8 result
/// string in guest memory; the host copies it out then frees it via `arcadia_dealloc`.
#[no_mangle]
pub extern "C" fn arcadia_dispatch(
    verb_ptr: i32,
    verb_len: i32,
    args_ptr: i32,
    args_len: i32,
) -> i64 {
    let verb = read_str(verb_ptr, verb_len);
    let args_json = read_str(args_ptr, args_len);

    let result = match verb.as_str() {
        "greet" => {
            let who = first_json_string(&args_json).unwrap_or_else(|| "world".to_string());
            format!("Hello, {who}")
        }
        other => format!("hello-wasm: unknown verb '{other}'"),
    };
    pack(result)
}

/// Read a UTF-8 string the host wrote into guest memory.
fn read_str(ptr: i32, len: i32) -> String {
    if ptr == 0 || len <= 0 {
        return String::new();
    }
    let slice = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };
    String::from_utf8_lossy(slice).into_owned()
}

/// Move a string into guest memory and return its packed `(ptr << 32) | len`.
fn pack(s: String) -> i64 {
    let bytes = s.into_bytes();
    let len = bytes.len() as i64;
    let ptr = bytes.as_ptr() as i64;
    std::mem::forget(bytes);
    (ptr << 32) | len
}

/// Extract the first quoted string from a flat JSON array like `["World"]`. Tiny hand
/// parser — the real SDK will give modules `serde`-backed argument decoding.
fn first_json_string(json: &str) -> Option<String> {
    let start = json.find('"')? + 1;
    let rest = &json[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}
