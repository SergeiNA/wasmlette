//! Unit tests for executor transfer functionality
//!
//! Tests the low-level execute_transfer method in ContractExecutor

use std::cell::RefCell;
use std::rc::Rc;
use wasmlette_blockchain::transaction::{Transaction, TransactionKind};
use wasmlette_blockchain::{Address, State};
use wasmlette_runtime::{ContractExecutor, TokenUnit};

#[test]
fn test_execute_transfer_success() {
    let executor = ContractExecutor::new().unwrap();
    let state = Rc::new(RefCell::new(State::new()));

    let from = Address::from_slice(&[1u8; Address::LENGTH]);
    let to = Address::from_slice(&[2u8; Address::LENGTH]);

    // Set up initial balances
    state
        .borrow_mut()
        .set_balance(from, TokenUnit::from_tokens(10.0));
    state
        .borrow_mut()
        .set_balance(to, TokenUnit::from_tokens(5.0));

    let tx = Transaction::new(
        from,
        0,
        TransactionKind::Transfer {
            to,
            amount: TokenUnit::from_tokens(2.0),
        },
        100_000,
        1,
    );

    let receipt = executor.execute_transaction(state.clone(), &tx).unwrap();

    // Verify success
    assert!(receipt.success, "Transfer should succeed");
    assert_eq!(receipt.gas_used, 11_000); // Standard transfer cost

    // Verify balances
    let from_balance = state.borrow().get_balance(&from);
    let to_balance = state.borrow().get_balance(&to);

    // From: 10.0 - 2.0 (transfer) - 0.011 (gas) = 7.989
    assert_eq!(from_balance, TokenUnit::from_tokens(7.989));

    // To: 5.0 + 2.0 = 7.0
    assert_eq!(to_balance, TokenUnit::from_tokens(7.0));

    // Nonce should increment
    assert_eq!(state.borrow().get_nonce(&from), 1);
}

#[test]
fn test_execute_transfer_insufficient_balance() {
    let executor = ContractExecutor::new().unwrap();
    let state = Rc::new(RefCell::new(State::new()));

    let from = Address::from_slice(&[1u8; Address::LENGTH]);
    let to = Address::from_slice(&[2u8; Address::LENGTH]);

    // From has only 1 token
    state
        .borrow_mut()
        .set_balance(from, TokenUnit::from_tokens(1.0));
    state
        .borrow_mut()
        .set_balance(to, TokenUnit::from_tokens(5.0));

    let tx = Transaction::new(
        from,
        0,
        TransactionKind::Transfer {
            to,
            amount: TokenUnit::from_tokens(5.0), // More than from has!
        },
        100_000,
        1,
    );

    let receipt = executor.execute_transaction(state.clone(), &tx).unwrap();

    // Transaction is valid but execution fails
    assert!(!receipt.success, "Transfer should fail");
    assert!(receipt.gas_used > 0, "Gas should be consumed");
    assert!(receipt.error_message.is_some());

    // From balance decreased by gas only
    let from_balance = state.borrow().get_balance(&from);
    assert!(from_balance < TokenUnit::from_tokens(1.0), "Gas charged");

    // To balance unchanged
    assert_eq!(state.borrow().get_balance(&to), TokenUnit::from_tokens(5.0));

    // Nonce should still increment (valid transaction)
    assert_eq!(state.borrow().get_nonce(&from), 1);
}

#[test]
fn test_execute_transfer_zero_amount() {
    let executor = ContractExecutor::new().unwrap();
    let state = Rc::new(RefCell::new(State::new()));

    let from = Address::from_slice(&[1u8; Address::LENGTH]);
    let to = Address::from_slice(&[2u8; Address::LENGTH]);

    state
        .borrow_mut()
        .set_balance(from, TokenUnit::from_tokens(10.0));
    state
        .borrow_mut()
        .set_balance(to, TokenUnit::from_tokens(5.0));

    let tx = Transaction::new(
        from,
        0,
        TransactionKind::Transfer { to, amount: 0 },
        100_000,
        1,
    );

    let receipt = executor.execute_transaction(state.clone(), &tx).unwrap();

    // Zero transfer should succeed
    assert!(receipt.success, "Zero transfer should succeed");

    // Balances: from pays gas, to unchanged
    let from_balance = state.borrow().get_balance(&from);
    let to_balance = state.borrow().get_balance(&to);

    assert!(from_balance < TokenUnit::from_tokens(10.0), "From pays gas");
    assert_eq!(to_balance, TokenUnit::from_tokens(5.0), "To unchanged");

    // Nonce increments
    assert_eq!(state.borrow().get_nonce(&from), 1);
}

#[test]
fn test_execute_transfer_to_self() {
    let executor = ContractExecutor::new().unwrap();
    let state = Rc::new(RefCell::new(State::new()));

    let address = Address::from_slice(&[1u8; Address::LENGTH]);

    state
        .borrow_mut()
        .set_balance(address, TokenUnit::from_tokens(10.0));

    let tx = Transaction::new(
        address,
        0,
        TransactionKind::Transfer {
            to: address,
            amount: TokenUnit::from_tokens(3.0),
        },
        100_000,
        1,
    );

    let receipt = executor.execute_transaction(state.clone(), &tx).unwrap();

    // Self-transfer should succeed
    assert!(receipt.success, "Self-transfer should succeed");

    // Balance: only pays gas (transfer cancels out)
    let balance = state.borrow().get_balance(&address);
    assert!(balance < TokenUnit::from_tokens(10.0), "Pays gas");
    assert!(
        balance > TokenUnit::from_tokens(9.9),
        "Only pays gas, not transfer"
    );

    // Nonce increments
    assert_eq!(state.borrow().get_nonce(&address), 1);
}

#[test]
fn test_execute_transfer_invalid_nonce() {
    let executor = ContractExecutor::new().unwrap();
    let state = Rc::new(RefCell::new(State::new()));

    let from = Address::from_slice(&[1u8; Address::LENGTH]);
    let to = Address::from_slice(&[2u8; Address::LENGTH]);

    state
        .borrow_mut()
        .set_balance(from, TokenUnit::from_tokens(10.0));

    // Wrong nonce (expected 0, providing 5)
    let tx = Transaction::new(
        from,
        5, // Wrong!
        TransactionKind::Transfer {
            to,
            amount: TokenUnit::from_tokens(2.0),
        },
        100_000,
        1,
    );

    let result = executor.execute_transaction(state.clone(), &tx);

    // Should return Err for invalid nonce
    assert!(result.is_err(), "Invalid nonce should return Err");
    assert!(result.unwrap_err().to_string().contains("Invalid nonce"));

    // State unchanged
    assert_eq!(
        state.borrow().get_balance(&from),
        TokenUnit::from_tokens(10.0)
    );
    assert_eq!(state.borrow().get_nonce(&from), 0);
}

#[test]
fn test_execute_transfer_insufficient_gas_balance() {
    let executor = ContractExecutor::new().unwrap();
    let state = Rc::new(RefCell::new(State::new()));

    let from = Address::from_slice(&[1u8; Address::LENGTH]);
    let to = Address::from_slice(&[2u8; Address::LENGTH]);

    // Very low balance - not enough for gas
    state
        .borrow_mut()
        .set_balance(from, TokenUnit::from_tokens(0.0001));

    let tx = Transaction::new(
        from,
        0,
        TransactionKind::Transfer {
            to,
            amount: TokenUnit::from_tokens(0.00001),
        },
        1_000_000, // Needs 1 token for gas
        1,
    );

    let result = executor.execute_transaction(state.clone(), &tx);

    // Should return Err for insufficient balance
    assert!(
        result.is_err(),
        "Insufficient gas balance should return Err"
    );
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Insufficient balance"));

    // State unchanged
    assert_eq!(
        state.borrow().get_balance(&from),
        TokenUnit::from_tokens(0.0001)
    );
    assert_eq!(state.borrow().get_nonce(&from), 0);
}

#[test]
fn test_execute_transfer_gas_refund() {
    let executor = ContractExecutor::new().unwrap();
    let state = Rc::new(RefCell::new(State::new()));

    let from = Address::from_slice(&[1u8; Address::LENGTH]);
    let to = Address::from_slice(&[2u8; Address::LENGTH]);

    state
        .borrow_mut()
        .set_balance(from, TokenUnit::from_tokens(10.0));

    let tx = Transaction::new(
        from,
        0,
        TransactionKind::Transfer {
            to,
            amount: TokenUnit::from_tokens(1.0),
        },
        100_000, // High gas limit
        1,
    );

    let receipt = executor.execute_transaction(state.clone(), &tx).unwrap();

    // Only actual gas used should be charged, not full limit
    assert!(receipt.success);
    assert_eq!(receipt.gas_used, 11_000); // Actual transfer cost
    assert!(receipt.gas_used < 100_000, "Should refund unused gas");

    // Verify refund applied
    let from_balance = state.borrow().get_balance(&from);

    // Cost = 11_000 * 1 = 11_000 micro-tokens = 0.011 tokens
    // Balance = 10.0 - 1.0 (transfer) - 0.011 (gas) = 8.989
    assert_eq!(from_balance, TokenUnit::from_tokens(8.989));
}

#[test]
fn test_execute_multiple_transfers_sequential() {
    let executor = ContractExecutor::new().unwrap();
    let state = Rc::new(RefCell::new(State::new()));

    let alice = Address::from_slice(&[1u8; Address::LENGTH]);
    let bob = Address::from_slice(&[2u8; Address::LENGTH]);

    state
        .borrow_mut()
        .set_balance(alice, TokenUnit::from_tokens(10.0));

    // Transfer 1: Alice -> Bob (nonce 0)
    let tx1 = Transaction::new(
        alice,
        0,
        TransactionKind::Transfer {
            to: bob,
            amount: TokenUnit::from_tokens(2.0),
        },
        100_000,
        1,
    );

    let receipt1 = executor.execute_transaction(state.clone(), &tx1).unwrap();
    assert!(receipt1.success);
    assert_eq!(state.borrow().get_nonce(&alice), 1);

    // Transfer 2: Alice -> Bob (nonce 1)
    let tx2 = Transaction::new(
        alice,
        1,
        TransactionKind::Transfer {
            to: bob,
            amount: TokenUnit::from_tokens(3.0),
        },
        100_000,
        1,
    );

    let receipt2 = executor.execute_transaction(state.clone(), &tx2).unwrap();
    assert!(receipt2.success);
    assert_eq!(state.borrow().get_nonce(&alice), 2);

    // Bob should have 5.0 tokens
    assert_eq!(
        state.borrow().get_balance(&bob),
        TokenUnit::from_tokens(5.0)
    );

    // Alice should have: 10.0 - 5.0 (transfers) - ~0.022 (gas) ≈ 4.978
    let alice_balance = state.borrow().get_balance(&alice);
    assert!(alice_balance < TokenUnit::from_tokens(5.0));
    assert!(alice_balance > TokenUnit::from_tokens(4.9));
}
