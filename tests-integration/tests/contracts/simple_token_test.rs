//! Simple Token contract integration tests

use crate::common::{TestEnv, ArgsBuilder};
use wasmlette_blockchain::Address;

const TOKEN_WASM: &[u8] = include_bytes!("../../../target/wasm32-unknown-unknown/release/simple_token.wasm");

fn get_balance_key(address: &Address) -> Vec<u8> {
    let mut key = b"balance".to_vec();
    key.extend_from_slice(address.as_bytes());
    key
}

#[test]
fn test_token_deploy() {
    let env = TestEnv::new();
    let deployer = env.create_account(1, 10_000_000);

    let initial_supply = 1_000_000u64;
    let init_args = ArgsBuilder::new()
        .add_u64(initial_supply)
        .build();

    let token = env.deploy_contract(deployer, 0, TOKEN_WASM, init_args)
        .expect("Deploy should succeed");

    assert!(env.contract_exists(&token));

    // Verify total supply
    let supply = env.get_storage_u64(token, b"total_supply").unwrap();
    assert_eq!(supply, initial_supply);

    // Verify total supply
    let transfer_count = env.get_storage_u64(token, b"transfer_count").unwrap();
    assert_eq!(transfer_count, 0);

    // Verify deployer has all tokens
    let balance = env.get_storage_u64(token, &get_balance_key(&deployer)).unwrap();
    assert_eq!(balance, initial_supply);
}

#[test]
fn test_token_transfer() {
    let env = TestEnv::new();
    let alice = env.create_account(1, 10_000_000);
    let bob = env.create_account(2, 10_000_000);

    // Deploy with Alice as owner
    let initial_supply = 1_000_000u64;
    let init_args = ArgsBuilder::new()
        .add_u64(initial_supply)
        .build();

    let token = env.deploy_contract(alice, 0, TOKEN_WASM, init_args).unwrap();

    // Alice transfers 5000 tokens to Bob
    let transfer_args = ArgsBuilder::new()
        .add_address(&bob)
        .add_u64(5000)
        .build();

    env.call_contract(alice, 1, token, "transfer", transfer_args)
        .expect("Transfer should succeed");

    // Verify balances
    let alice_balance = env.get_storage_u64(token, &get_balance_key(&alice)).unwrap();
    let bob_balance = env.get_storage_u64(token, &get_balance_key(&bob)).unwrap();

    assert_eq!(alice_balance, 995_000);
    assert_eq!(bob_balance, 5_000);

    // Verify total supply unchanged
    let supply = env.get_storage_u64(token, b"total_supply").unwrap();
    assert_eq!(supply, initial_supply);
}

#[test]
fn test_token_transfer_insufficient_balance() {
    let env = TestEnv::new();
    let alice = env.create_account(1, 10_000_000);
    let bob = env.create_account(2, 10_000_000);

    let init_args = ArgsBuilder::new()
        .add_u64(1_000)
        .build();

    let token = env.deploy_contract(alice, 0, TOKEN_WASM, init_args).unwrap();

    // Try to transfer more than Alice has
    let transfer_args = ArgsBuilder::new()
        .add_address(&bob)
        .add_u64(2_000)  // More than 1000!
        .build();

    // This should fail with error code -3 (insufficient balance)
    // Note: Currently we expect transaction to succeed but return error code
    // In a real system, might want to check return data
    let result = env.call_contract(alice, 1, token, "transfer", transfer_args);

    // For now, just verify balances didn't change
    let alice_balance = env.get_storage_u64(token, &get_balance_key(&alice)).unwrap();
    assert_eq!(alice_balance, 1_000, "Alice's balance should be unchanged");
}

#[test]
fn test_token_transfer_zero_amount() {
    let env = TestEnv::new();
    let alice = env.create_account(1, 10_000_000);
    let bob = env.create_account(2, 10_000_000);

    let init_args = ArgsBuilder::new()
        .add_u64(1_000)
        .build();

    let token = env.deploy_contract(alice, 0, TOKEN_WASM, init_args).unwrap();

    // Try to transfer 0 tokens (should return error -1)
    let transfer_args = ArgsBuilder::new()
        .add_address(&bob)
        .add_u64(0)
        .build();

    let _result = env.call_contract(alice, 1, token, "transfer", transfer_args);

    // Verify balances didn't change
    let alice_balance = env.get_storage_u64(token, &get_balance_key(&alice)).unwrap();
    assert_eq!(alice_balance, 1_000);
}

#[test]
fn test_token_self_transfer() {
    let env = TestEnv::new();
    let alice = env.create_account(1, 10_000_000);

    let init_args = ArgsBuilder::new()
        .add_u64(1_000)
        .build();

    let token = env.deploy_contract(alice, 0, TOKEN_WASM, init_args).unwrap();

    // Try to transfer to self (should return error -2)
    let transfer_args = ArgsBuilder::new()
        .add_address(&alice)
        .add_u64(100)
        .build();

    let _result = env.call_contract(alice, 1, token, "transfer", transfer_args);

    // Verify balance didn't change
    let alice_balance = env.get_storage_u64(token, &get_balance_key(&alice)).unwrap();
    assert_eq!(alice_balance, 1_000);
}

#[test]
fn test_token_multiple_transfers() {
    let env = TestEnv::new();
    let alice = env.create_account(1, 10_000_000);
    let bob = env.create_account(2, 10_000_000);
    let charlie = env.create_account(3, 10_000_000);

    let init_args = ArgsBuilder::new()
        .add_u64(10_000)
        .build();

    let token = env.deploy_contract(alice, 0, TOKEN_WASM, init_args).unwrap();

    // Alice -> Bob: 3000
    let args1 = ArgsBuilder::new().add_address(&bob).add_u64(3000).build();
    env.call_contract(alice, 1, token, "transfer", args1).unwrap();

    // Alice -> Charlie: 2000
    let args2 = ArgsBuilder::new().add_address(&charlie).add_u64(2000).build();
    env.call_contract(alice, 2, token, "transfer", args2).unwrap();

    // Bob -> Charlie: 1000
    let args3 = ArgsBuilder::new().add_address(&charlie).add_u64(1000).build();
    env.call_contract(bob, 0, token, "transfer", args3).unwrap();

    // Verify final balances
    let alice_balance = env.get_storage_u64(token, &get_balance_key(&alice)).unwrap();
    let bob_balance = env.get_storage_u64(token, &get_balance_key(&bob)).unwrap();
    let charlie_balance = env.get_storage_u64(token, &get_balance_key(&charlie)).unwrap();

    assert_eq!(alice_balance, 5_000);   // 10000 - 3000 - 2000
    assert_eq!(bob_balance, 2_000);     // 3000 - 1000
    assert_eq!(charlie_balance, 3_000); // 2000 + 1000

    // Verify total supply unchanged
    let supply = env.get_storage_u64(token, b"total_supply").unwrap();
    assert_eq!(supply, 10_000);
}

#[test]
fn test_token_query_balance() {
    let env = TestEnv::new();
    let alice = env.create_account(1, 10_000_000);
    let bob = env.create_account(2, 10_000_000);

    let init_args = ArgsBuilder::new()
        .add_u64(1_000)
        .build();

    let token = env.deploy_contract(alice, 0, TOKEN_WASM, init_args).unwrap();

    // Query Bob's balance (should be 0)
    let bob_balance = env.get_storage_u64(token, &get_balance_key(&bob)).unwrap_or(0);
    assert_eq!(bob_balance, 0);

    // Transfer some to Bob
    let transfer_args = ArgsBuilder::new().add_address(&bob).add_u64(500).build();
    env.call_contract(alice, 1, token, "transfer", transfer_args).unwrap();

    // Query Bob's balance again
    let bob_balance = env.get_storage_u64(token, &get_balance_key(&bob)).unwrap();
    assert_eq!(bob_balance, 500);
}
