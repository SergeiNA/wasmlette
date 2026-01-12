//! Node-level transfer integration tests-integration
//!
//! Tests transfer functionality at the node level, including:
//! - Transaction execution
//! - Block creation
//! - Chain updates
//! - State persistence

use wasmlette_node::WasmletteNode;
use wasmlette_runtime::TokenUnit;

#[test]
fn test_node_basic_transfer() {
    let mut node = WasmletteNode::new().unwrap();

    let alice = node.create_account(1, TokenUnit::from_tokens(10.0));
    let bob = node.create_account(2, TokenUnit::from_tokens(5.0));

    let alice_initial = node.get_balance(&alice);
    let bob_initial = node.get_balance(&bob);
    let alice_nonce = node.get_nonce(&alice);

    // Transfer 2 tokens from Alice to Bob
    let receipt = node
        .transfer(alice, bob, TokenUnit::from_tokens(2.0))
        .unwrap();

    // Verify receipt
    assert!(receipt.success, "Transfer should succeed");
    assert!(receipt.gas_used > 0, "Gas should be consumed");

    // Verify balances
    let alice_final = node.get_balance(&alice);
    let bob_final = node.get_balance(&bob);

    // Alice pays: transfer + gas
    assert!(alice_final < alice_initial - TokenUnit::from_tokens(2.0));

    // Bob receives exact amount (no gas for recipient)
    assert_eq!(bob_final, bob_initial + TokenUnit::from_tokens(2.0));

    // Nonce increments
    assert_eq!(node.get_nonce(&alice), alice_nonce + 1);

    // Verify blockchain state
    assert_eq!(node.height(), 1, "Should have 1 block after genesis");
}

#[test]
fn test_node_transfer_insufficient_balance() {
    let mut node = WasmletteNode::new().unwrap();

    let alice = node.create_account(1, TokenUnit::from_tokens(1.0));
    let bob = node.create_account(2, TokenUnit::from_tokens(5.0));

    let alice_initial = node.get_balance(&alice);
    let bob_initial = node.get_balance(&bob);
    let alice_nonce = node.get_nonce(&alice);

    // Try to transfer more than Alice has
    let receipt = node
        .transfer(alice, bob, TokenUnit::from_tokens(5.0))
        .unwrap();

    // Transaction valid but execution fails
    assert!(!receipt.success, "Transfer should fail");
    assert!(receipt.error_message.is_some());

    // Alice pays gas only
    assert!(node.get_balance(&alice) < alice_initial, "Alice pays gas");

    // Bob receives nothing
    assert_eq!(node.get_balance(&bob), bob_initial);

    // Nonce still increments (valid transaction)
    assert_eq!(node.get_nonce(&alice), alice_nonce + 1);

    // Block still created
    assert_eq!(node.height(), 1);
}

#[test]
fn test_node_transfer_zero_amount() {
    let mut node = WasmletteNode::new().unwrap();

    let alice = node.create_account(1, TokenUnit::from_tokens(10.0));
    let bob = node.create_account(2, TokenUnit::from_tokens(5.0));

    let receipt = node.transfer(alice, bob, 0).unwrap();

    // Zero transfer should succeed
    assert!(receipt.success, "Zero transfer should succeed");

    // Alice pays gas only
    assert!(node.get_balance(&alice) < TokenUnit::from_tokens(10.0));

    // Bob unchanged
    assert_eq!(node.get_balance(&bob), TokenUnit::from_tokens(5.0));

    // Block created
    assert_eq!(node.height(), 1);
}

#[test]
fn test_node_transfer_to_self() {
    let mut node = WasmletteNode::new().unwrap();

    let alice = node.create_account(1, TokenUnit::from_tokens(10.0));

    let alice_initial = node.get_balance(&alice);

    let receipt = node
        .transfer(alice, alice, TokenUnit::from_tokens(3.0))
        .unwrap();

    // Self-transfer should succeed
    assert!(receipt.success, "Self-transfer should succeed");

    // Only pays gas (transfer cancels out)
    let alice_final = node.get_balance(&alice);
    assert!(alice_final < alice_initial, "Pays gas");
    assert!(
        alice_final > alice_initial - TokenUnit::from_tokens(0.1),
        "Only pays gas"
    );

    // Block created
    assert_eq!(node.height(), 1);
}

#[test]
fn test_node_multiple_transfers() {
    let mut node = WasmletteNode::new().unwrap();

    let alice = node.create_account(1, TokenUnit::from_tokens(20.0));
    let bob = node.create_account(2, TokenUnit::from_tokens(10.0));
    let charlie = node.create_account(3, TokenUnit::from_tokens(5.0));

    // Transfer 1: Alice -> Bob
    let receipt1 = node
        .transfer(alice, bob, TokenUnit::from_tokens(3.0))
        .unwrap();
    assert!(receipt1.success);
    assert_eq!(node.get_nonce(&alice), 1);
    assert_eq!(node.height(), 1);

    // Transfer 2: Alice -> Charlie
    let receipt2 = node
        .transfer(alice, charlie, TokenUnit::from_tokens(2.0))
        .unwrap();
    assert!(receipt2.success);
    assert_eq!(node.get_nonce(&alice), 2);
    assert_eq!(node.height(), 2);

    // Transfer 3: Bob -> Charlie
    let receipt3 = node
        .transfer(bob, charlie, TokenUnit::from_tokens(1.0))
        .unwrap();
    assert!(receipt3.success);
    assert_eq!(node.get_nonce(&bob), 1);
    assert_eq!(node.height(), 3);

    // Verify final balances
    let bob_balance = node.get_balance(&bob);
    let charlie_balance = node.get_balance(&charlie);

    // Bob: 10 + 3 (from Alice) - 1 (to Charlie) - gas ≈ 11.98x
    assert!(bob_balance > TokenUnit::from_tokens(11.9));
    assert!(bob_balance < TokenUnit::from_tokens(12.0));

    // Charlie: 5 + 2 (from Alice) + 1 (from Bob) = 8
    assert_eq!(charlie_balance, TokenUnit::from_tokens(8.0));
}

#[test]
fn test_node_transfer_creates_blocks() {
    let mut node = WasmletteNode::new().unwrap();

    let alice = node.create_account(1, TokenUnit::from_tokens(10.0));
    let bob = node.create_account(2, TokenUnit::from_tokens(5.0));

    assert_eq!(node.height(), 0, "Should start with genesis only");

    // Each transfer creates a new block
    node.transfer(alice, bob, TokenUnit::from_tokens(1.0))
        .unwrap();
    assert_eq!(node.height(), 1);

    node.transfer(alice, bob, TokenUnit::from_tokens(1.0))
        .unwrap();
    assert_eq!(node.height(), 2);

    node.transfer(alice, bob, TokenUnit::from_tokens(1.0))
        .unwrap();
    assert_eq!(node.height(), 3);

    // Verify blocks can be retrieved
    assert!(node.get_block(0).is_some(), "Genesis block exists");
    assert!(node.get_block(1).is_some(), "Block 1 exists");
    assert!(node.get_block(2).is_some(), "Block 2 exists");
    assert!(node.get_block(3).is_some(), "Block 3 exists");
    assert!(node.get_block(4).is_none(), "Block 4 doesn't exist");
}

#[test]
fn test_node_transfer_chain_continuity() {
    let mut node = WasmletteNode::new().unwrap();

    let alice = node.create_account(1, TokenUnit::from_tokens(10.0));
    let bob = node.create_account(2, TokenUnit::from_tokens(5.0));

    // Make transfers and verify chain continuity
    node.transfer(alice, bob, TokenUnit::from_tokens(1.0))
        .unwrap();
    node.transfer(alice, bob, TokenUnit::from_tokens(1.0))
        .unwrap();

    let block1 = node.get_block(1).unwrap();
    let block2 = node.get_block(2).unwrap();

    // Block 2 should reference block 1
    assert_eq!(block2.parent_hash, block1.hash());
    assert_eq!(block2.number, block1.number + 1);
    assert!(block2.timestamp >= block1.timestamp);
}

#[test]
fn test_node_transfer_bidirectional() {
    let mut node = WasmletteNode::new().unwrap();

    let alice = node.create_account(1, TokenUnit::from_tokens(10.0));
    let bob = node.create_account(2, TokenUnit::from_tokens(10.0));

    // Alice -> Bob
    node.transfer(alice, bob, TokenUnit::from_tokens(3.0))
        .unwrap();

    // Bob -> Alice
    node.transfer(bob, alice, TokenUnit::from_tokens(2.0))
        .unwrap();

    // Both should have incremented nonces
    assert_eq!(node.get_nonce(&alice), 1);
    assert_eq!(node.get_nonce(&bob), 1);

    // Verify balances (accounting for gas)
    let alice_balance = node.get_balance(&alice);
    let bob_balance = node.get_balance(&bob);

    // Alice: 10 - 3 (sent) + 2 (received) - gas ≈ 8.98x
    assert!(alice_balance < TokenUnit::from_tokens(9.0));
    assert!(alice_balance > TokenUnit::from_tokens(8.9));

    // Bob: 10 + 3 (received) - 2 (sent) - gas ≈ 10.98x
    assert!(bob_balance < TokenUnit::from_tokens(11.0));
    assert!(bob_balance > TokenUnit::from_tokens(10.9));
}

#[test]
fn test_node_transfer_with_nonce_tracking() {
    let mut node = WasmletteNode::new().unwrap();

    let alice = node.create_account(1, TokenUnit::from_tokens(20.0));
    let bob = node.create_account(2, TokenUnit::from_tokens(5.0));

    // Sequential transfers from Alice
    for i in 0..5 {
        let receipt = node
            .transfer(alice, bob, TokenUnit::from_tokens(1.0))
            .unwrap();

        assert!(receipt.success);
        assert_eq!(node.get_nonce(&alice), i + 1, "Nonce should increment");
    }

    // Bob should have received 5 tokens
    assert_eq!(
        node.get_balance(&bob),
        TokenUnit::from_tokens(10.0),
        "Bob should have 10 tokens"
    );

    // Chain height should be 5 (5 blocks after genesis)
    assert_eq!(node.height(), 5);
}

#[test]
fn test_node_transfer_state_persistence() {
    let mut node = WasmletteNode::new().unwrap();

    let alice = node.create_account(1, TokenUnit::from_tokens(10.0));
    let bob = node.create_account(2, TokenUnit::from_tokens(5.0));

    // Make a transfer
    node.transfer(alice, bob, TokenUnit::from_tokens(2.0))
        .unwrap();

    // State changes should persist
    let alice_balance = node.get_balance(&alice);
    let bob_balance = node.get_balance(&bob);
    let alice_nonce = node.get_nonce(&alice);

    // Make another transfer
    node.transfer(alice, bob, TokenUnit::from_tokens(1.0))
        .unwrap();

    // Second transfer should build on first transfer's state
    let alice_balance_2 = node.get_balance(&alice);
    let bob_balance_2 = node.get_balance(&bob);

    assert!(alice_balance_2 < alice_balance, "Alice balance decreases");
    assert!(bob_balance_2 > bob_balance, "Bob balance increases");
    assert_eq!(node.get_nonce(&alice), alice_nonce + 1, "Nonce increments");
}
