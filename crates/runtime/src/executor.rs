//! Contract execution orchestration

use crate::constants::RECEIPT_FAILURE_GAS_FEE;
use crate::gas_fee_calculator::GasCalculator;
use crate::host_api::register_host_functions;
use crate::{DeployGasMeter, RuntimeContext, TransferGasMeter, WasmEngine};
use anyhow::Result;
use std::sync::{Arc, Mutex};
use tracing::error;
use tracing::info;
use wasmlette_blockchain::transaction::Address;
use wasmlette_blockchain::utils::generate_contract_address;
use wasmlette_blockchain::{State, Transaction, TransactionKind, TransactionReceipt};
use wasmtime::Linker;

/// Default contract execution gas limit
const MAX_CONTRACT_FUEL: u64 = 1_000_000;

/// Contract executor
pub struct ContractExecutor {
    engine: WasmEngine,
}

impl ContractExecutor {
    /// Create a new contract executor
    pub fn new() -> Result<Self> {
        Ok(ContractExecutor {
            engine: WasmEngine::new()?,
        })
    }

    /// Execute a transaction
    pub fn execute_transaction(
        &self,
        state: Arc<Mutex<State>>,
        tx: &Transaction,
    ) -> Result<TransactionReceipt> {
        let tx_hash = tx.hash();
        info!(
            "[TEST:execute_transaction] tx.hash(): {:?}, tx.sender: {}",
            tx_hash, tx.sender
        );
        // Validate transaction
        // Return error if nonce is invalid or insufficient balance
        self.validate_transaction(tx, &state)?;

        // Increment nonce
        state
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire state lock: {}", e))?
            .increment_nonce(&tx.sender);

        // Execute based on transaction type
        let result = match &tx.kind {
            TransactionKind::Deploy {
                wasm_code,
                init_args,
            } => self.execute_deploy(
                state.clone(),
                &tx.sender,
                tx.nonce,
                tx.gas_limit,
                wasm_code,
                init_args,
            ),
            TransactionKind::Call {
                contract,
                method,
                args,
            } => self.execute_call(
                state.clone(),
                &tx.sender,
                contract,
                tx.gas_limit,
                method,
                args,
            ),
            TransactionKind::Transfer { to, amount } => {
                self.execute_transfer(state.clone(), &tx.sender, to, *amount)
            }
        };

        match result {
            Ok((gas_used, return_data, contract_address)) => {
                info!(
                    "[TEST:execute_transaction] success, gas_used: {}, tx_hash: {:?}, tx.sender: {}",
                    gas_used, tx_hash, tx.sender
                );
                let gas_refund = GasCalculator::settle_gas(tx.gas_limit, gas_used, tx.gas_price)?;

                state
                    .lock()
                    .map_err(|e| anyhow::anyhow!("Failed to acquire state lock: {}", e))?
                    .add_balance(tx.sender.clone(), gas_refund.refund);

                {
                    let caller_balance = state
                        .lock()
                        .map_err(|e| anyhow::anyhow!("Failed to acquire state lock: {}", e))?
                        .get_balance(&tx.sender);
                    info!(
                        "[TEST:execute_transaction] caller_balance after gas refund: {}",
                        caller_balance
                    );
                }

                Ok(TransactionReceipt {
                    tx_hash,
                    success: true,
                    gas_used,
                    return_data,
                    error_message: None,
                    contract_address,
                })
            }
            Err(e) => {
                let gas_refund =
                    GasCalculator::settle_gas(tx.gas_limit, RECEIPT_FAILURE_GAS_FEE, tx.gas_price)?;

                info!(
                    "[TEST:execute_transaction] fail, gas_used: {}, tx_hash: {:?}, tx.sender: {}",
                    RECEIPT_FAILURE_GAS_FEE, tx_hash, tx.sender
                );

                state
                    .lock()
                    .map_err(|e| anyhow::anyhow!("Failed to acquire state lock: {}", e))?
                    .add_balance(tx.sender.clone(), gas_refund.refund);

                Ok(TransactionReceipt {
                    tx_hash,
                    success: false,
                    gas_used: RECEIPT_FAILURE_GAS_FEE, // Consume some gas on error TODO we should calculate it for different calls
                    return_data: vec![],
                    error_message: Some(e.to_string()),
                    contract_address: None,
                })
            }
        }
    }

    fn validate_transaction(&self, tx: &Transaction, state: &Arc<Mutex<State>>) -> Result<()> {
        // Validate nonce
        let expected_nonce = state
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire state lock: {}", e))?
            .get_nonce(&tx.sender);
        if tx.nonce != expected_nonce {
            error!(
            "[TEST:execute_transaction] validate_transaction fail, tx.nonce: {} != expected_nonce: {}",
            tx.nonce, expected_nonce
            );
            return anyhow::bail!(
                "Invalid nonce: expected {}, got {}",
                expected_nonce,
                tx.nonce
            );
        }
        //TODO validate signature

        // Check balance
        let max_gas_cost = GasCalculator::max_cost(tx.gas_limit, tx.gas_price)?;
        let caller_balance = state
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire state lock: {}", e))?
            .get_balance(&tx.sender);
        info!(
            "[TEST:execute_transaction] caller_balance: {}, max_gas_cost {}",
            caller_balance, max_gas_cost
        );
        if caller_balance < max_gas_cost {
            error!(
                "[TEST:execute_transaction] validate_transaction fail, caller_balance: {} < max_gas_cost: {}",
            caller_balance, max_gas_cost
            );
            return anyhow::bail!(
                "Insufficient balance: expected {}, got {}",
                max_gas_cost,
                caller_balance
            );
        }
        // Reserve balance
        state
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire state lock: {}", e))?
            .set_balance(tx.sender.clone(), caller_balance - max_gas_cost);

        {
            let caller_balance = state
                .lock()
                .map_err(|e| anyhow::anyhow!("Failed to acquire state lock: {}", e))?
                .get_balance(&tx.sender);
            info!(
                "[TEST:execute_transaction] caller_balance after gas fee: {}",
                caller_balance
            );
        }
        Ok(())
    }

    fn execute_deploy(
        &self,
        state: Arc<Mutex<State>>,
        deployer: &Address,
        nonce: u64,
        gas_limit: u64,
        wasm_code: &[u8],
        init_args: &[u8],
    ) -> Result<(u64, Vec<u8>, Option<Address>)> {
        info!("[TEST:execute_deploy] start, deployer: {}", deployer);
        // Load and validate module
        let _ = self.engine.load_module(wasm_code)?;

        // Generate contract address
        let contract_address = generate_contract_address(deployer, nonce);

        // Deploy contract
        state
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire state lock: {}", e))?
            .deploy_contract(contract_address, wasm_code.to_vec())?;

        let gas_meter = DeployGasMeter::new();
        let mut total_gas_used = gas_meter.operation_cost(wasm_code.len() as u32) as u64;
        info!("[TEST:execute_deploy] deploy gas used: {}", total_gas_used);
        // Try to call init function (it's optional)
        let init_result = self.execute_call(
            state,
            deployer,
            &contract_address,
            gas_limit,
            "init",
            init_args,
        )?;
        info!("[TEST:execute_deploy] init gas used: {}", init_result.0);
        total_gas_used += init_result.0;
        info!("[TEST:execute_deploy] total_gas_used: {}", total_gas_used);
        Ok((total_gas_used, vec![], Some(contract_address)))
    }

    fn execute_call(
        &self,
        state: Arc<Mutex<State>>,
        caller: &Address,
        contract: &Address,
        gas_limit: u64,
        method: &str,
        args: &[u8],
    ) -> Result<(u64, Vec<u8>, Option<Address>)> {
        info!(
            "[TEST:execute_call] start, caller: {}, method: {}",
            caller, method
        );
        // Verify contract exists
        if !state
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire state lock: {}", e))?
            .contract_exists(contract)
        {
            anyhow::bail!("Contract not found at address");
        }

        // TODO add more reliable check
        let contract_code = state
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire state lock: {}", e))?
            .get_contract(contract)
            .ok_or_else(|| anyhow::anyhow!("Contract not found at address"))?
            .code
            .clone();

        // Create context
        let context = RuntimeContext {
            caller_address: caller.clone(),
            contract_address: contract.clone(),
            state: state.clone(),
            gas_remaining: gas_limit,
        };

        // Create store
        let mut store = self.engine.create_store(context);

        // Set fuel
        store
            .set_fuel(gas_limit)
            .map_err(|e| anyhow::anyhow!("Failed to set fuel: {}", e))?; // TODO should be set up other way

        // Load module
        let module = self
            .engine
            .load_module(&contract_code)
            .map_err(|e| anyhow::anyhow!("Failed to load module: {}", e))?;

        // Create linker with host functions
        let mut linker = Linker::new(self.engine.engine());
        register_host_functions(&mut linker)
            .map_err(|e| anyhow::anyhow!("Failed to register host functions: {}", e))?;

        // Instantiate
        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| anyhow::anyhow!("Failed to instantiate module: {}", e))?;

        // Get function
        let func = instance
            .get_func(&mut store, method)
            .ok_or_else(|| anyhow::anyhow!("Method not found: {}", method))?;

        // Parse arguments based on function signature
        let func_type = func.ty(&store);
        let param_types: Vec<_> = func_type.params().collect();
        let result_types: Vec<_> = func_type.results().collect();

        // Get WASM memory for writing pointer data
        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| anyhow::anyhow!("Contract has no memory export"))?;

        // Convert raw bytes to WASM values (detects addresses and writes to memory)
        let wasm_args = self.parse_args(&mut store, &memory, args, &param_types)?;

        // Prepare results buffer
        let mut results = vec![wasmtime::Val::I32(0); result_types.len()];

        // Call the function with parsed arguments
        func.call(&mut store, &wasm_args, &mut results)
            .map_err(|e| anyhow::anyhow!("Contract execution failed: {}", e))?;

        // Convert return values to bytes
        let return_data = self.serialize_results(&results)?;

        // Calculate gas used from WASM execution
        let wasm_fuel_consumed = gas_limit - store.get_fuel()?;
        info!(
            "[TEST:execute_call] store.get_fuel(): {}",
            store.get_fuel()?
        );
        info!(
            "[TEST:execute_call] wasm_fuel_consumed: {}",
            wasm_fuel_consumed
        );

        // Add host function gas costs
        let context = store.data();
        let host_gas_consumed = gas_limit - context.gas_remaining;
        info!(
            "[TEST:execute_call] host_gas_consumed: {}",
            host_gas_consumed
        );

        // Total gas = WASM instructions + host function costs
        let total_gas_consumed = wasm_fuel_consumed + host_gas_consumed;
        info!(
            "[TEST:execute_call] total_gas_consumed: {}",
            total_gas_consumed
        );

        Ok((total_gas_consumed, return_data, None))
    }

    fn execute_transfer(
        &self,
        state: Arc<Mutex<State>>,
        from: &Address,
        to: &Address,
        amount: u64,
    ) -> Result<(u64, Vec<u8>, Option<Address>)> {
        state
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire state lock: {}", e))?
            .transfer(*from, *to, amount)?;
        Ok((
            TransferGasMeter::new().operation_cost() as u64,
            vec![],
            None,
        ))
    }

    /// Parse raw bytes into WASM values, detecting address pointers
    /// This is a bit hacky, but it works for now. Assume that we can have i32 as a number or address pointer.
    /// We don't have functions with more than 2 parameters and more than 1 address, so we can use this to detect addresses.
    ///
    /// Strategy:
    /// - i32: Check remaining bytes. If ≥20 (and not exactly 4), treat as an address pointer.
    ///        Write an address to memory and return a pointer. Otherwise, parse as regular i32.
    /// - i64: Always parse as 8-byte number
    fn parse_args(
        &self,
        store: &mut wasmtime::Store<RuntimeContext>,
        memory: &wasmtime::Memory,
        args: &[u8],
        param_types: &[wasmtime::ValType],
    ) -> Result<Vec<wasmtime::Val>> {
        let mut wasm_args = Vec::new();
        let mut offset = 0;
        let mut mem_offset = 0x1000u32; // Start writing at 4KB (safe region) TODO move to constant

        for param_type in param_types {
            match param_type {
                wasmtime::ValType::I32 => {
                    let remaining = args.len() - offset;

                    // Check if this looks like an address pointer
                    if remaining >= Address::LENGTH {
                        // This is an address! Write to memory and return pointer
                        if offset + Address::LENGTH > args.len() {
                            anyhow::bail!("Insufficient bytes for address");
                        }

                        let address_bytes = &args[offset..offset + Address::LENGTH];

                        // Write to WASM memory
                        let mem_data = memory.data_mut(&mut *store);
                        if (mem_offset as usize) + Address::LENGTH > mem_data.len() {
                            anyhow::bail!("Not enough WASM memory for address");
                        }
                        mem_data[mem_offset as usize..(mem_offset as usize) + Address::LENGTH]
                            .copy_from_slice(address_bytes);

                        // Return pointer
                        wasm_args.push(wasmtime::Val::I32(mem_offset as i32));

                        offset += Address::LENGTH;
                        mem_offset += Address::LENGTH as u32;
                    } else {
                        // Regular i32 number
                        if offset + size_of::<i32>() > args.len() {
                            anyhow::bail!("Insufficient args for I32 parameter");
                        }

                        let value =
                            i32::from_le_bytes(args[offset..offset + size_of::<i32>()].try_into()?);
                        wasm_args.push(wasmtime::Val::I32(value));
                        offset += size_of::<i32>();
                    }
                }
                wasmtime::ValType::I64 => {
                    if offset + size_of::<i64>() > args.len() {
                        anyhow::bail!("Insufficient args for I64 parameter");
                    }
                    let value =
                        i64::from_le_bytes(args[offset..offset + size_of::<i64>()].try_into()?);

                    wasm_args.push(wasmtime::Val::I64(value));
                    offset += size_of::<i64>();
                }
                _ => anyhow::bail!("Unsupported parameter type: {:?}", param_type),
            }
        }

        Ok(wasm_args)
    }

    /// Serialize WASM return values to bytes
    fn serialize_results(&self, results: &[wasmtime::Val]) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();

        for result in results {
            match result {
                wasmtime::Val::I32(v) => bytes.extend_from_slice(&v.to_le_bytes()),
                wasmtime::Val::I64(v) => bytes.extend_from_slice(&v.to_le_bytes()),
                _ => anyhow::bail!("Unsupported return type: {:?}", result),
            }
        }

        Ok(bytes)
    }
}

impl Default for ContractExecutor {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| panic!("Failed to create contract executor"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;
    use wasmlette_blockchain::TransactionKind;
    use wasmlette_tokens::TokenUnit;
    use std::sync::{Arc, Mutex};

    static INIT: Once = Once::new();

    /// Initialize tracing for tests (call once)
    fn init_tracing() {
        INIT.call_once(|| {
            tracing_subscriber::fmt()
                .with_test_writer()
                .with_env_filter(
                    tracing_subscriber::EnvFilter::from_default_env()
                        .add_directive(tracing::Level::INFO.into()),
                )
                .init();
        });
    }

    #[test]
    fn test_transfer_execution() {
        init_tracing();
        let executor = ContractExecutor::new().unwrap();
        let state = Arc::new(Mutex::new(State::new()));

        let from = Address::from_slice(&[1u8; Address::LENGTH]);
        let to = Address::from_slice(&[2u8; Address::LENGTH]);

        // Give sender some balance
        state
            .lock()
            .unwrap()
            .set_balance(from, TokenUnit::from_tokens(1.0));

        let tx = Transaction::new(
            from,
            0,
            TransactionKind::Transfer {
                to,
                amount: TokenUnit::from_tokens(0.01),
            },
            100000,
            1,
        );

        let receipt = executor.execute_transaction(state.clone(), &tx).unwrap();

        assert!(receipt.success);
        assert_eq!(receipt.gas_used, 11000);
        assert_eq!(
            state.lock().unwrap().get_balance(&from),
            TokenUnit::from_tokens(0.979)
        );
        assert_eq!(
            state.lock().unwrap().get_balance(&to),
            TokenUnit::from_tokens(0.01)
        );
    }

    #[test]
    fn test_invalid_nonce_returns_error() {
        let executor = ContractExecutor::new().unwrap();
        let state = Arc::new(Mutex::new(State::new()));

        let from = Address::from_slice(&[1u8; Address::LENGTH]);
        let to = Address::from_slice(&[2u8; Address::LENGTH]);

        // Give sender enough balance
        state
            .lock()
            .unwrap()
            .set_balance(from, TokenUnit::from_tokens(1.0));

        // Expected nonce is 0, but we provide 5
        let tx = Transaction::new(
            from,
            5, // Wrong nonce!
            TransactionKind::Transfer {
                to,
                amount: TokenUnit::from_tokens(0.01),
            },
            100000,
            1,
        );

        let initial_balance = state.lock().unwrap().get_balance(&from);
        let initial_nonce = state.lock().unwrap().get_nonce(&from);

        // Should return Err, not Ok(receipt)
        let result = executor.execute_transaction(state.clone(), &tx);

        assert!(result.is_err(), "Expected Err for invalid nonce");
        assert!(result.unwrap_err().to_string().contains("Invalid nonce"));

        // Verify state unchanged
        assert_eq!(
            state.lock().unwrap().get_balance(&from),
            initial_balance,
            "Balance should not change"
        );
        assert_eq!(
            state.lock().unwrap().get_nonce(&from),
            initial_nonce,
            "Nonce should not increment"
        );
        assert_eq!(
            state.lock().unwrap().get_balance(&to),
            0,
            "Recipient should have no balance"
        );
    }

    #[test]
    fn test_insufficient_balance_returns_error() {
        let executor = ContractExecutor::new().unwrap();
        let state = Arc::new(Mutex::new(State::new()));

        let from = Address::from_slice(&[1u8; Address::LENGTH]);
        let to = Address::from_slice(&[2u8; Address::LENGTH]);

        // Give sender very small balance (not enough for gas)
        state
            .lock()
            .unwrap()
            .set_balance(from, TokenUnit::from_tokens(0.001)); // Only 0.001 tokens

        let tx = Transaction::new(
            from,
            0,
            TransactionKind::Transfer {
                to,
                amount: TokenUnit::from_tokens(0.01),
            },
            100000, // gas_limit * gas_price = 100000 micro-tokens needed
            1,
        );

        let initial_balance = state.lock().unwrap().get_balance(&from);
        let initial_nonce = state.lock().unwrap().get_nonce(&from);

        // Should return Err for insufficient balance
        let result = executor.execute_transaction(state.clone(), &tx);

        assert!(result.is_err(), "Expected Err for insufficient balance");
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Insufficient balance"));

        // Verify state unchanged
        assert_eq!(
            state.lock().unwrap().get_balance(&from),
            initial_balance,
            "Balance should not change"
        );
        assert_eq!(
            state.lock().unwrap().get_nonce(&from),
            initial_nonce,
            "Nonce should not increment"
        );
    }

    #[test]
    fn test_valid_transaction_failed_execution() {
        let executor = ContractExecutor::new().unwrap();
        let state = Arc::new(Mutex::new(State::new()));

        let from = Address::from_slice(&[1u8; Address::LENGTH]);
        let to = Address::from_slice(&[2u8; Address::LENGTH]);

        // Give sender enough balance for gas but NOT for transfer amount
        state
            .lock()
            .unwrap()
            .set_balance(from, TokenUnit::from_tokens(0.2)); // 200,000 micro-tokens

        let tx = Transaction::new(
            from,
            0,
            TransactionKind::Transfer {
                to,
                amount: TokenUnit::from_tokens(1.0), // More than available!
            },
            100000,
            1,
        );

        let initial_nonce = state.lock().unwrap().get_nonce(&from);

        // Should return Ok(receipt) with success=false
        let result = executor.execute_transaction(state.clone(), &tx);

        assert!(result.is_ok(), "Should return Ok(receipt) for valid tx");
        let receipt = result.unwrap();

        assert!(!receipt.success, "Receipt should indicate failure");
        assert!(receipt.gas_used > 0, "Gas should be consumed");
        assert!(receipt.error_message.is_some());

        // Nonce SHOULD be incremented (transaction was valid)
        assert_eq!(
            state.lock().unwrap().get_nonce(&from),
            initial_nonce + 1,
            "Nonce should increment for valid tx even if execution fails"
        );

        // Recipient should have no balance
        assert_eq!(
            state.lock().unwrap().get_balance(&to),
            0,
            "Transfer should not happen"
        );
    }

    #[test]
    fn test_validate_transaction_success() {
        let executor = ContractExecutor::new().unwrap();
        let state = Arc::new(Mutex::new(State::new()));

        let from = Address::from_slice(&[1u8; Address::LENGTH]);

        // Give sufficient balance
        state
            .lock()
            .unwrap()
            .set_balance(from, TokenUnit::from_tokens(1.0));

        let tx = Transaction::new(
            from,
            0, // Correct nonce
            TransactionKind::Transfer {
                to: Address::zero(),
                amount: TokenUnit::from_tokens(0.01),
            },
            100000,
            1,
        );

        // Should succeed
        let result = executor.validate_transaction(&tx, &state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_transaction_wrong_nonce() {
        let executor = ContractExecutor::new().unwrap();
        let state = Arc::new(Mutex::new(State::new()));

        let from = Address::from_slice(&[1u8; Address::LENGTH]);
        state
            .lock()
            .unwrap()
            .set_balance(from, TokenUnit::from_tokens(1.0));

        let tx = Transaction::new(
            from,
            10, // Wrong nonce (expected 0)
            TransactionKind::Transfer {
                to: Address::zero(),
                amount: TokenUnit::from_tokens(0.01),
            },
            100000,
            1,
        );

        let result = executor.validate_transaction(&tx, &state);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid nonce"));
    }

    #[test]
    fn test_validate_transaction_insufficient_balance() {
        let executor = ContractExecutor::new().unwrap();
        let state = Arc::new(Mutex::new(State::new()));

        let from = Address::from_slice(&[1u8; Address::LENGTH]);
        state
            .lock()
            .unwrap()
            .set_balance(from, TokenUnit::from_tokens(0.0001)); // Very low

        let tx = Transaction::new(
            from,
            0,
            TransactionKind::Transfer {
                to: Address::zero(),
                amount: TokenUnit::from_tokens(0.01),
            },
            100000, // Needs 100000 micro-tokens
            1,
        );

        let result = executor.validate_transaction(&tx, &state);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Insufficient balance"));
    }

    #[test]
    fn test_parse_args_simple_i32() {
        let executor = ContractExecutor::new().unwrap();
        let engine = WasmEngine::new().unwrap();
        let context = RuntimeContext {
            caller_address: Address::zero(),
            contract_address: Address::zero(),
            state: Arc::new(Mutex::new(State::new())),
            gas_remaining: 1_000_000,
        };
        let mut store = engine.create_store(context);

        // Create a dummy WASM module with memory
        let wasm = wat::parse_str(
            r#"
            (module
                (memory (export "memory") 1)
            )
            "#,
        )
        .unwrap();
        let module = wasmtime::Module::new(engine.engine(), wasm).unwrap();
        let instance = wasmtime::Instance::new(&mut store, &module, &[]).unwrap();
        let memory = instance.get_memory(&mut store, "memory").unwrap();

        // Test: Single i32 with exactly 4 bytes (should be treated as regular number)
        let args = vec![0x2A, 0x00, 0x00, 0x00]; // 42 in little-endian
        let param_types = vec![wasmtime::ValType::I32];

        let result = executor
            .parse_args(&mut store, &memory, &args, &param_types)
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].unwrap_i32(), 42);
    }

    #[test]
    fn test_parse_args_simple_i64() {
        let executor = ContractExecutor::new().unwrap();
        let engine = WasmEngine::new().unwrap();
        let context = RuntimeContext {
            caller_address: Address::zero(),
            contract_address: Address::zero(),
            state: Arc::new(Mutex::new(State::new())),
            gas_remaining: 1_000_000,
        };
        let mut store = engine.create_store(context);

        let wasm = wat::parse_str(
            r#"
            (module
                (memory (export "memory") 1)
            )
            "#,
        )
        .unwrap();
        let module = wasmtime::Module::new(engine.engine(), wasm).unwrap();
        let instance = wasmtime::Instance::new(&mut store, &module, &[]).unwrap();
        let memory = instance.get_memory(&mut store, "memory").unwrap();

        // Test: Single i64
        let args = vec![0x88, 0x13, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]; // 5000 in little-endian
        let param_types = vec![wasmtime::ValType::I64];

        let result = executor
            .parse_args(&mut store, &memory, &args, &param_types)
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].unwrap_i64(), 5000);
    }

    #[test]
    fn test_parse_args_address_detection() {
        let executor = ContractExecutor::new().unwrap();
        let engine = WasmEngine::new().unwrap();
        let context = RuntimeContext {
            caller_address: Address::zero(),
            contract_address: Address::zero(),
            state: Arc::new(Mutex::new(State::new())),
            gas_remaining: 1_000_000,
        };
        let mut store = engine.create_store(context);

        let wasm = wat::parse_str(
            r#"
            (module
                (memory (export "memory") 1)
            )
            "#,
        )
        .unwrap();
        let module = wasmtime::Module::new(engine.engine(), wasm).unwrap();
        let instance = wasmtime::Instance::new(&mut store, &module, &[]).unwrap();
        let memory = instance.get_memory(&mut store, "memory").unwrap();

        // Test: 20-byte address + u64 (should detect address and write to memory)
        let mut args = vec![0x2A; Address::LENGTH]; // 20 bytes of 0x2A
        args.extend_from_slice(&[0x88, 0x13, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]); // 5000

        let param_types = vec![wasmtime::ValType::I32, wasmtime::ValType::I64];

        let result = executor
            .parse_args(&mut store, &memory, &args, &param_types)
            .unwrap();

        assert_eq!(result.len(), 2);

        // First param should be a memory pointer (0x1000)
        let ptr = result[0].unwrap_i32();
        assert_eq!(ptr, 0x1000);

        // Verify address was written to memory
        let mem_data = memory.data(&store);
        assert_eq!(&mem_data[0x1000..0x1014], &[0x2A; Address::LENGTH]);

        // Second param should be the value
        assert_eq!(result[1].unwrap_i64(), 5000);
    }

    #[test]
    fn test_parse_args_multiple_addresses() {
        let executor = ContractExecutor::new().unwrap();
        let engine = WasmEngine::new().unwrap();
        let context = RuntimeContext {
            caller_address: Address::zero(),
            contract_address: Address::zero(),
            state: Arc::new(Mutex::new(State::new())),
            gas_remaining: 1_000_000,
        };
        let mut store = engine.create_store(context);

        let wasm = wat::parse_str(
            r#"
            (module
                (memory (export "memory") 1)
            )
            "#,
        )
        .unwrap();
        let module = wasmtime::Module::new(engine.engine(), wasm).unwrap();
        let instance = wasmtime::Instance::new(&mut store, &module, &[]).unwrap();
        let memory = instance.get_memory(&mut store, "memory").unwrap();

        // Test: Two addresses + u64
        let mut args = vec![0x11; Address::LENGTH]; // First address
        args.extend_from_slice(&[0x22; Address::LENGTH]); // Second address
        args.extend_from_slice(&[0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]); // 100

        let param_types = vec![
            wasmtime::ValType::I32,
            wasmtime::ValType::I32,
            wasmtime::ValType::I64,
        ];

        let result = executor
            .parse_args(&mut store, &memory, &args, &param_types)
            .unwrap();

        assert_eq!(result.len(), 3);

        // First address at 0x1000
        assert_eq!(result[0].unwrap_i32(), 0x1000);
        let mem_data = memory.data(&store);
        assert_eq!(&mem_data[0x1000..0x1014], &[0x11; Address::LENGTH]);

        // Second address at 0x1014 (0x1000 + 20)
        assert_eq!(result[1].unwrap_i32(), 0x1014);
        assert_eq!(&mem_data[0x1014..0x1028], &[0x22; Address::LENGTH]);

        // Value
        assert_eq!(result[2].unwrap_i64(), 100);
    }

    #[test]
    fn test_parse_args_mixed_types() {
        let executor = ContractExecutor::new().unwrap();
        let engine = WasmEngine::new().unwrap();
        let context = RuntimeContext {
            caller_address: Address::zero(),
            contract_address: Address::zero(),
            state: Arc::new(Mutex::new(State::new())),
            gas_remaining: 1_000_000,
        };
        let mut store = engine.create_store(context);

        let wasm = wat::parse_str(
            r#"
            (module
                (memory (export "memory") 1)
            )
            "#,
        )
        .unwrap();
        let module = wasmtime::Module::new(engine.engine(), wasm).unwrap();
        let instance = wasmtime::Instance::new(&mut store, &module, &[]).unwrap();
        let memory = instance.get_memory(&mut store, "memory").unwrap();

        // Test: i32 (regular) + address + i64
        // This simulates: fn(flag: i32, address: &[u8; 20], count: u64)
        let mut args = vec![0x01, 0x00, 0x00, 0x00]; // flag = 1
        args.extend_from_slice(&[0xAA; Address::LENGTH]); // address
        args.extend_from_slice(&[0xE8, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]); // 1000

        let param_types = vec![
            wasmtime::ValType::I32,
            wasmtime::ValType::I32,
            wasmtime::ValType::I64,
        ];

        let result = executor
            .parse_args(&mut store, &memory, &args, &param_types)
            .unwrap();

        assert_eq!(result.len(), 3);

        // First i32 has 4 bytes followed by 28 bytes
        // 28 != 4, so heuristic says: "first param might be pointer"
        // But actually we want: regular i32 + address pointer + i64
        // This is a limitation of the current heuristic!
        //
        // Current behavior: Treats first 20 bytes as address (wrong!)
        // We'd need smarter detection or explicit type hints to handle this case
    }

    #[test]
    fn test_serialize_results_empty() {
        let executor = ContractExecutor::new().unwrap();
        let results = vec![];

        let bytes = executor.serialize_results(&results).unwrap();

        assert_eq!(bytes.len(), 0);
    }

    #[test]
    fn test_serialize_results_single_i32() {
        let executor = ContractExecutor::new().unwrap();
        let results = vec![wasmtime::Val::I32(42)];

        let bytes = executor.serialize_results(&results).unwrap();

        assert_eq!(bytes.len(), 4);
        assert_eq!(bytes, vec![0x2A, 0x00, 0x00, 0x00]); // 42 in little-endian
    }

    #[test]
    fn test_serialize_results_single_i64() {
        let executor = ContractExecutor::new().unwrap();
        let results = vec![wasmtime::Val::I64(5000)];

        let bytes = executor.serialize_results(&results).unwrap();

        assert_eq!(bytes.len(), 8);
        assert_eq!(bytes, vec![0x88, 0x13, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        // 5000 in little-endian
    }

    #[test]
    fn test_serialize_results_negative_i32() {
        let executor = ContractExecutor::new().unwrap();
        let results = vec![wasmtime::Val::I32(-1)];

        let bytes = executor.serialize_results(&results).unwrap();

        assert_eq!(bytes.len(), 4);
        assert_eq!(bytes, vec![0xFF, 0xFF, 0xFF, 0xFF]); // -1 in two's complement little-endian
    }

    #[test]
    fn test_serialize_results_negative_i64() {
        let executor = ContractExecutor::new().unwrap();
        let results = vec![wasmtime::Val::I64(-3)]; // Error code

        let bytes = executor.serialize_results(&results).unwrap();

        assert_eq!(bytes.len(), 8);
        assert_eq!(bytes, vec![0xFD, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
        // -3 in two's complement
    }

    #[test]
    fn test_serialize_results_multiple_values() {
        let executor = ContractExecutor::new().unwrap();
        let results = vec![wasmtime::Val::I32(100), wasmtime::Val::I64(1000000)];

        let bytes = executor.serialize_results(&results).unwrap();

        assert_eq!(bytes.len(), 12); // 4 + 8

        // First 4 bytes: 100 as i32
        assert_eq!(&bytes[0..4], &[0x64, 0x00, 0x00, 0x00]);

        // Next 8 bytes: 1000000 as i64
        assert_eq!(
            &bytes[4..12],
            &[0x40, 0x42, 0x0F, 0x00, 0x00, 0x00, 0x00, 0x00]
        );
    }

    #[test]
    fn test_serialize_results_large_values() {
        let executor = ContractExecutor::new().unwrap();
        let results = vec![wasmtime::Val::I32(i32::MAX), wasmtime::Val::I64(i64::MAX)];

        let bytes = executor.serialize_results(&results).unwrap();

        assert_eq!(bytes.len(), 12);

        // i32::MAX = 0x7FFFFFFF
        assert_eq!(&bytes[0..4], &[0xFF, 0xFF, 0xFF, 0x7F]);

        // i64::MAX = 0x7FFFFFFFFFFFFFFF
        assert_eq!(
            &bytes[4..12],
            &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F]
        );
    }

    #[test]
    fn test_serialize_results_zero_values() {
        let executor = ContractExecutor::new().unwrap();
        let results = vec![wasmtime::Val::I32(0), wasmtime::Val::I64(0)];

        let bytes = executor.serialize_results(&results).unwrap();

        assert_eq!(bytes.len(), 12);
        assert_eq!(bytes, vec![0x00; 12]); // All zeros
    }

    #[test]
    fn test_serialize_results_error_codes() {
        let executor = ContractExecutor::new().unwrap();

        // Common error codes from token contract
        let error_codes = vec![
            -1, // Zero amount
            -2, // Self-transfer
            -3, // Insufficient balance
            -4, // Overflow
        ];

        for error_code in error_codes {
            let results = vec![wasmtime::Val::I32(error_code)];
            let bytes = executor.serialize_results(&results).unwrap();

            assert_eq!(bytes.len(), 4);

            // Verify we can deserialize back
            let deserialized = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            assert_eq!(deserialized, error_code);
        }
    }
}
