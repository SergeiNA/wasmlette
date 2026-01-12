//! Simple Token contract integration tests-integration

use crate::common::{ArgsBuilder, TestEnv};
use wasmlette_blockchain::Address;
use wasmlette_runtime::TokenUnit;

const TOKEN_WASM: &[u8] =
    include_bytes!("../../../target/wasm32-unknown-unknown/release/simple_token.wasm");

fn get_balance_key(address: &Address) -> Vec<u8> {
    let mut key = b"balance".to_vec();
    key.extend_from_slice(address.as_bytes());
    key
}

#[test]
fn test_token_deploy() {
    let init_supply_contract_token = 10_000_000u64;
    let env = TestEnv::new();
    let deployer = env.create_account(1, TokenUnit::from_tokens(10.0));

    let init_args = ArgsBuilder::new()
        .add_u64(init_supply_contract_token)
        .build();

    let token = env
        .deploy_contract(deployer, 0, TOKEN_WASM, init_args)
        .expect("Deploy should succeed");

    assert!(env.contract_exists(&token));

    // Verify total supply
    // TODO should change when gas fee applys. for now we don't change it.
    let supply = env.get_storage_u64(token, b"total_supply").unwrap();
    assert_eq!(supply, init_supply_contract_token);

    // Verify total transfers
    let transfer_count = env.get_storage_u64(token, b"transfer_count").unwrap();
    assert_eq!(transfer_count, 0);

    // Verify deployer has all tokens
    let contract_balance = env
        .get_storage_u64(token, &get_balance_key(&deployer))
        .unwrap();
    assert_eq!(contract_balance, init_supply_contract_token);

    // Verify deployer native token balance was reduced by gas fee
    // Initial balance: 10.0 tokens (10_000_000 units)
    // Gas used: ~133321 units (varies based on deployment cost)
    // Expected remaining: 10_000_000 - 133321 = 9_866_679 units
    let deployer_balance = env.get_balance(&deployer);
    assert_eq!(
        deployer_balance, 9866679,
        "Deployer should have paid gas fee from native token balance"
    );
}

#[test]
fn test_token_transfer() {
    let env = TestEnv::new();
    let alice = env.create_account(1, TokenUnit::from_tokens(10.0));
    let bob = env.create_account(2, TokenUnit::from_tokens(10.0));

    // Deploy with Alice as owner
    let initial_supply = 1_000_000u64;
    let init_args = ArgsBuilder::new().add_u64(initial_supply).build();

    let token = env
        .deploy_contract(alice, 0, TOKEN_WASM, init_args)
        .unwrap();

    // Alice transfers 5000 tokens to Bob
    let transfer_args = ArgsBuilder::new().add_address(&bob).add_u64(5000).build();

    env.call_contract(alice, 1, token, "transfer", transfer_args)
        .expect("Transfer should succeed");

    // Verify balances
    let alice_balance = env
        .get_storage_u64(token, &get_balance_key(&alice))
        .unwrap();
    let bob_balance = env.get_storage_u64(token, &get_balance_key(&bob)).unwrap();

    assert_eq!(alice_balance, 995_000);
    assert_eq!(bob_balance, 5_000);

    // Verify total supply unchanged
    let supply = env.get_storage_u64(token, b"total_supply").unwrap();
    assert_eq!(supply, initial_supply);

    // Verify native token balances (gas fees deducted)
    // Alice: 10.0 tokens - deploy gas - transfer gas ≈ 9.73 tokens
    let alice_native_balance = env.get_balance(&alice);
    assert!(
        alice_native_balance < TokenUnit::from_tokens(10.0),
        "Alice should have paid gas for deploy + transfer"
    );

    // Bob: 10.0 tokens (no transactions, no gas fees)
    let bob_native_balance = env.get_balance(&bob);
    assert_eq!(
        bob_native_balance,
        TokenUnit::from_tokens(10.0),
        "Bob hasn't paid any gas fees"
    );
}

#[test]
fn test_token_transfer_insufficient_balance() {
    let env = TestEnv::new();
    let alice = env.create_account(1, TokenUnit::from_tokens(10.0));
    let bob = env.create_account(2, TokenUnit::from_tokens(10.0));

    let init_args = ArgsBuilder::new().add_u64(1_000).build();

    let token = env
        .deploy_contract(alice, 0, TOKEN_WASM, init_args)
        .unwrap();

    // Try to transfer more than Alice has
    let transfer_args = ArgsBuilder::new()
        .add_address(&bob)
        .add_u64(2_000) // More than 1000!
        .build();

    // This should fail with error code -3 (insufficient balance)
    // Note: Currently we expect transaction to succeed but return error code
    // In a real system, might want to check return data
    let _result = env.call_contract(alice, 1, token, "transfer", transfer_args);

    // For now, just verify balances didn't change
    let alice_balance = env
        .get_storage_u64(token, &get_balance_key(&alice))
        .unwrap();
    assert_eq!(alice_balance, 1_000, "Alice's balance should be unchanged");
}

#[test]
fn test_token_transfer_zero_amount() {
    let env = TestEnv::new();
    let alice = env.create_account(1, TokenUnit::from_tokens(10.0));
    let bob = env.create_account(2, TokenUnit::from_tokens(10.0));

    let init_args = ArgsBuilder::new().add_u64(1_000).build();

    let token = env
        .deploy_contract(alice, 0, TOKEN_WASM, init_args)
        .unwrap();

    // Try to transfer 0 tokens (should return error -1)
    let transfer_args = ArgsBuilder::new().add_address(&bob).add_u64(0).build();

    let _result = env.call_contract(alice, 1, token, "transfer", transfer_args);

    // Verify balances didn't change
    let alice_balance = env
        .get_storage_u64(token, &get_balance_key(&alice))
        .unwrap();
    assert_eq!(alice_balance, 1_000);
}

#[test]
fn test_token_self_transfer() {
    let env = TestEnv::new();
    let alice = env.create_account(1, TokenUnit::from_tokens(10.0));

    let init_args = ArgsBuilder::new().add_u64(1_000).build();

    let token = env
        .deploy_contract(alice, 0, TOKEN_WASM, init_args)
        .unwrap();

    // Try to transfer to self (should return error -2)
    let transfer_args = ArgsBuilder::new().add_address(&alice).add_u64(100).build();

    let _result = env.call_contract(alice, 1, token, "transfer", transfer_args);

    // Verify balance didn't change
    let alice_balance = env
        .get_storage_u64(token, &get_balance_key(&alice))
        .unwrap();
    assert_eq!(alice_balance, 1_000);
}

#[test]
fn test_token_multiple_transfers() {
    let env = TestEnv::new();
    let alice = env.create_account(1, TokenUnit::from_tokens(10.0));
    let bob = env.create_account(2, TokenUnit::from_tokens(10.0));
    let charlie = env.create_account(3, TokenUnit::from_tokens(10.0));

    let init_args = ArgsBuilder::new().add_u64(10_000).build();

    let token = env
        .deploy_contract(alice, 0, TOKEN_WASM, init_args)
        .unwrap();

    // Alice -> Bob: 3000
    let args1 = ArgsBuilder::new().add_address(&bob).add_u64(3000).build();
    env.call_contract(alice, 1, token, "transfer", args1)
        .unwrap();

    // Alice -> Charlie: 2000
    let args2 = ArgsBuilder::new()
        .add_address(&charlie)
        .add_u64(2000)
        .build();
    env.call_contract(alice, 2, token, "transfer", args2)
        .unwrap();

    // Bob -> Charlie: 1000
    let args3 = ArgsBuilder::new()
        .add_address(&charlie)
        .add_u64(1000)
        .build();
    env.call_contract(bob, 0, token, "transfer", args3).unwrap();

    // Verify final balances
    let alice_balance = env
        .get_storage_u64(token, &get_balance_key(&alice))
        .unwrap();
    let bob_balance = env.get_storage_u64(token, &get_balance_key(&bob)).unwrap();
    let charlie_balance = env
        .get_storage_u64(token, &get_balance_key(&charlie))
        .unwrap();

    assert_eq!(alice_balance, 5_000); // 10000 - 3000 - 2000
    assert_eq!(bob_balance, 2_000); // 3000 - 1000
    assert_eq!(charlie_balance, 3_000); // 2000 + 1000

    // Verify total supply unchanged
    let supply = env.get_storage_u64(token, b"total_supply").unwrap();
    assert_eq!(supply, 10_000);

    // Verify native token gas costs
    // Alice: deployed + 2 transfers = 3 transactions
    let alice_native = env.get_balance(&alice);
    let alice_gas_paid = TokenUnit::from_tokens(10.0) - alice_native;
    assert!(
        alice_gas_paid > 0,
        "Alice should have paid gas for 3 transactions"
    );

    // Bob: 1 transfer = 1 transaction
    let bob_native = env.get_balance(&bob);
    let bob_gas_paid = TokenUnit::from_tokens(10.0) - bob_native;
    assert!(
        bob_gas_paid > 0,
        "Bob should have paid gas for 1 transaction"
    );
    assert!(
        alice_gas_paid > bob_gas_paid,
        "Alice paid more gas (3 txs) than Bob (1 tx)"
    );

    // Charlie: no transactions = no gas
    let charlie_native = env.get_balance(&charlie);
    assert_eq!(
        charlie_native,
        TokenUnit::from_tokens(10.0),
        "Charlie hasn't sent any transactions"
    );
}

#[test]
fn test_token_query_balance() {
    let env = TestEnv::new();
    let alice = env.create_account(1, TokenUnit::from_tokens(10.0));
    let bob = env.create_account(2, TokenUnit::from_tokens(10.0));

    let init_args = ArgsBuilder::new().add_u64(1_000).build();

    let token = env
        .deploy_contract(alice, 0, TOKEN_WASM, init_args)
        .unwrap();

    // Query Bob's balance (should be 0)
    let bob_balance = env
        .get_storage_u64(token, &get_balance_key(&bob))
        .unwrap_or(0);
    assert_eq!(bob_balance, 0);

    // Transfer some to Bob
    let transfer_args = ArgsBuilder::new().add_address(&bob).add_u64(500).build();
    env.call_contract(alice, 1, token, "transfer", transfer_args)
        .unwrap();

    // Query Bob's balance again
    let bob_balance = env.get_storage_u64(token, &get_balance_key(&bob)).unwrap();
    assert_eq!(bob_balance, 500);
}

#[test]
fn test_insufficient_native_balance_for_gas() {
    let env = TestEnv::new();
    // Alice has only 0.1 tokens - not enough to deploy
    let alice = env.create_account(1, TokenUnit::from_tokens(0.1));

    let init_args = ArgsBuilder::new().add_u64(1_000).build();

    // Try to deploy - should fail due to insufficient balance for gas
    let result = env.deploy_contract(alice, 0, TOKEN_WASM, init_args);

    assert!(
        result.is_err(),
        "Deploy should fail with insufficient balance for gas"
    );

    // Verify Alice's balance unchanged (no gas charged on rejection)
    let alice_balance = env.get_balance(&alice);
    assert_eq!(
        alice_balance,
        TokenUnit::from_tokens(0.1),
        "No gas should be charged when transaction is rejected"
    );
}

#[test]
fn test_exact_balance_for_gas() {
    let env = TestEnv::new();
    // Deploy with minimal balance, then verify gas refund allows further operations
    let alice = env.create_account(1, TokenUnit::from_tokens(10.0));

    let init_args = ArgsBuilder::new().add_u64(1_000).build();
    let token = env
        .deploy_contract(alice, 0, TOKEN_WASM, init_args)
        .unwrap();

    // Alice should have some balance left due to gas refund
    let alice_balance_after_deploy = env.get_balance(&alice);
    assert!(
        alice_balance_after_deploy > 0,
        "Gas refund should leave some balance"
    );

    // Should be able to make another transaction with refunded gas
    let bob = env.create_account(2, TokenUnit::from_tokens(10.0));
    let transfer_args = ArgsBuilder::new().add_address(&bob).add_u64(100).build();

    let result = env.call_contract(alice, 1, token, "transfer", transfer_args);
    assert!(
        result.is_ok(),
        "Should be able to transact with refunded balance"
    );
}

#[test]
fn test_deploy_invalid_nonce_rejected() {
    let env = TestEnv::new();
    let alice = env.create_account(1, TokenUnit::from_tokens(10.0));

    let initial_balance = env.get_balance(&alice);
    let initial_nonce = env.get_nonce(&alice);

    let init_args = ArgsBuilder::new().add_u64(1_000).build();

    // Try to deploy with wrong nonce (expected 0, providing 5)
    let result = env.deploy_contract(
        alice, 5, // Wrong nonce!
        TOKEN_WASM, init_args,
    );

    // Should be rejected with error
    assert!(result.is_err(), "Invalid nonce should return Err");
    assert!(
        result.unwrap_err().contains("Invalid nonce"),
        "Error should mention invalid nonce"
    );

    // Verify state unchanged
    assert_eq!(
        env.get_balance(&alice),
        initial_balance,
        "Balance should not change on rejected transaction"
    );
    assert_eq!(
        env.get_nonce(&alice),
        initial_nonce,
        "Nonce should not increment on rejected transaction"
    );
}

#[test]
fn test_call_invalid_nonce_rejected() {
    let env = TestEnv::new();
    let alice = env.create_account(1, TokenUnit::from_tokens(10.0));
    let bob = env.create_account(2, TokenUnit::from_tokens(10.0));

    // Deploy token first
    let init_args = ArgsBuilder::new().add_u64(1_000).build();
    let token = env
        .deploy_contract(alice, 0, TOKEN_WASM, init_args)
        .unwrap();

    let balance_after_deploy = env.get_balance(&alice);
    let nonce_after_deploy = env.get_nonce(&alice); // Should be 1

    // Try to call with wrong nonce (expected 1, providing 10)
    let transfer_args = ArgsBuilder::new().add_address(&bob).add_u64(100).build();
    let result = env.call_contract(
        alice,
        10, // Wrong nonce!
        token,
        "transfer",
        transfer_args,
    );

    // Should be rejected
    assert!(result.is_err(), "Invalid nonce should return Err");
    assert!(result.unwrap_err().contains("Invalid nonce"));

    // Verify state unchanged
    assert_eq!(
        env.get_balance(&alice),
        balance_after_deploy,
        "Balance should not change"
    );
    assert_eq!(
        env.get_nonce(&alice),
        nonce_after_deploy,
        "Nonce should not increment"
    );

    // Bob should have no tokens
    let bob_token_balance = env.get_storage_u64(token, &get_balance_key(&bob));
    assert_eq!(bob_token_balance, None, "Transfer should not happen");
}

#[test]
fn test_deploy_insufficient_gas_balance_rejected() {
    let env = TestEnv::new();

    // Give Alice very small balance (not enough for gas)
    let alice = env.create_account(1, TokenUnit::from_tokens(0.001)); // Only 0.001 tokens

    let initial_balance = env.get_balance(&alice);
    let initial_nonce = env.get_nonce(&alice);

    let init_args = ArgsBuilder::new().add_u64(1_000).build();

    // Try to deploy (requires much more gas than available)
    let result = env.deploy_contract(alice, 0, TOKEN_WASM, init_args);

    // Should be rejected
    assert!(result.is_err(), "Insufficient balance should return Err");
    assert!(
        result.unwrap_err().contains("Insufficient balance"),
        "Error should mention insufficient balance"
    );

    // Verify state unchanged
    assert_eq!(
        env.get_balance(&alice),
        initial_balance,
        "Balance should not change"
    );
    assert_eq!(
        env.get_nonce(&alice),
        initial_nonce,
        "Nonce should not increment"
    );
}

#[test]
fn test_call_insufficient_gas_balance_rejected() {
    let env = TestEnv::new();
    let alice = env.create_account(1, TokenUnit::from_tokens(10.0));

    // Deploy token first (Alice uses some gas)
    let init_args = ArgsBuilder::new().add_u64(1_000).build();
    let token = env
        .deploy_contract(alice, 0, TOKEN_WASM, init_args)
        .unwrap();

    // Drain Alice's balance almost completely
    let _current_balance = env.get_balance(&alice);
    let bob = env.create_account(2, TokenUnit::from_tokens(10.0));

    // Create Charlie with very low balance
    let charlie = env.create_account(3, TokenUnit::from_tokens(0.0001));

    let charlie_balance = env.get_balance(&charlie);
    let charlie_nonce = env.get_nonce(&charlie);

    // Try to call contract (Charlie has insufficient gas)
    let transfer_args = ArgsBuilder::new().add_address(&bob).add_u64(50).build();
    let result = env.call_contract(charlie, 0, token, "transfer", transfer_args);

    // Should be rejected
    assert!(result.is_err(), "Insufficient balance should return Err");
    assert!(result.unwrap_err().contains("Insufficient balance"));

    // Verify Charlie's state unchanged
    assert_eq!(
        env.get_balance(&charlie),
        charlie_balance,
        "Balance should not change"
    );
    assert_eq!(
        env.get_nonce(&charlie),
        charlie_nonce,
        "Nonce should not increment"
    );
}

#[test]
fn test_valid_tx_execution_failure_charges_gas() {
    let env = TestEnv::new();
    let alice = env.create_account(1, TokenUnit::from_tokens(10.0));
    let bob = env.create_account(2, TokenUnit::from_tokens(10.0));

    // Deploy token
    let init_args = ArgsBuilder::new().add_u64(1_000).build();
    let token = env
        .deploy_contract(alice, 0, TOKEN_WASM, init_args)
        .unwrap();

    let balance_after_deploy = env.get_balance(&alice);
    let nonce_after_deploy = env.get_nonce(&alice);

    // Try to transfer more tokens than Alice has (execution will fail)
    let transfer_args = ArgsBuilder::new()
        .add_address(&bob)
        .add_u64(10_000) // Alice only has 1000!
        .build();

    let result = env.call_contract(alice, 1, token, "transfer", transfer_args);

    // Transaction should be accepted (valid nonce, sufficient gas balance)
    // But execution fails
    assert!(
        result.is_ok(),
        "Valid transaction should return Ok(receipt)"
    );

    // Nonce SHOULD increment (transaction was valid)
    assert_eq!(
        env.get_nonce(&alice),
        nonce_after_deploy + 1,
        "Nonce should increment for valid tx even if execution fails"
    );

    // Gas SHOULD be charged
    let balance_after_failed_tx = env.get_balance(&alice);
    assert!(
        balance_after_failed_tx < balance_after_deploy,
        "Gas should be charged even for failed execution"
    );

    // Bob should not receive tokens
    let bob_token_balance = env
        .get_storage_u64(token, &get_balance_key(&bob))
        .unwrap_or(0);
    assert_eq!(
        bob_token_balance, 0,
        "Failed transfer should not move tokens"
    );
}
