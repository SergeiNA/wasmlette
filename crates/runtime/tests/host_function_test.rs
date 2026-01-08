use std::cell::RefCell;
use std::rc::Rc;
use wasmlette_blockchain::{Address, State};
use wasmlette_runtime::{RuntimeContext, WasmEngine};
use wasmtime::Linker;

#[test]
fn test_storage_host_functions() {
    // Setup
    let state = Rc::new(RefCell::new(State::new()));
    let contract = Address::from_slice(&[0xAB; Address::LENGTH]);
    let wasm_code = include_bytes!("../../../target/wasm32-unknown-unknown/release/tester.wasm");

    state
        .borrow_mut()
        .deploy_contract(contract, wasm_code.to_vec().clone())
        .unwrap();

    // Create context
    let context = RuntimeContext {
        caller_address: Address::zero(),
        contract_address: contract,
        state: state.clone(),
        gas_remaining: 1_000_000_000,
    };

    // Create engine and store
    let engine = WasmEngine::new().unwrap();
    let mut store = engine.create_store(context);
    store.set_fuel(1_000_000_000).unwrap();

    // Load module
    let module = engine.load_module(&wasm_code.to_vec()).unwrap();

    // Register host functions
    let mut linker = Linker::new(engine.engine());
    wasmlette_runtime::host_api::register_host_functions(&mut linker).unwrap();

    // Instantiate
    let instance = linker.instantiate(&mut store, &module).unwrap();

    // Call init function
    let init_func = instance.get_func(&mut store, "init").unwrap();
    init_func.call(&mut store, &[], &mut []).unwrap();

    // Check initial count is 0
    let value = state
        .borrow()
        .get_storage(contract, b"count".into())
        .unwrap();
    let count = u64::from_le_bytes(value.try_into().unwrap());
    assert_eq!(count, 0);

    // Call contract function that uses storage
    // Test set
    let set_func = instance.get_func(&mut store, "test_set_count").unwrap();
    set_func.call(&mut store, &[42i64.into()], &mut []).unwrap();

    // Test get
    let mut results = [wasmtime::Val::I64(0)];
    let get_func = instance.get_func(&mut store, "test_get_count").unwrap();
    get_func.call(&mut store, &[], &mut results).unwrap();
    let count_from_func = results[0].unwrap_i64() as u64;
    assert_eq!(count_from_func, 42);

    // Verify state was modified
    let value = state
        .borrow()
        .get_storage(contract, b"count".into())
        .unwrap();
    let count = u64::from_le_bytes(value.try_into().unwrap());
    assert_eq!(count, 42);
}

#[test]
fn test_balance_host_functions() {
    // Setup
    let state = Rc::new(RefCell::new(State::new()));
    let contract = Address::from_slice(&[0xAB; Address::LENGTH]);
    let wasm_code = include_bytes!("../../../target/wasm32-unknown-unknown/release/tester.wasm");

    // Set up balances for testing
    let alice = Address::from_slice(&[0x01; Address::LENGTH]);
    let bob = Address::from_slice(&[0x02; Address::LENGTH]);
    let zero_balance = Address::from_slice(&[0x03; Address::LENGTH]);

    state.borrow_mut().set_balance(alice, 1_000_000u64);
    state.borrow_mut().set_balance(bob, 500_000u64);
    // zero_balance intentionally has no balance (0)

    // Deploy contract
    state
        .borrow_mut()
        .deploy_contract(contract, wasm_code.to_vec())
        .unwrap();

    // Create context
    let context = RuntimeContext {
        caller_address: Address::zero(),
        contract_address: contract,
        state: state.clone(),
        gas_remaining: 1_000_000_000,
    };

    // Create engine and store
    let engine = WasmEngine::new().unwrap();
    let mut store = engine.create_store(context);
    store.set_fuel(1_000_000_000).unwrap();

    // Load module
    let module = engine.load_module(&wasm_code.to_vec()).unwrap();

    // Register host functions
    let mut linker = Linker::new(engine.engine());
    wasmlette_runtime::host_api::register_host_functions(&mut linker).unwrap();

    // Instantiate
    let instance = linker.instantiate(&mut store, &module).unwrap();

    // Test 1: Check Alice's balance
    let balance_func = instance.get_func(&mut store, "test_get_balance").unwrap();

    // Get memory
    let memory = instance.get_memory(&mut store, "memory").unwrap();

    // Write Alice's address to memory at offset 0
    memory.write(&mut store, 0, alice.as_bytes()).unwrap();

    let mut results = [wasmtime::Val::I64(0)];
    // Call the function with memory pointer
    balance_func
        .call(&mut store, &[wasmtime::Val::I32(0)], &mut results)
        .unwrap();

    let alice_balance = results[0].unwrap_i64() as u64;
    assert_eq!(alice_balance, 1_000_000);

    // Test 2: Check Bob's balance
    memory.write(&mut store, 0, bob.as_bytes()).unwrap();
    balance_func
        .call(&mut store, &[wasmtime::Val::I32(0)], &mut results)
        .unwrap();

    let bob_balance = results[0].unwrap_i64() as u64;
    assert_eq!(bob_balance, 500_000);

    // Test 3: Check address with no balance
    memory
        .write(&mut store, 0, zero_balance.as_bytes())
        .unwrap();
    balance_func
        .call(&mut store, &[wasmtime::Val::I32(0)], &mut results)
        .unwrap();

    let zero_bal = results[0].unwrap_i64() as u64;
    assert_eq!(zero_bal, 0);
}

#[test]
fn test_hash_host_functions() {
    // Setup
    let state = Rc::new(RefCell::new(State::new()));
    let contract = Address::from_slice(&[0xAB; Address::LENGTH]);
    let wasm_code = include_bytes!("../../../target/wasm32-unknown-unknown/release/tester.wasm");

    // Deploy contract
    state
        .borrow_mut()
        .deploy_contract(contract, wasm_code.to_vec())
        .unwrap();

    // Create context
    let context = RuntimeContext {
        caller_address: Address::zero(),
        contract_address: contract,
        state: state.clone(),
        gas_remaining: 1_000_000_000,
    };

    // Create engine and store
    let engine = WasmEngine::new().unwrap();
    let mut store = engine.create_store(context);
    store.set_fuel(1_000_000_000).unwrap();

    // Load module
    let module = engine.load_module(&wasm_code.to_vec()).unwrap();

    // Register host functions
    let mut linker = Linker::new(engine.engine());
    wasmlette_runtime::host_api::register_host_functions(&mut linker).unwrap();

    // Instantiate
    let instance = linker.instantiate(&mut store, &module).unwrap();

    // Get memory
    let memory = instance.get_memory(&mut store, "memory").unwrap();

    // Test value to hash
    let test_value: u64 = 42;

    // Allocate space in WASM memory for the hash output (32 bytes)
    let output_offset = 0;

    // Get the hash function
    let hash_func = instance.get_func(&mut store, "test_hash").unwrap();
    hash_func
        .call(
            &mut store,
            &[
                wasmtime::Val::I64(test_value as i64),
                wasmtime::Val::I32(output_offset),
            ],
            &mut [],
        )
        .unwrap();

    let mut hash_result = vec![0u8; 32];
    memory
        .read(&store, output_offset as usize, &mut hash_result)
        .unwrap();

    // Verify against expected hash
    let expected_hash = blake3::hash(&test_value.to_le_bytes());
    assert_eq!(hash_result, expected_hash.as_bytes());
}

#[test]
fn test_get_caller_host_functions() {
    // Setup
    let state = Rc::new(RefCell::new(State::new()));
    let contract = Address::from_slice(&[0xAB; Address::LENGTH]);
    let wasm_code = include_bytes!("../../../target/wasm32-unknown-unknown/release/tester.wasm");

    // Deploy contract
    state
        .borrow_mut()
        .deploy_contract(contract, wasm_code.to_vec())
        .unwrap();
    // Create test address
    let alice = Address::from_slice(&[0x01; Address::LENGTH]);

    // Create context
    let context = RuntimeContext {
        caller_address: alice,
        contract_address: contract,
        state: state.clone(),
        gas_remaining: 1_000_000_000,
    };

    // Create engine and store
    let engine = WasmEngine::new().unwrap();
    let mut store = engine.create_store(context);
    store.set_fuel(1_000_000_000).unwrap();

    // Load module
    let module = engine.load_module(&wasm_code.to_vec()).unwrap();

    // Register host functions
    let mut linker = Linker::new(engine.engine());
    wasmlette_runtime::host_api::register_host_functions(&mut linker).unwrap();

    // Instantiate
    let instance = linker.instantiate(&mut store, &module).unwrap();

    // Get memory
    let memory = instance.get_memory(&mut store, "memory").unwrap();

    // Allocate space in WASM memory for the address output (20 bytes)
    let output_offset = 0;

    // Get the get caller function
    let hash_func = instance.get_func(&mut store, "test_get_caller").unwrap();
    hash_func
        .call(
            &mut store,
            &[
                wasmtime::Val::I32(output_offset),
            ],
            &mut [],
        )
        .unwrap();

    let mut caller_address = vec![0u8; Address::LENGTH];
    memory
        .read(&store, output_offset as usize, &mut caller_address)
        .unwrap();

    // Verify address
    assert_eq!(alice, Address::from_slice(&caller_address));
}
