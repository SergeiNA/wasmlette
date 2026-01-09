//! Counter contract integration tests

use crate::common::{ArgsBuilder, TestEnv};
use wasmlette_blockchain::Address;
use wasmlette_runtime::TokenUnit;

const COUNTER_WASM: &[u8] =
    include_bytes!("../../../target/wasm32-unknown-unknown/release/counter.wasm");

#[test]
fn test_counter_deploy_and_init() {
    let env = TestEnv::new();
    let deployer = env.create_account(1, TokenUnit::from_tokens(10.0));

    // Deploy counter contract
    let contract = env
        .deploy_contract(deployer, 0, COUNTER_WASM, vec![])
        .expect("Deploy should succeed");

    // Verify contract exists
    assert!(env.contract_exists(&contract));

    // Verify initial count is 0 (init sets it)
    let count = env
        .get_storage_u64(contract, b"count")
        .expect("Count should be initialized");
    assert_eq!(count, 0, "Initial count should be 0");
}

#[test]
fn test_counter_increment() {
    let env = TestEnv::new();
    let deployer = env.create_account(1, TokenUnit::from_tokens(10.0));

    // Deploy
    let contract = env
        .deploy_contract(deployer, 0, COUNTER_WASM, vec![])
        .expect("Deploy should succeed");

    // Increment
    env.call_contract(deployer, 1, contract, "increment", vec![])
        .expect("Increment should succeed");

    // Verify count is 1
    let count = env.get_storage_u64(contract, b"count").unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_counter_multiple_increments() {
    let env = TestEnv::new();
    let deployer = env.create_account(1, TokenUnit::from_tokens(10.0));

    let contract = env
        .deploy_contract(deployer, 0, COUNTER_WASM, vec![])
        .expect("Deploy should succeed");

    // Increment 5 times
    for nonce in 1..=5 {
        env.call_contract(deployer, nonce, contract, "increment", vec![])
            .expect("Increment should succeed");
    }

    // Verify count is 5
    let count = env.get_storage_u64(contract, b"count").unwrap();
    assert_eq!(count, 5);
}

#[test]
fn test_counter_decrement() {
    let env = TestEnv::new();
    let deployer = env.create_account(1, TokenUnit::from_tokens(10.0));

    let contract = env
        .deploy_contract(deployer, 0, COUNTER_WASM, vec![])
        .expect("Deploy should succeed");

    // Increment twice
    env.call_contract(deployer, 1, contract, "increment", vec![])
        .unwrap();
    env.call_contract(deployer, 2, contract, "increment", vec![])
        .unwrap();

    // Decrement once
    env.call_contract(deployer, 3, contract, "decrement", vec![])
        .expect("Decrement should succeed");

    // Verify count is 1
    let count = env.get_storage_u64(contract, b"count").unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_counter_set_count() {
    let env = TestEnv::new();
    let deployer = env.create_account(1, TokenUnit::from_tokens(10.0));

    let contract = env
        .deploy_contract(deployer, 0, COUNTER_WASM, vec![])
        .expect("Deploy should succeed");

    // Set count to 42
    let args = ArgsBuilder::new().add_u64(42).build();
    env.call_contract(deployer, 1, contract, "set_count", args)
        .expect("set_count should succeed");

    // Verify count is 42
    let count = env.get_storage_u64(contract, b"count").unwrap();
    assert_eq!(count, 42);
}

#[test]
fn test_counter_get_count() {
    let env = TestEnv::new();
    let deployer = env.create_account(1, TokenUnit::from_tokens(10.0));

    let contract = env
        .deploy_contract(deployer, 0, COUNTER_WASM, vec![])
        .expect("Deploy should succeed");

    // Increment
    env.call_contract(deployer, 1, contract, "increment", vec![])
        .unwrap();

    // Get count (returns u64)
    let result = env
        .call_contract(deployer, 2, contract, "get_count", vec![])
        .expect("get_count should succeed");

    // Note: Need to handle return value properly
    // For now, verify via storage
    let count = env.get_storage_u64(contract, b"count").unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_counter_cannot_decrement_below_zero() {
    let env = TestEnv::new();
    let deployer = env.create_account(1, TokenUnit::from_tokens(10.0));

    let contract = env
        .deploy_contract(deployer, 0, COUNTER_WASM, vec![])
        .expect("Deploy should succeed");

    // Try to decrement from 0 (should be no-op)
    env.call_contract(deployer, 1, contract, "decrement", vec![])
        .expect("Should not fail, just no-op");

    // Verify count is still 0
    let count = env.get_storage_u64(contract, b"count").unwrap();
    assert_eq!(count, 0);
}
