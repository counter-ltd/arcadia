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

/// Per-store host state. MVP carries only the module id (for log prefixing); permission
/// gating adds `granted_permissions` here post-MVP.
pub struct HostState {
    pub module_id: String,
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
    /// optional `arcadia_init` export.
    pub fn instantiate(bytes: &[u8], module_id: &str) -> Result<LoadedModule, String> {
        let engine = Engine::default();
        let module = Module::new(&engine, bytes).map_err(|e| format!("invalid wasm: {e}"))?;
        let mut store = Store::new(
            &engine,
            HostState {
                module_id: module_id.to_string(),
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

/// Register the `arcadia` host import module. MVP: `host_log` only.
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
    Ok(())
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
