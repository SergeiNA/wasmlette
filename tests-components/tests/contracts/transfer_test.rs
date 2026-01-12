//! Native token transfer integration tests-integration
//!
//! Tests for TransactionKind::Transfer - direct transfers of native blockchain tokens
//! between accounts without involving smart contracts.

use crate::common::TestEnv;
use wasmlette_runtime::TokenUnit;

#[test]
fn test_basic_native_transfer() {
    let env = TestEnv::new();

    // Create two accounts
    let alice = env.create_account(1, TokenUnit::from_tokens(10.0));
    let bob = env.create_account(2, TokenUnit::from_tokens(5.0));

    let alice_initial = env.get_balance(&alice);
    let bob_initial = env.get_balance(&bob);
    let alice_nonce = env.get_nonce(&alice);

    // Alice transfers 2 tokens to Bob
    let result = env.transfer(alice, 0, bob, TokenUnit::from_tokens(2.0));

    assert!(result.is_ok(), "Transfer should succeed");

    // Verify balances changed (accounting for gas cost)
    let alice_final = env.get_balance(&alice);
    let bob_final = env.get_balance(&bob);

    // Alice should have less than initial - transfer amount (due to gas)
    assert!(alice_final < alice_initial - TokenUnit::from_tokens(2.0));

    // Bob should have exactly initial + transfer amount (no gas cost for recipient)
    assert_eq!(bob_final, bob_initial + TokenUnit::from_tokens(2.0));

    // Nonce should increment
    assert_eq!(env.get_nonce(&alice), alice_nonce + 1);
}

#[test]
fn test_transfer_insufficient_balance() {
    let env = TestEnv::new();

    // Alice has 1 token, tries to send 5 tokens
    let alice = env.create_account(1, TokenUnit::from_tokens(1.0));
    let bob = env.create_account(2, TokenUnit::from_tokens(5.0));

    let alice_initial = env.get_balance(&alice);
    let bob_initial = env.get_balance(&bob);
    let alice_nonce = env.get_nonce(&alice);

    // Try to transfer more than Alice has
    let result = env.transfer(alice, 0, bob, TokenUnit::from_tokens(5.0));

    // Should return Ok(receipt) but with success=false (valid tx, failed execution)
    assert!(
        result.is_ok(),
        "Should return Ok(receipt) for valid transaction"
    );

    // Balances: Alice pays gas, Bob receives nothing
    let alice_final = env.get_balance(&alice);
    let bob_final = env.get_balance(&bob);

    assert!(
        alice_final < alice_initial,
        "Alice should pay gas even on failure"
    );
    assert_eq!(bob_final, bob_initial, "Bob should receive nothing");

    // Nonce should still increment (transaction was valid)
    assert_eq!(env.get_nonce(&alice), alice_nonce + 1);
}

#[test]
fn test_transfer_zero_amount() {
    let env = TestEnv::new();

    let alice = env.create_account(1, TokenUnit::from_tokens(10.0));
    let bob = env.create_account(2, TokenUnit::from_tokens(5.0));

    let alice_initial = env.get_balance(&alice);
    let bob_initial = env.get_balance(&bob);
    let alice_nonce = env.get_nonce(&alice);

    // Transfer 0 tokens
    let result = env.transfer(alice, 0, bob, 0);

    // Should succeed (valid transaction)
    assert!(result.is_ok(), "Zero transfer should be valid");

    // Alice pays gas, Bob gets nothing
    let alice_final = env.get_balance(&alice);
    let bob_final = env.get_balance(&bob);

    assert!(alice_final < alice_initial, "Alice should pay gas");
    assert_eq!(bob_final, bob_initial, "Bob balance unchanged");

    // Nonce increments
    assert_eq!(env.get_nonce(&alice), alice_nonce + 1);
}

#[test]
fn test_transfer_to_self() {
    let env = TestEnv::new();

    let alice = env.create_account(1, TokenUnit::from_tokens(10.0));

    let alice_initial = env.get_balance(&alice);
    let alice_nonce = env.get_nonce(&alice);

    // Alice transfers to herself
    let result = env.transfer(alice, 0, alice, TokenUnit::from_tokens(2.0));

    // Should succeed (valid transaction)
    assert!(result.is_ok(), "Self-transfer should be valid");

    // Alice only pays gas (transfer to self is net zero)
    let alice_final = env.get_balance(&alice);

    // Balance should decrease only by gas cost
    assert!(alice_final < alice_initial, "Should pay gas");
    assert!(
        alice_final > alice_initial - TokenUnit::from_tokens(0.1),
        "Should only pay gas, not transfer amount"
    );

    // Nonce increments
    assert_eq!(env.get_nonce(&alice), alice_nonce + 1);
}

#[test]
fn test_transfer_invalid_nonce_rejected() {
    let env = TestEnv::new();

    let alice = env.create_account(1, TokenUnit::from_tokens(10.0));
    let bob = env.create_account(2, TokenUnit::from_tokens(5.0));

    let alice_initial = env.get_balance(&alice);
    let bob_initial = env.get_balance(&bob);
    let alice_nonce = env.get_nonce(&alice);

    // Try transfer with wrong nonce (expected 0, providing 5)
    let result = env.transfer(alice, 5, bob, TokenUnit::from_tokens(2.0));

    // Should return Err for invalid nonce
    assert!(result.is_err(), "Invalid nonce should return Err");
    assert!(result.unwrap_err().contains("Invalid nonce"));

    // No state changes
    assert_eq!(
        env.get_balance(&alice),
        alice_initial,
        "Alice balance unchanged"
    );
    assert_eq!(env.get_balance(&bob), bob_initial, "Bob balance unchanged");
    assert_eq!(env.get_nonce(&alice), alice_nonce, "Nonce unchanged");
}

#[test]
fn test_transfer_insufficient_gas_balance_rejected() {
    let env = TestEnv::new();

    // Alice has barely any tokens
    let alice = env.create_account(1, TokenUnit::from_tokens(0.0001));
    let bob = env.create_account(2, TokenUnit::from_tokens(5.0));

    let alice_initial = env.get_balance(&alice);
    let bob_initial = env.get_balance(&bob);
    let alice_nonce = env.get_nonce(&alice);

    // Try to transfer (won't have enough for gas)
    let result = env.transfer(alice, 0, bob, TokenUnit::from_tokens(0.00001));

    // Should return Err for insufficient balance
    assert!(
        result.is_err(),
        "Insufficient gas balance should return Err"
    );
    assert!(result.unwrap_err().contains("Insufficient balance"));

    // No state changes
    assert_eq!(env.get_balance(&alice), alice_initial);
    assert_eq!(env.get_balance(&bob), bob_initial);
    assert_eq!(env.get_nonce(&alice), alice_nonce);
}

#[test]
fn test_multiple_transfers_with_nonces() {
    let env = TestEnv::new();

    let alice = env.create_account(1, TokenUnit::from_tokens(20.0));
    let bob = env.create_account(2, TokenUnit::from_tokens(10.0));
    let charlie = env.create_account(3, TokenUnit::from_tokens(5.0));

    // Alice's initial state
    let alice_initial = env.get_balance(&alice);
    assert_eq!(env.get_nonce(&alice), 0);

    // Transfer 1: Alice -> Bob (nonce 0)
    let result1 = env.transfer(alice, 0, bob, TokenUnit::from_tokens(3.0));
    assert!(result1.is_ok());
    assert_eq!(env.get_nonce(&alice), 1);

    let bob_after_1 = env.get_balance(&bob);
    assert_eq!(bob_after_1, TokenUnit::from_tokens(13.0));

    // Transfer 2: Alice -> Charlie (nonce 1)
    let result2 = env.transfer(alice, 1, charlie, TokenUnit::from_tokens(2.0));
    assert!(result2.is_ok());
    assert_eq!(env.get_nonce(&alice), 2);

    let charlie_after_2 = env.get_balance(&charlie);
    assert_eq!(charlie_after_2, TokenUnit::from_tokens(7.0));

    // Transfer 3: Alice -> Bob again (nonce 2)
    let result3 = env.transfer(alice, 2, bob, TokenUnit::from_tokens(1.0));
    assert!(result3.is_ok());
    assert_eq!(env.get_nonce(&alice), 3);

    let bob_after_3 = env.get_balance(&bob);
    assert_eq!(bob_after_3, TokenUnit::from_tokens(14.0));

    // Verify Alice paid for all transfers + gas
    let alice_final = env.get_balance(&alice);
    let total_transferred = TokenUnit::from_tokens(6.0); // 3 + 2 + 1
    assert!(alice_final < alice_initial - total_transferred); // Less due to gas
    assert!(alice_final > alice_initial - total_transferred - TokenUnit::from_tokens(0.5));
    // Not too much less
}

#[test]
fn test_transfer_with_exact_amount_for_gas() {
    let env = TestEnv::new();

    // Alice has exactly enough for gas but not for transfer amount
    // Gas = 1,000,000 * 1 = 1,000,000 units = 1 token
    // So Alice needs > 1 token to pass validation, then fails execution
    let alice = env.create_account(1, TokenUnit::from_tokens(1.5));
    let bob = env.create_account(2, TokenUnit::from_tokens(5.0));

    let alice_nonce = env.get_nonce(&alice);

    // Try to transfer 5 tokens when Alice only has 1.5
    // After gas (1 token) is reserved, she has 0.5 left, can't send 5
    let result = env.transfer(alice, 0, bob, TokenUnit::from_tokens(5.0));

    // Should succeed as valid transaction but execution fails
    assert!(result.is_ok(), "Valid tx should return Ok(receipt)");

    // Nonce increments, gas charged
    assert_eq!(env.get_nonce(&alice), alice_nonce + 1);
    assert!(
        env.get_balance(&alice) < TokenUnit::from_tokens(1.5),
        "Gas charged"
    );
    assert_eq!(
        env.get_balance(&bob),
        TokenUnit::from_tokens(5.0),
        "Bob receives nothing"
    );
}

#[test]
fn test_bidirectional_transfers() {
    let env = TestEnv::new();

    let alice = env.create_account(1, TokenUnit::from_tokens(10.0));
    let bob = env.create_account(2, TokenUnit::from_tokens(10.0));

    // Alice -> Bob
    let result1 = env.transfer(alice, 0, bob, TokenUnit::from_tokens(3.0));
    assert!(result1.is_ok());

    // Bob -> Alice
    let result2 = env.transfer(bob, 0, alice, TokenUnit::from_tokens(2.0));
    assert!(result2.is_ok());

    // Both nonces should be 1
    assert_eq!(env.get_nonce(&alice), 1);
    assert_eq!(env.get_nonce(&alice), 1);

    // Verify final balances (accounting for gas on both)
    let alice_final = env.get_balance(&alice);
    let bob_final = env.get_balance(&bob);

    // Alice: started 10, sent 3, received 2, paid gas = ~9 - gas
    // Bob: started 10, received 3, sent 2, paid gas = ~11 - gas

    // Alice should have less than initial (net -1 - gas)
    assert!(alice_final < TokenUnit::from_tokens(10.0));
    assert!(alice_final > TokenUnit::from_tokens(8.5)); // Not too much gas

    // Bob should have more than initial (net +1 - gas)
    assert!(bob_final > TokenUnit::from_tokens(10.0));
    assert!(bob_final < TokenUnit::from_tokens(11.5));
}
