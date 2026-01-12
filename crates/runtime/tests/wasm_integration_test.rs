use std::sync::{Arc, Mutex};
use wasmlette_blockchain::transaction::{Transaction, TransactionKind};
use wasmlette_blockchain::{transaction::Address, State};
use wasmlette_runtime::ContractExecutor;
use wasmlette_tokens::TokenUnit;

#[test]
fn test_deploy_and_call_contract() {
    // Setup
    let state = Arc::new(Mutex::new(State::new()));
    let executor = ContractExecutor::new().unwrap();

    // Use counter_raw which doesn't have an init() function
    let wasm_code = include_bytes!("../../../target/wasm32-unknown-unknown/release/counter.wasm");

    // Create deploy transaction
    let deployer = Address::from_slice(&[0x01; Address::LENGTH]);
    let deployer_balance_tokens = 2.0;
    state
        .lock().unwrap()
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
    assert!(state.lock().unwrap().contract_exists(&contract_address));

    // Verify storage was initialized
    let stored_value = state
        .lock().unwrap()
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
        .lock().unwrap()
        .get_storage(contract_address, b"count".to_vec());
    assert!(
        stored_value.is_some(),
        "Storage should contain 'count' key after increment"
    );

    // Verify count is 1 (little-endian u64)
    assert_eq!(stored_value.unwrap(), 1u64.to_le_bytes().to_vec());

    println!("Deploy gas used: {}", receipt.gas_used);
    println!("Call gas used: {}", call_receipt.gas_used);

    // Verify gas refund was applied
    let deployer_final_balance = state.lock().unwrap().get_balance(&deployer);
    let total_gas_paid = TokenUnit::from_tokens(deployer_balance_tokens) - deployer_final_balance;

    // Calculate expected gas cost
    let expected_cost = (receipt.gas_used + call_receipt.gas_used) * 1; // gas_price = 1
    assert_eq!(
        total_gas_paid, expected_cost,
        "Total gas paid should equal sum of gas used * gas_price"
    );

    // Verify deployer still has some balance (gas refund worked)
    assert!(
        deployer_final_balance > 0,
        "Deployer should have balance left after gas refunds"
    );
}

#[test]
fn test_failed_transaction_gas_refund() {
    // Test that failed transactions still charge gas but refund unused portion
    let state = Arc::new(Mutex::new(State::new()));
    let executor = ContractExecutor::new().unwrap();

    let deployer = Address::from_slice(&[0x01; Address::LENGTH]);
    state
        .lock().unwrap()
        .set_balance(deployer, TokenUnit::from_tokens(10.0));

    // Try to call a non-existent contract
    let non_existent = Address::from_slice(&[0xFF; Address::LENGTH]);

    let call_tx = Transaction {
        sender: deployer,
        nonce: 0,
        kind: TransactionKind::Call {
            contract: non_existent,
            method: "some_method".to_string(),
            args: vec![],
        },
        gas_limit: 1_000_000,
        gas_price: 1,
    };

    let initial_balance = state.lock().unwrap().get_balance(&deployer);

    // Execute - should fail but still process
    let receipt = executor
        .execute_transaction(state.clone(), &call_tx)
        .unwrap();

    assert!(!receipt.success, "Transaction should fail");
    assert!(receipt.error_message.is_some(), "Should have error message");

    // Verify gas was charged but refunded
    let final_balance = state.lock().unwrap().get_balance(&deployer);
    let gas_paid = initial_balance - final_balance;

    // Should charge SOME gas (failure fee) but not the full limit
    assert!(gas_paid > 0, "Should charge some gas even on failure");
    assert!(
        gas_paid < 1_000_000,
        "Should not charge full gas_limit on early failure"
    );

    // Verify refund happened
    let expected_cost = receipt.gas_used * call_tx.gas_price;
    assert_eq!(
        gas_paid, expected_cost,
        "Gas paid should match gas_used * gas_price"
    );
}

#[test]
fn test_multiple_transactions_cumulative_gas() {
    // Test cumulative gas costs across multiple transactions
    let state = Arc::new(Mutex::new(State::new()));
    let executor = ContractExecutor::new().unwrap();

    let wasm_code = include_bytes!("../../../target/wasm32-unknown-unknown/release/counter.wasm");
    let deployer = Address::from_slice(&[0x01; Address::LENGTH]);

    let initial_balance = TokenUnit::from_tokens(10.0);
    state.lock().unwrap().set_balance(deployer, initial_balance);

    // Transaction 1: Deploy
    let deploy_tx = Transaction {
        sender: deployer,
        nonce: 0,
        kind: TransactionKind::Deploy {
            wasm_code: wasm_code.to_vec(),
            init_args: vec![],
        },
        gas_limit: 1_000_000,
        gas_price: 1,
    };

    let receipt1 = executor
        .execute_transaction(state.clone(), &deploy_tx)
        .unwrap();
    assert!(receipt1.success);
    let contract_address = receipt1.contract_address.unwrap();

    let balance_after_deploy = state.lock().unwrap().get_balance(&deployer);
    let gas_cost_1 = initial_balance - balance_after_deploy;

    // Transaction 2: Call increment
    let call_tx1 = Transaction {
        sender: deployer,
        nonce: 1,
        kind: TransactionKind::Call {
            contract: contract_address,
            method: "increment".to_string(),
            args: vec![],
        },
        gas_limit: 1_000_000,
        gas_price: 1,
    };

    let receipt2 = executor
        .execute_transaction(state.clone(), &call_tx1)
        .unwrap();
    assert!(receipt2.success);

    let balance_after_call1 = state.lock().unwrap().get_balance(&deployer);
    let gas_cost_2 = balance_after_deploy - balance_after_call1;

    // Transaction 3: Call increment again
    let call_tx2 = Transaction {
        sender: deployer,
        nonce: 2,
        kind: TransactionKind::Call {
            contract: contract_address,
            method: "increment".to_string(),
            args: vec![],
        },
        gas_limit: 1_000_000,
        gas_price: 1,
    };

    let receipt3 = executor
        .execute_transaction(state.clone(), &call_tx2)
        .unwrap();
    assert!(receipt3.success);

    let final_balance = state.lock().unwrap().get_balance(&deployer);
    let gas_cost_3 = balance_after_call1 - final_balance;

    // Verify cumulative costs
    let total_gas_paid = initial_balance - final_balance;
    let sum_of_costs = gas_cost_1 + gas_cost_2 + gas_cost_3;
    assert_eq!(
        total_gas_paid, sum_of_costs,
        "Total gas should equal sum of individual costs"
    );

    // Verify each transaction charged correctly
    assert_eq!(gas_cost_1, receipt1.gas_used * deploy_tx.gas_price);
    assert_eq!(gas_cost_2, receipt2.gas_used * call_tx1.gas_price);
    assert_eq!(gas_cost_3, receipt3.gas_used * call_tx2.gas_price);

    // Verify counter was incremented twice
    let count = state
        .lock().unwrap()
        .get_storage(contract_address, b"count".to_vec())
        .unwrap();
    assert_eq!(u64::from_le_bytes(count.try_into().unwrap()), 2);

    println!(
        "Deploy gas: {}, Call1 gas: {}, Call2 gas: {}",
        receipt1.gas_used, receipt2.gas_used, receipt3.gas_used
    );
    println!("Total gas paid: {}", total_gas_paid);
}
