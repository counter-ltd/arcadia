//! One loaded WASM module: its `wasmi` store, instance, and the host/guest ABI.
//!
//! ABI (core wasm, no Component Model):
//! - Module **exports**: `memory`, `arcadia_alloc(i32)->i32`, `arcadia_dealloc(i32,i32)`,
//!   `arcadia_abi_version()->i32`, `arcadia_init()` (optional),
//!   `arcadia_dispatch(verb_ptr,verb_len,args_ptr,args_len)->i64` (packed `(ptr<<32)|len`),
//!   `arcadia_shutdown()` (optional).
//! - Module **imports** (module `"arcadia"`): `host_log(level,ptr,len)`.

use wasmi::{Caller, Engine, Extern, Instance, Linker, Memory, Module, Store, TypedFunc};

/// ABI version this host speaks. A module declaring a different version is rejected.
pub const HOST_ABI_VERSION: i32 = 1;

/// Per-store host state, available to every host function via `Caller::data`.
pub struct HostState {
    pub module_id: String,
    /// Permissions the module declared *and* that are effectively granted to
    /// `wasm:<module_id>`. Resolved once at instantiation. Host functions may consult
    /// this; command dispatch through `host_execute_command` is additionally gated by
    /// `execute_command`'s own per-command permission check.
    pub granted_permissions: Vec<String>,
}

/// A single instantiated WASM module. Not `Sync` — callers wrap it in a `Mutex`.
pub struct LoadedModule {
    store: Store<HostState>,
    memory: Memory,
    alloc: TypedFunc<i32, i32>,
    dealloc: TypedFunc<(i32, i32), ()>,
    dispatch: TypedFunc<(i32, i32, i32, i32), i64>,
}

impl LoadedModule {
    /// Instantiate a module from its wasm bytes. Verifies the ABI version and runs the
    /// optional `arcadia_init` export. `required_permissions` is the manifest's declared
    /// set — each one effectively granted to `wasm:<module_id>` is recorded in `HostState`.
    pub fn instantiate(
        bytes: &[u8],
        module_id: &str,
        required_permissions: &[String],
    ) -> Result<LoadedModule, String> {
        let engine = Engine::default();
        let module = Module::new(&engine, bytes).map_err(|e| format!("invalid wasm: {e}"))?;
        let granted = resolve_granted_permissions(module_id, required_permissions);
        let mut store = Store::new(
            &engine,
            HostState {
                module_id: module_id.to_string(),
                granted_permissions: granted,
            },
        );
        let mut linker: Linker<HostState> = Linker::new(&engine);
        register_host_functions(&mut linker)?;

        let instance = linker
            .instantiate_and_start(&mut store, &module)
            .map_err(|e| format!("instantiate failed: {e}"))?;

        let memory = instance
            .get_memory(&store, "memory")
            .ok_or_else(|| "module does not export 'memory'".to_string())?;
        let alloc = typed_func(&instance, &store, "arcadia_alloc")?;
        let dealloc = typed_func(&instance, &store, "arcadia_dealloc")?;
        let dispatch = typed_func(&instance, &store, "arcadia_dispatch")?;
        let abi: TypedFunc<(), i32> = typed_func(&instance, &store, "arcadia_abi_version")?;

        let mut loaded = LoadedModule {
            store,
            memory,
            alloc,
            dealloc,
            dispatch,
        };

        let declared = abi
            .call(&mut loaded.store, ())
            .map_err(|e| format!("arcadia_abi_version trapped: {e}"))?;
        if declared != HOST_ABI_VERSION {
            return Err(format!(
                "module ABI version {declared} != host ABI version {HOST_ABI_VERSION}"
            ));
        }

        // Optional init export.
        if let Ok(init) = instance.get_typed_func::<(), ()>(&loaded.store, "arcadia_init") {
            init.call(&mut loaded.store, ())
                .map_err(|e| format!("arcadia_init trapped: {e}"))?;
        }
        Ok(loaded)
    }

    /// Dispatch one command verb to the module. `args` is encoded as a JSON array string.
    pub fn dispatch(&mut self, verb: &str, args: &[String]) -> Result<String, String> {
        let args_json = serde_json::to_string(args).unwrap_or_else(|_| "[]".to_string());
        let (verb_ptr, verb_len) = self.write_guest(verb.as_bytes())?;
        let (args_ptr, args_len) = self.write_guest(args_json.as_bytes())?;

        let packed = self
            .dispatch
            .call(
                &mut self.store,
                (verb_ptr, verb_len, args_ptr, args_len),
            )
            .map_err(|e| format!("arcadia_dispatch trapped: {e}"))?;

        // Inputs were copied into the guest; free them.
        let _ = self.dealloc.call(&mut self.store, (verb_ptr, verb_len));
        let _ = self.dealloc.call(&mut self.store, (args_ptr, args_len));

        let res_ptr = (packed >> 32) as i32;
        let res_len = (packed & 0xffff_ffff) as i32;
        let result = self.read_guest(res_ptr, res_len)?;
        let _ = self.dealloc.call(&mut self.store, (res_ptr, res_len));
        Ok(result)
    }

    /// Copy bytes into guest linear memory via `arcadia_alloc`. Returns `(ptr, len)`.
    fn write_guest(&mut self, bytes: &[u8]) -> Result<(i32, i32), String> {
        let len = bytes.len() as i32;
        let ptr = self
            .alloc
            .call(&mut self.store, len)
            .map_err(|e| format!("arcadia_alloc trapped: {e}"))?;
        self.memory
            .write(&mut self.store, ptr as usize, bytes)
            .map_err(|e| format!("guest memory write failed: {e}"))?;
        Ok((ptr, len))
    }

    /// Read a UTF-8 string from guest linear memory.
    fn read_guest(&self, ptr: i32, len: i32) -> Result<String, String> {
        if len < 0 || ptr < 0 {
            return Err("module returned a negative pointer/length".to_string());
        }
        let mut buf = vec![0u8; len as usize];
        self.memory
            .read(&self.store, ptr as usize, &mut buf)
            .map_err(|e| format!("guest memory read failed: {e}"))?;
        String::from_utf8(buf).map_err(|_| "module returned non-UTF-8 output".to_string())
    }
}

fn typed_func<P, R>(
    instance: &Instance,
    store: &Store<HostState>,
    name: &str,
) -> Result<TypedFunc<P, R>, String>
where
    P: wasmi::WasmParams,
    R: wasmi::WasmResults,
{
    instance
        .get_typed_func::<P, R>(store, name)
        .map_err(|e| format!("module is missing required export '{name}': {e}"))
}

/// Register the `arcadia` host import module: `host_log` and `host_execute_command`.
fn register_host_functions(linker: &mut Linker<HostState>) -> Result<(), String> {
    linker
        .func_wrap(
            "arcadia",
            "host_log",
            |caller: Caller<'_, HostState>, level: i32, ptr: i32, len: i32| {
                let msg = read_caller_str(&caller, ptr, len);
                let id = &caller.data().module_id;
                eprintln!("[wasm:{id}] (level {level}) {msg}");
            },
        )
        .map_err(|e| format!("failed to register host_log: {e}"))?;

    linker
        .func_wrap(
            "arcadia",
            "host_has_permission",
            |caller: Caller<'_, HostState>, ptr: i32, len: i32| -> i32 {
                let perm = read_caller_str(&caller, ptr, len);
                i32::from(
                    caller
                        .data()
                        .granted_permissions
                        .iter()
                        .any(|p| *p == perm),
                )
            },
        )
        .map_err(|e| format!("failed to register host_has_permission: {e}"))?;

    linker
        .func_wrap(
            "arcadia",
            "host_execute_command",
            |mut caller: Caller<'_, HostState>,
             tok_ptr: i32,
             tok_len: i32,
             args_ptr: i32,
             args_len: i32|
             -> i64 {
                let token = read_caller_str(&caller, tok_ptr, tok_len);
                let args_json = read_caller_str(&caller, args_ptr, args_len);
                let module_id = caller.data().module_id.clone();
                let result = host_execute_command(&module_id, &token, &args_json);
                write_caller_str(&mut caller, &result)
            },
        )
        .map_err(|e| format!("failed to register host_execute_command: {e}"))?;
    Ok(())
}

/// Resolve which of a module's declared permissions are effectively granted to
/// `wasm:<module_id>`. Done once at instantiation.
fn resolve_granted_permissions(module_id: &str, declared: &[String]) -> Vec<String> {
    use arcadia_core::config::permissions::{PermissionSubject, PermissionsConfig};
    use arcadia_core::config::ConfigFile;

    let Ok(cfg) = PermissionsConfig::load_or_create() else {
        return Vec::new();
    };
    let subject = PermissionSubject::wasm(module_id.to_string());
    declared
        .iter()
        .filter(|p| cfg.effective_allowed(&subject, p))
        .cloned()
        .collect()
}

/// Re-enter Arcadia's command dispatch on behalf of a WASM module. `args_json` is the
/// JSON string array the module passed; the result is the command output or an error
/// string. The `invoking_wasm_module` context lets nested permission checks accept
/// grants on `wasm:<module_id>`.
fn host_execute_command(module_id: &str, token: &str, args_json: &str) -> String {
    use arcadia_core::modules::{execute_command, ExecutionContext};

    let args: Vec<String> = serde_json::from_str(args_json).unwrap_or_default();
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let ctx = ExecutionContext {
        invoking_wasm_module: Some(module_id.to_string()),
        ..ExecutionContext::default()
    };
    match execute_command(token, &arg_refs, &ctx) {
        Ok(Some(s)) => s,
        Ok(None) => format!("error: unknown command '{token}'"),
        Err(e) => format!("error: {e}"),
    }
}

/// Allocate guest memory via `arcadia_alloc`, copy `s` into it, and return the packed
/// `(ptr << 32) | len` the guest unpacks. Returns 0 on any failure.
fn write_caller_str(caller: &mut Caller<'_, HostState>, s: &str) -> i64 {
    let bytes = s.as_bytes();
    let len = bytes.len() as i32;
    let Some(Extern::Func(alloc)) = caller.get_export("arcadia_alloc") else {
        return 0;
    };
    let Ok(alloc) = alloc.typed::<i32, i32>(&*caller) else {
        return 0;
    };
    let Ok(ptr) = alloc.call(&mut *caller, len) else {
        return 0;
    };
    let Some(Extern::Memory(mem)) = caller.get_export("memory") else {
        return 0;
    };
    if mem.write(&mut *caller, ptr as usize, bytes).is_err() {
        return 0;
    }
    ((ptr as i64) << 32) | (len as i64)
}

/// Read a guest string during a host-function call.
fn read_caller_str(caller: &Caller<'_, HostState>, ptr: i32, len: i32) -> String {
    if ptr < 0 || len < 0 {
        return String::new();
    }
    let Some(Extern::Memory(mem)) = caller.get_export("memory") else {
        return String::new();
    };
    let mut buf = vec![0u8; len as usize];
    if mem.read(caller, ptr as usize, &mut buf).is_err() {
        return String::new();
    }
    String::from_utf8_lossy(&buf).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal valid module implementing the ABI: a bump allocator and a `dispatch`
    /// that writes "ok" at offset 8 and returns the packed pointer.
    const OK_MODULE_WAT: &str = r#"
        (module
          (memory (export "memory") 1)
          (global $bump (mut i32) (i32.const 1024))
          (func (export "arcadia_abi_version") (result i32) (i32.const 1))
          (func (export "arcadia_alloc") (param $len i32) (result i32)
            (local $p i32)
            (local.set $p (global.get $bump))
            (global.set $bump (i32.add (global.get $bump) (local.get $len)))
            (local.get $p))
          (func (export "arcadia_dealloc") (param i32 i32))
          (func (export "arcadia_dispatch") (param i32 i32 i32 i32) (result i64)
            (i32.store8 (i32.const 8) (i32.const 111))
            (i32.store8 (i32.const 9) (i32.const 107))
            (i64.or (i64.shl (i64.const 8) (i64.const 32)) (i64.const 2))))
    "#;

    /// Same shape but declares ABI version 99 — must be rejected.
    const BAD_ABI_WAT: &str = r#"
        (module
          (memory (export "memory") 1)
          (func (export "arcadia_abi_version") (result i32) (i32.const 99))
          (func (export "arcadia_alloc") (param i32) (result i32) (i32.const 0))
          (func (export "arcadia_dealloc") (param i32 i32))
          (func (export "arcadia_dispatch") (param i32 i32 i32 i32) (result i64)
            (i64.const 0)))
    "#;

    #[test]
    fn instantiate_and_dispatch_round_trip() {
        let wasm = wat::parse_str(OK_MODULE_WAT).expect("WAT must compile");
        let mut module = match LoadedModule::instantiate(&wasm, "test-mod", &[]) {
            Ok(m) => m,
            Err(e) => panic!("module must instantiate: {e}"),
        };
        let out = module
            .dispatch("greet", &["World".to_string()])
            .expect("dispatch must succeed");
        assert_eq!(out, "ok");
    }

    #[test]
    fn rejects_non_wasm_bytes() {
        assert!(LoadedModule::instantiate(b"not a wasm module", "x", &[]).is_err());
    }

    #[test]
    fn rejects_abi_version_mismatch() {
        let wasm = wat::parse_str(BAD_ABI_WAT).expect("WAT must compile");
        match LoadedModule::instantiate(&wasm, "bad", &[]) {
            Ok(_) => panic!("ABI mismatch must be rejected"),
            Err(e) => assert!(e.contains("ABI version"), "unexpected error: {e}"),
        }
    }

    #[test]
    fn rejects_module_missing_required_export() {
        // No `arcadia_dispatch` export.
        let wat = r#"(module (memory (export "memory") 1)
            (func (export "arcadia_abi_version") (result i32) (i32.const 1))
            (func (export "arcadia_alloc") (param i32) (result i32) (i32.const 0))
            (func (export "arcadia_dealloc") (param i32 i32)))"#;
        let wasm = wat::parse_str(wat).expect("WAT must compile");
        assert!(LoadedModule::instantiate(&wasm, "incomplete", &[]).is_err());
    }
}
