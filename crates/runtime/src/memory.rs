//! Runtime memory abstraction

use crate::RuntimeContext;
use anyhow::{bail, Result};
use wasmtime::{Caller, Memory};

/// Helper functions for reading/writing WASM memory
pub struct MemoryHelper;

impl MemoryHelper {
    /// Read bytes from WASM memory
    pub fn read_bytes(
        caller: &mut Caller<'_, RuntimeContext>,
        memory: &Memory,
        ptr: u32,
        len: u32,
    ) -> Result<Vec<u8>> {
        let data = memory.data(&caller);

        let start = ptr as usize;
        let end = start + len as usize;

        if end > data.len() {
            bail!(
                "Memory access out of bounds: ptr={}, len={}, memory_size={}",
                ptr,
                len,
                data.len()
            );
        }

        Ok(data[start..end].to_vec())
    }

    /// Write bytes to WASM memory
    pub fn write_bytes(
        caller: &mut Caller<'_, RuntimeContext>,
        memory: &Memory,
        ptr: u32,
        data: &[u8],
    ) -> Result<()> {
        let mem_data = memory.data_mut(&mut *caller);

        let start = ptr as usize;
        let end = start + data.len();

        if end > mem_data.len() {
            bail!(
                "Memory write out of bounds: ptr={}, len={}, memory_size={}",
                ptr,
                data.len(),
                mem_data.len()
            );
        }

        mem_data[start..end].copy_from_slice(data);
        Ok(())
    }

    /// Get the memory export from an instance
    pub fn get_memory(caller: &mut Caller<'_, RuntimeContext>, name: &str) -> Result<Memory> {
        caller
            .get_export(name)
            .and_then(|e| e.into_memory())
            .ok_or_else(|| anyhow::anyhow!("Memory export '{}' not found", name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasmlette_blockchain::{Address, State};
    use wasmtime::{Engine, Instance, Linker, Module, Store};

    /// Create test context and WASM instance
    fn create_test_setup() -> Result<(Store<RuntimeContext>, Instance, Memory)> {
        let engine = Engine::default();

        // Create minimal runtime context
        let state = Rc::new(RefCell::new(State::new()));
        let context = RuntimeContext {
            caller_address: Address::from_slice(&[1u8; Address::LENGTH]),
            contract_address: Address::from_slice(&[2u8; Address::LENGTH]),
            state,
            gas_remaining: 100_000,
        };

        let mut store = Store::new(&engine, context);

        // WAT module with memory and host function imports
        // Note: Imports must come before other definitions in WAT
        let wat = r#"
            (module
                ;; Host function imports for testing (must come first)
                (import "env" "test_read" (func $test_read (param i32 i32) (result i32)))
                (import "env" "test_write" (func $test_write (param i32 i32) (result i32)))

                ;; Memory export
                (memory (export "memory") 1)

                ;; Test function that calls host read
                (func (export "call_read") (param $ptr i32) (param $len i32) (result i32)
                    local.get $ptr
                    local.get $len
                    call $test_read
                )

                ;; Test function that calls host write
                (func (export "call_write") (param $ptr i32) (param $len i32) (result i32)
                    local.get $ptr
                    local.get $len
                    call $test_write
                )
            )
        "#;

        let module = Module::new(&engine, wat)?;
        let mut linker = Linker::new(&engine);

        // Link host functions that use MemoryHelper
        linker.func_wrap(
            "env",
            "test_read",
            |mut caller: Caller<'_, RuntimeContext>, ptr: i32, len: i32| -> Result<i32> {
                let memory = MemoryHelper::get_memory(&mut caller, "memory")?;
                let result =
                    MemoryHelper::read_bytes(&mut caller, &memory, ptr as u32, len as u32)?;
                Ok(result.len() as i32)
            },
        )?;

        linker.func_wrap(
            "env",
            "test_write",
            |mut caller: Caller<'_, RuntimeContext>, ptr: i32, len: i32| -> Result<i32> {
                let memory = MemoryHelper::get_memory(&mut caller, "memory")?;
                let test_data: Vec<u8> = vec![0xAB; len as usize];
                MemoryHelper::write_bytes(&mut caller, &memory, ptr as u32, &test_data)?;
                Ok(0)
            },
        )?;

        let instance = linker.instantiate(&mut store, &module)?;
        let memory = instance.get_memory(&mut store, "memory").unwrap();

        Ok((store, instance, memory))
    }

    #[test]
    fn test_memory_read_via_host_function() {
        let (mut store, instance, memory) = create_test_setup().unwrap();

        // Write test data directly to memory
        {
            let data = memory.data_mut(&mut store);
            data[100..105].copy_from_slice(b"hello");
        }

        // Call WASM function that calls our host function
        let call_read = instance
            .get_typed_func::<(i32, i32), i32>(&mut store, "call_read")
            .unwrap();

        let result = call_read.call(&mut store, (100, 5)).unwrap();
        assert_eq!(result, 5); // Should have read 5 bytes
    }

    #[test]
    fn test_memory_write_via_host_function() {
        let (mut store, instance, memory) = create_test_setup().unwrap();

        // Call WASM function that calls our host write function
        let call_write = instance
            .get_typed_func::<(i32, i32), i32>(&mut store, "call_write")
            .unwrap();

        let result = call_write.call(&mut store, (200, 10)).unwrap();
        assert_eq!(result, 0); // Success

        // Verify data was written
        let data = memory.data(&store);
        assert_eq!(&data[200..210], &[0xAB; 10]);
    }

    #[test]
    fn test_memory_read_out_of_bounds() {
        let (mut store, instance, _memory) = create_test_setup().unwrap();

        // Try to read past memory bounds (1 page = 65536 bytes)
        let call_read = instance
            .get_typed_func::<(i32, i32), i32>(&mut store, "call_read")
            .unwrap();

        // This should trap due to out of bounds access
        let result = call_read.call(&mut store, (65530, 100));
        assert!(result.is_err());
    }

    #[test]
    fn test_memory_write_out_of_bounds() {
        let (mut store, instance, _memory) = create_test_setup().unwrap();

        // Try to write past memory bounds
        let call_write = instance
            .get_typed_func::<(i32, i32), i32>(&mut store, "call_write")
            .unwrap();

        // This should trap
        let result = call_write.call(&mut store, (65530, 100));
        assert!(result.is_err());
    }

    #[test]
    fn test_memory_operations_at_boundary() {
        let (mut store, instance, memory) = create_test_setup().unwrap();

        // Write at the last 4 bytes (boundary)
        let call_write = instance
            .get_typed_func::<(i32, i32), i32>(&mut store, "call_write")
            .unwrap();

        let result = call_write.call(&mut store, (65532, 4)).unwrap();
        assert_eq!(result, 0);

        // Verify
        let data = memory.data(&store);
        assert_eq!(&data[65532..65536], &[0xAB; 4]);

        // Now read it back
        let call_read = instance
            .get_typed_func::<(i32, i32), i32>(&mut store, "call_read")
            .unwrap();

        let result = call_read.call(&mut store, (65532, 4)).unwrap();
        assert_eq!(result, 4); // Read 4 bytes successfully
    }

    #[test]
    fn test_memory_get_memory_success() {
        let (mut store, instance, _memory) = create_test_setup().unwrap();

        // Create a simple host function to test get_memory
        let engine = store.engine().clone();
        let wat = r#"
            (module
                (import "env" "test_get" (func $test_get (result i32)))
                (memory (export "memory") 1)
                (func (export "test") (result i32)
                    call $test_get
                )
            )
        "#;

        let module = Module::new(&engine, wat).unwrap();
        let mut linker = Linker::new(&engine);

        linker
            .func_wrap(
                "env",
                "test_get",
                |mut caller: Caller<'_, RuntimeContext>| -> Result<i32> {
                    let memory = MemoryHelper::get_memory(&mut caller, "memory")?;
                    let size = memory.data(&caller).len();
                    Ok(size as i32)
                },
            )
            .unwrap();

        let instance = linker.instantiate(&mut store, &module).unwrap();
        let test_func = instance
            .get_typed_func::<(), i32>(&mut store, "test")
            .unwrap();

        let result = test_func.call(&mut store, ()).unwrap();
        assert_eq!(result, 65536); // 1 page = 64KB
    }

    #[test]
    fn test_memory_get_memory_not_found() {
        let (mut store, _instance, _memory) = create_test_setup().unwrap();

        // Create a module without exported memory
        let engine = store.engine().clone();
        let wat = r#"
            (module
                (import "env" "test_get" (func $test_get (result i32)))
                (func (export "test") (result i32)
                    call $test_get
                )
            )
        "#;

        let module = Module::new(&engine, wat).unwrap();
        let mut linker = Linker::new(&engine);

        linker
            .func_wrap(
                "env",
                "test_get",
                |mut caller: Caller<'_, RuntimeContext>| -> Result<i32> {
                    // This should fail - no memory export
                    MemoryHelper::get_memory(&mut caller, "memory")?;
                    Ok(0)
                },
            )
            .unwrap();

        let instance = linker.instantiate(&mut store, &module).unwrap();
        let test_func = instance
            .get_typed_func::<(), i32>(&mut store, "test")
            .unwrap();

        // Should trap with "Memory export 'memory' not found"
        let result = test_func.call(&mut store, ());
        assert!(result.is_err());
    }

    #[test]
    fn test_memory_zero_length_operations() {
        let (mut store, instance, _memory) = create_test_setup().unwrap();

        // Read zero bytes
        let call_read = instance
            .get_typed_func::<(i32, i32), i32>(&mut store, "call_read")
            .unwrap();

        let result = call_read.call(&mut store, (0, 0)).unwrap();
        assert_eq!(result, 0); // Read 0 bytes

        // Write zero bytes
        let call_write = instance
            .get_typed_func::<(i32, i32), i32>(&mut store, "call_write")
            .unwrap();

        let result = call_write.call(&mut store, (0, 0)).unwrap();
        assert_eq!(result, 0); // Success
    }

    #[test]
    fn test_memory_large_read() {
        let (mut store, instance, _memory) = create_test_setup().unwrap();

        // Read a large chunk (10KB)
        let call_read = instance
            .get_typed_func::<(i32, i32), i32>(&mut store, "call_read")
            .unwrap();

        let result = call_read.call(&mut store, (0, 10240)).unwrap();
        assert_eq!(result, 10240); // Successfully read 10KB
    }
}
