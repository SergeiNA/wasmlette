use std::cell::RefCell;
use std::rc::Rc;
use wasmlette_blockchain::transaction::{Transaction, TransactionKind};
use wasmlette_blockchain::{transaction::Address, State};
use wasmlette_runtime::ContractExecutor;
use wasmlette_tokens::TokenUnit;

#[test]
fn test_deploy_and_call_contract() {
    // Setup
    let state = Rc::new(RefCell::new(State::new()));
    let executor = ContractExecutor::new().unwrap();

    // Use counter_raw which doesn't have an init() function
    let wasm_code = include_bytes!("../../../target/wasm32-unknown-unknown/release/counter.wasm");

    // Create deploy transaction
    let deployer = Address::from_slice(&[0x01; Address::LENGTH]);
    let deployer_balance_tokens = 2.0;
    state
        .borrow_mut()
        .set_balance(deployer, TokenUnit::from_tokens(deployer_balance_tokens)); // Give deployer some balance

    let deploy_tx = Transaction {
        sender: deployer,
        nonce: 0,
        kind: TransactionKind::Deploy {
            wasm_code: wasm_code.to_vec(),
            init_args: vec![],
        },
        gas_limit: 1_000_000,
        gas_price: 1u64,
    };

    // Execute deploy
    let receipt = executor
        .execute_transaction(state.clone(), &deploy_tx)
        .unwrap();
    assert!(
        receipt.success,
        "Deploy failed: {:?}",
        receipt.error_message
    );

    let contract_address = receipt.contract_address.expect("No contract address");

    // Verify contract exists
    assert!(state.borrow().contract_exists(&contract_address));

    // Verify storage was initialized
    let stored_value = state
        .borrow()
        .get_storage(contract_address, b"count".to_vec());
    assert!(
        stored_value.is_some(),
        "Storage should contain 'count' key after init"
    );

    // Verify count is 0 (little-endian u64) after init
    assert_eq!(stored_value.unwrap(), 0u64.to_le_bytes().to_vec());

    // Call increment() function from counter_raw contract
    let call_tx = Transaction {
        sender: deployer,
        nonce: 1,
        kind: TransactionKind::Call {
            contract: contract_address,
            method: "increment".to_string(),
            args: vec![],
        },
        gas_limit: 1_000_000,
        gas_price: 1u64,
    };

    // Execute call
    let call_receipt = executor
        .execute_transaction(state.clone(), &call_tx)
        .unwrap();
    assert!(
        call_receipt.success,
        "Call failed: {:?}",
        call_receipt.error_message
    );

    // Verify storage was updated (use contract_address!)
    let stored_value = state
        .borrow()
        .get_storage(contract_address, b"count".to_vec());
    assert!(
        stored_value.is_some(),
        "Storage should contain 'count' key after increment"
    );

    // Verify count is 1 (little-endian u64)
    assert_eq!(stored_value.unwrap(), 1u64.to_le_bytes().to_vec());

    println!("Deploy gas used: {}", receipt.gas_used);
    println!("Call gas used: {}", call_receipt.gas_used);
}
