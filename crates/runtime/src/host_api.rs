//! Host functions exposed to WASM contracts

use crate::gas_meter::{GetBalanceGasMeter, HashGasMeter, StoreGasMeter, StoreOperationType};
use crate::RuntimeContext;
use anyhow::Result;
use wasmlette_blockchain::Address;
use wasmtime::Linker;
use tracing::info;

/// Register all host functions with the linker
pub fn register_host_functions(linker: &mut Linker<RuntimeContext>) -> Result<()> {
    // Storage functions
    register_storage_functions(linker)?;

    // Balance functions
    register_balance_functions(linker)?;

    // Crypto functions
    register_crypto_functions(linker)?;

    // Context functions (caller info)
    register_context_functions(linker)?;

    Ok(())
}

fn register_storage_functions(linker: &mut Linker<RuntimeContext>) -> Result<()> {
    // env::get_storage(key_ptr: i32, key_len: i32) -> i32
    linker.func_wrap(
        "env",
        "get_storage",
        |mut caller: wasmtime::Caller<RuntimeContext>,
         key_ptr: i32,
         key_len: i32,
         output_ptr: i32,
         output_capacity: i32|
         -> i32 {
            // Get memory
            let memory = match caller.get_export("memory") {
                Some(wasmtime::Extern::Memory(mem)) => mem,
                _ => return -1, // Memory not found
            };

            // Read key from WASM memory
            let key = match crate::memory::MemoryHelper::read_bytes(
                &mut caller,
                &memory,
                key_ptr as u32,
                key_len as u32,
            ) {
                Ok(k) => k,
                Err(_) => return -1,
            };

            let value: Vec<u8>;
            {
                // Get state and contract address from context
                let context = caller.data_mut();
                let contract_address = context.contract_address;
                let state = context.state.borrow();

                // Query storage
                value = match state.get_storage(contract_address, key) {
                    Some(v) => v,
                    None => {
                        //TODO move to function
                        let gas_meter = StoreGasMeter::new();
                        let gas_cost = gas_meter
                            .operation_cost(key_len as u32, StoreOperationType::Get)
                            as u64;
                        info!("[TEST:get_storage (key no found)] Gas cost: {}", gas_cost);
                        info!(
                            "[TEST:get_storage (key no found)] Gas remaining: {}",
                            context.gas_remaining
                        );
                        if context.gas_remaining < gas_cost {
                            context.gas_remaining = context
                                .gas_remaining
                                .saturating_sub(gas_meter.fail_operation_cost as u64);
                            return -1;
                        }
                        context.gas_remaining -= gas_cost;
                        info!(
                            "[TEST:get_storage (key no found)] Gas remaining: {}",
                            context.gas_remaining
                        );

                        return 0;
                    }
                };
            }

            // Check if output buffer is large enough
            let value_len = value.len() as i32;
            {
                let gas_meter = StoreGasMeter::new();
                let data_len = key_len as u32 + value_len.max(output_capacity) as u32;
                let gas_cost = gas_meter.operation_cost(data_len, StoreOperationType::Get) as u64;

                info!("[TEST:get_storage] Gas cost: {}", gas_cost);
                let context = caller.data_mut();
                info!(
                    "[TEST:get_storage] Gas remaining: {}",
                    context.gas_remaining
                );
                if context.gas_remaining < gas_cost {
                    context.gas_remaining = context
                        .gas_remaining
                        .saturating_sub(gas_meter.fail_operation_cost as u64);
                    return -1;
                }
                context.gas_remaining -= gas_cost;
                info!(
                    "[TEST:get_storage] Gas remaining: {}",
                    context.gas_remaining
                );
            }

            if value_len > output_capacity {
                return -1;
            }

            // Write value to WASM memory
            if let Err(_) = crate::memory::MemoryHelper::write_bytes(
                &mut caller,
                &memory,
                output_ptr as u32,
                &value,
            ) {
                return -1;
            }
            // Return actual length written
            value_len
        },
    )?;

    // env::set_storage(key_ptr: i32, key_len: i32, val_ptr: i32, val_len: i32)
    linker.func_wrap(
        "env",
        "set_storage",
        |mut caller: wasmtime::Caller<RuntimeContext>,
         key_ptr: i32,
         key_len: i32,
         val_ptr: i32,
         val_len: i32| {
            // Gas fee
            {
                let gas_meter = StoreGasMeter::new();
                let data_len = key_len as u32 + val_len as u32;
                let gas_cost = gas_meter.operation_cost(data_len, StoreOperationType::Set) as u64;

                info!("[TEST:set_storage] Gas cost: {}", gas_cost);
                let context = caller.data_mut();
                info!(
                    "[TEST:set_storage] Gas remaining: {}",
                    context.gas_remaining
                );
                if context.gas_remaining < gas_cost {
                    context.gas_remaining = context.gas_remaining.saturating_sub(200);
                    return;
                }
                context.gas_remaining -= gas_cost;
                info!(
                    "[TEST:set_storage] Gas remaining: {}",
                    context.gas_remaining
                );
            }
            // Get memory
            let memory = match caller.get_export("memory") {
                Some(wasmtime::Extern::Memory(mem)) => mem,
                _ => return,
            };
            // Read key from WASM memory
            let key = match crate::memory::MemoryHelper::read_bytes(
                &mut caller,
                &memory,
                key_ptr as u32,
                key_len as u32,
            ) {
                Ok(k) => k,
                Err(_) => return,
            };
            info!("[TEST:set_storage] Key: {:?}", key);

            // Read value from WASM memory
            let value = match crate::memory::MemoryHelper::read_bytes(
                &mut caller,
                &memory,
                val_ptr as u32,
                val_len as u32,
            ) {
                Ok(k) => k,
                Err(_) => return,
            };
            info!("[TEST:set_storage] Value: {:?}", value);

            // Get state and contract address from context
            let context = caller.data();
            let contract_address = context.contract_address;
            let mut state = context.state.borrow_mut();

            // Write to storage
            state.set_storage(contract_address, key, value);

            // TODO: Consume gas based on value size
        },
    )?;

    Ok(())
}

fn register_balance_functions(linker: &mut Linker<RuntimeContext>) -> Result<()> {
    // env::get_balance(address_ptr: i32) -> i64
    linker.func_wrap(
        "env",
        "get_balance",
        |mut caller: wasmtime::Caller<RuntimeContext>, address_ptr: i32| -> i64 {
            // Gas fee
            {
                let gas_meter = GetBalanceGasMeter::new();
                let gas_cost = gas_meter.operation_cost() as u64;

                let context = caller.data_mut();
                if context.gas_remaining < gas_cost {
                    context.gas_remaining = context
                        .gas_remaining
                        .saturating_sub(gas_meter.fail_operation_cost as u64);
                    return -1;
                }
                context.gas_remaining -= gas_cost;
            }
            // Get memory
            let memory = match caller.get_export("memory") {
                Some(wasmtime::Extern::Memory(mem)) => mem,
                _ => return -1,
            };
            // Read addr from WASM memory
            let addr_bytes = match crate::memory::MemoryHelper::read_bytes(
                &mut caller,
                &memory,
                address_ptr as u32,
                Address::LENGTH as u32,
            ) {
                Ok(addr) => addr,
                Err(_) => return -1,
            };

            let address = wasmlette_blockchain::Address::from_slice(&addr_bytes);

            // Get state and contract address from context
            let context = caller.data();
            let state = context.state.borrow();

            state.get_balance(&address) as i64
        },
    )?;

    Ok(())
}

fn register_crypto_functions(linker: &mut Linker<RuntimeContext>) -> Result<()> {
    // env::hash_blake3(data_ptr: i32, data_len: i32, output_ptr: i32)
    linker.func_wrap(
        "env",
        "hash_blake3",
        |mut caller: wasmtime::Caller<RuntimeContext>,
         data_ptr: i32,
         data_len: i32,
         output_ptr: i32| {
            // Gas fee
            {
                let gas_meter = HashGasMeter::new();
                let gas_cost = gas_meter.operation_cost(data_len as u32) as u64;

                let context = caller.data_mut();
                if context.gas_remaining < gas_cost {
                    context.gas_remaining = context
                        .gas_remaining
                        .saturating_sub(gas_meter.fail_operation_cost as u64);
                    return;
                }
                context.gas_remaining -= gas_cost;
            }
            // Get memory
            let memory = match caller.get_export("memory") {
                Some(wasmtime::Extern::Memory(mem)) => mem,
                _ => return,
            };

            // Read data from WASM memory
            let data = match crate::memory::MemoryHelper::read_bytes(
                &mut caller,
                &memory,
                data_ptr as u32,
                data_len as u32,
            ) {
                Ok(data) => data,
                Err(_) => return,
            };

            // Hash data
            let hash: [u8; 32] = blake3::hash(&data).into();

            // Write hashed data to WASM memory
            let _ = crate::memory::MemoryHelper::write_bytes(
                &mut caller,
                &memory,
                output_ptr as u32,
                &hash,
            );
        },
    )?;

    Ok(())
}

fn register_context_functions(linker: &mut Linker<RuntimeContext>) -> Result<()> {
    // env::get_caller(output_ptr: i32)
    linker.func_wrap(
        "env",
        "get_caller",
        |mut caller: wasmtime::Caller<RuntimeContext>, output_ptr: i32| {
            // Get memory
            let memory = match caller.get_export("memory") {
                Some(wasmtime::Extern::Memory(mem)) => mem,
                _ => return,
            };

            // Get caller address from context (20 bytes) - copy to avoid borrow conflict
            let caller_address = {
                let context = caller.data();
                *context.caller_address.as_bytes()
            };

            info!(
                "[TEST:register_context_functions] caller address: {:?}",
                caller_address
            );

            // Write caller address to WASM memory
            let _ = crate::memory::MemoryHelper::write_bytes(
                &mut caller,
                &memory,
                output_ptr as u32,
                &caller_address,
            );
        },
    )?;

    Ok(())
}
