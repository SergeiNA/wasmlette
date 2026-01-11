//! Node-level contract integration tests
//!
//! Tests contract deployment and execution at the node level, including:
//! - Contract deployment with block creation
//! - Contract method calls with state changes
//! - Gas mechanics and nonce tracking
//! - Blockchain state persistence
//! - Multi-contract interactions

use wasmlette_node::WasmletteNode;
use wasmlette_runtime::TokenUnit;

const TOKEN_WASM: &[u8] =
    include_bytes!("../../../target/wasm32-unknown-unknown/release/simple_token.wasm");

const DEFAULT_GAS_LIMIT: u64 = 1_000_000;

fn get_balance_key(address: &wasmlette_blockchain::Address) -> Vec<u8> {
    let mut key = b"balance".to_vec();
    key.extend_from_slice(address.as_bytes());
    key
}

fn encode_u64(value: u64) -> Vec<u8> {
    value.to_le_bytes().to_vec()
}

fn encode_address(address: &wasmlette_blockchain::Address) -> Vec<u8> {
    address.as_bytes().to_vec()
}

fn build_init_args(initial_supply: u64) -> Vec<u8> {
    encode_u64(initial_supply)
}

fn build_transfer_args(to: &wasmlette_blockchain::Address, amount: u64) -> Vec<u8> {
    let mut args = encode_address(to);
    args.extend_from_slice(&encode_u64(amount));
    args
}

#[test]
fn test_node_contract_deploy() {
    let mut node = WasmletteNode::new().unwrap();

    let deployer = node.create_account(1, TokenUnit::from_tokens(10.0));
    let initial_balance = node.get_balance(&deployer);

    // Deploy token contract
    let init_supply = 1_000_000u64;
    let init_args = build_init_args(init_supply);

    let receipt = node
        .deploy_contract(deployer, TOKEN_WASM.to_vec(), init_args, DEFAULT_GAS_LIMIT)
        .unwrap();

    // Verify deployment success
    assert!(receipt.success, "Deploy should succeed");
    assert!(
        receipt.contract_address.is_some(),
        "Should have contract address"
    );
    assert!(receipt.gas_used > 0, "Gas should be consumed");

    let contract = receipt.contract_address.unwrap();

    // Verify contract exists
    assert!(node.contract_exists(&contract), "Contract should exist");

    // Verify deployer paid gas
    let final_balance = node.get_balance(&deployer);
    assert!(final_balance < initial_balance, "Deployer should pay gas");

    // Verify nonce incremented
    assert_eq!(node.get_nonce(&deployer), 1, "Nonce should increment");

    // Verify block was created
    assert_eq!(node.height(), 1, "Should create block 1");

    // Verify block contains transaction
    let block = node.get_block(1).unwrap();
    assert_eq!(
        block.transactions.len(),
        1,
        "Block should have 1 transaction"
    );

    // Verify contract storage (total supply)
    let supply_key = b"total_supply".to_vec();
    let supply_bytes = node.get_storage(contract, supply_key).unwrap();
    let supply = u64::from_le_bytes(supply_bytes.try_into().unwrap());
    assert_eq!(supply, init_supply, "Total supply should match");
}

#[test]
fn test_node_contract_call() {
    let mut node = WasmletteNode::new().unwrap();

    let alice = node.create_account(1, TokenUnit::from_tokens(10.0));
    let bob = node.create_account(2, TokenUnit::from_tokens(10.0));

    // Deploy contract
    let init_supply = 1_000_000u64;
    let init_args = build_init_args(init_supply);

    let deploy_receipt = node
        .deploy_contract(alice, TOKEN_WASM.to_vec(), init_args, DEFAULT_GAS_LIMIT)
        .unwrap();

    let contract = deploy_receipt.contract_address.unwrap();
    assert_eq!(node.height(), 1, "Deploy creates block 1");

    let alice_balance_after_deploy = node.get_balance(&alice);

    // Call contract to transfer tokens
    let transfer_amount = 5000u64;
    let transfer_args = build_transfer_args(&bob, transfer_amount);

    let call_receipt = node
        .call_contract(
            alice,
            contract,
            "transfer".to_string(),
            transfer_args,
            DEFAULT_GAS_LIMIT,
        )
        .unwrap();

    // Verify call success
    assert!(call_receipt.success, "Call should succeed");
    assert!(call_receipt.gas_used > 0, "Gas should be consumed");

    // Verify nonce incremented (now 2: deploy + call)
    assert_eq!(node.get_nonce(&alice), 2, "Alice nonce should be 2");

    // Verify block was created
    assert_eq!(node.height(), 2, "Should create block 2");

    // Verify Alice paid gas for call
    let alice_final = node.get_balance(&alice);
    assert!(
        alice_final < alice_balance_after_deploy,
        "Alice pays gas for call"
    );

    // Verify token balances in storage
    let alice_token_balance = node
        .get_storage(contract, get_balance_key(&alice))
        .map(|bytes| u64::from_le_bytes(bytes.try_into().unwrap()))
        .unwrap();

    let bob_token_balance = node
        .get_storage(contract, get_balance_key(&bob))
        .map(|bytes| u64::from_le_bytes(bytes.try_into().unwrap()))
        .unwrap();

    assert_eq!(
        alice_token_balance,
        init_supply - transfer_amount,
        "Alice should have remaining tokens"
    );
    assert_eq!(
        bob_token_balance, transfer_amount,
        "Bob should receive tokens"
    );
}

#[test]
fn test_node_multiple_contract_calls() {
    let mut node = WasmletteNode::new().unwrap();

    let alice = node.create_account(1, TokenUnit::from_tokens(20.0));
    let bob = node.create_account(2, TokenUnit::from_tokens(20.0));
    let charlie = node.create_account(3, TokenUnit::from_tokens(20.0));

    // Deploy contract
    let init_supply = 10_000u64;
    let init_args = build_init_args(init_supply);

    let deploy_receipt = node
        .deploy_contract(alice, TOKEN_WASM.to_vec(), init_args, DEFAULT_GAS_LIMIT)
        .unwrap();

    let contract = deploy_receipt.contract_address.unwrap();
    assert_eq!(node.height(), 1, "Deploy creates block");

    // Alice -> Bob: 3000 tokens
    let args1 = build_transfer_args(&bob, 3000);
    let receipt1 = node
        .call_contract(
            alice,
            contract,
            "transfer".to_string(),
            args1,
            DEFAULT_GAS_LIMIT,
        )
        .unwrap();

    assert!(receipt1.success, "Transfer 1 should succeed");
    assert_eq!(node.get_nonce(&alice), 2, "Alice nonce: 2");
    assert_eq!(node.height(), 2, "Block 2 created");

    // Alice -> Charlie: 2000 tokens
    let args2 = build_transfer_args(&charlie, 2000);
    let receipt2 = node
        .call_contract(
            alice,
            contract,
            "transfer".to_string(),
            args2,
            DEFAULT_GAS_LIMIT,
        )
        .unwrap();

    assert!(receipt2.success, "Transfer 2 should succeed");
    assert_eq!(node.get_nonce(&alice), 3, "Alice nonce: 3");
    assert_eq!(node.height(), 3, "Block 3 created");

    // Bob -> Charlie: 1000 tokens
    let args3 = build_transfer_args(&charlie, 1000);
    let receipt3 = node
        .call_contract(
            bob,
            contract,
            "transfer".to_string(),
            args3,
            DEFAULT_GAS_LIMIT,
        )
        .unwrap();

    assert!(receipt3.success, "Transfer 3 should succeed");
    assert_eq!(node.get_nonce(&bob), 1, "Bob nonce: 1");
    assert_eq!(node.height(), 4, "Block 4 created");

    // Verify final token balances
    let alice_tokens = node
        .get_storage(contract, get_balance_key(&alice))
        .map(|b| u64::from_le_bytes(b.try_into().unwrap()))
        .unwrap();

    let bob_tokens = node
        .get_storage(contract, get_balance_key(&bob))
        .map(|b| u64::from_le_bytes(b.try_into().unwrap()))
        .unwrap();

    let charlie_tokens = node
        .get_storage(contract, get_balance_key(&charlie))
        .map(|b| u64::from_le_bytes(b.try_into().unwrap()))
        .unwrap();

    assert_eq!(alice_tokens, 5_000, "Alice: 10000 - 3000 - 2000 = 5000");
    assert_eq!(bob_tokens, 2_000, "Bob: 3000 - 1000 = 2000");
    assert_eq!(charlie_tokens, 3_000, "Charlie: 2000 + 1000 = 3000");

    // Verify native token gas costs
    let alice_native = node.get_balance(&alice);
    let alice_gas_paid = TokenUnit::from_tokens(20.0) - alice_native;
    assert!(alice_gas_paid > 0, "Alice paid gas for deploy + 2 calls");

    let bob_native = node.get_balance(&bob);
    let bob_gas_paid = TokenUnit::from_tokens(20.0) - bob_native;
    assert!(bob_gas_paid > 0, "Bob paid gas for 1 call");

    assert!(
        alice_gas_paid > bob_gas_paid,
        "Alice paid more gas (3 txs) than Bob (1 tx)"
    );

    // Charlie hasn't sent transactions
    assert_eq!(
        node.get_balance(&charlie),
        TokenUnit::from_tokens(20.0),
        "Charlie hasn't paid gas"
    );
}

#[test]
fn test_node_contract_deploy_insufficient_balance() {
    let mut node = WasmletteNode::new().unwrap();

    // Account with insufficient balance for deployment gas
    let deployer = node.create_account(1, TokenUnit::from_tokens(0.001));

    let initial_balance = node.get_balance(&deployer);
    let initial_nonce = node.get_nonce(&deployer);

    let init_args = build_init_args(1_000);

    // Try to deploy with insufficient balance
    let result = node.deploy_contract(deployer, TOKEN_WASM.to_vec(), init_args, DEFAULT_GAS_LIMIT);

    // Should fail
    assert!(
        result.is_err(),
        "Deploy should fail with insufficient balance"
    );

    // State should be unchanged
    assert_eq!(
        node.get_balance(&deployer),
        initial_balance,
        "Balance unchanged on rejection"
    );
    assert_eq!(
        node.get_nonce(&deployer),
        initial_nonce,
        "Nonce unchanged on rejection"
    );

    // No block should be created
    assert_eq!(node.height(), 0, "No block created on rejection");
}

#[test]
fn test_node_contract_call_with_insufficient_tokens() {
    let mut node = WasmletteNode::new().unwrap();

    let alice = node.create_account(1, TokenUnit::from_tokens(10.0));
    let bob = node.create_account(2, TokenUnit::from_tokens(10.0));

    // Deploy contract with small supply
    let init_supply = 1_000u64;
    let init_args = build_init_args(init_supply);

    let deploy_receipt = node
        .deploy_contract(alice, TOKEN_WASM.to_vec(), init_args, DEFAULT_GAS_LIMIT)
        .unwrap();

    let contract = deploy_receipt.contract_address.unwrap();

    let alice_balance_after_deploy = node.get_balance(&alice);
    let alice_nonce_after_deploy = node.get_nonce(&alice);

    // Try to transfer more than Alice has
    // Note: The simple_token contract returns an error code but doesn't fail execution
    let transfer_args = build_transfer_args(&bob, 10_000); // More than 1000!

    let call_receipt = node
        .call_contract(
            alice,
            contract,
            "transfer".to_string(),
            transfer_args,
            DEFAULT_GAS_LIMIT,
        )
        .unwrap();

    // Transaction is valid and execution succeeds (but returns error code)
    assert!(call_receipt.success, "Execution succeeds but returns error");

    // Nonce should increment (valid transaction)
    assert_eq!(
        node.get_nonce(&alice),
        alice_nonce_after_deploy + 1,
        "Nonce increments on valid tx"
    );

    // Gas should be charged
    let alice_final = node.get_balance(&alice);
    assert!(
        alice_final < alice_balance_after_deploy,
        "Gas charged for execution"
    );

    // Block should be created
    assert_eq!(node.height(), 2, "Block created");

    // Bob should not receive tokens (contract logic prevented transfer)
    let bob_tokens = node
        .get_storage(contract, get_balance_key(&bob))
        .map(|b| u64::from_le_bytes(b.try_into().unwrap()))
        .unwrap_or(0);

    assert_eq!(bob_tokens, 0, "Bob should not receive tokens");

    // Alice should still have all her tokens
    let alice_tokens = node
        .get_storage(contract, get_balance_key(&alice))
        .map(|b| u64::from_le_bytes(b.try_into().unwrap()))
        .unwrap();

    assert_eq!(alice_tokens, init_supply, "Alice keeps all tokens");
}

#[test]
fn test_node_contract_chain_continuity() {
    let mut node = WasmletteNode::new().unwrap();

    let alice = node.create_account(1, TokenUnit::from_tokens(10.0));

    // Deploy contract
    let init_args = build_init_args(1_000);
    let deploy_receipt = node
        .deploy_contract(alice, TOKEN_WASM.to_vec(), init_args, DEFAULT_GAS_LIMIT)
        .unwrap();

    let contract = deploy_receipt.contract_address.unwrap();

    // Make another call
    let bob = node.create_account(2, TokenUnit::from_tokens(10.0));

    let transfer_args = build_transfer_args(&bob, 100);
    node.call_contract(
        alice,
        contract,
        "transfer".to_string(),
        transfer_args,
        DEFAULT_GAS_LIMIT,
    )
    .unwrap();

    // Verify chain continuity
    let block1 = node.get_block(1).unwrap();
    let block2 = node.get_block(2).unwrap();

    assert_eq!(
        block2.parent_hash,
        block1.hash(),
        "Block 2 should reference block 1"
    );
    assert_eq!(block2.number, block1.number + 1, "Block numbers sequential");
    assert!(
        block2.timestamp >= block1.timestamp,
        "Timestamps non-decreasing"
    );
}

#[test]
fn test_node_contract_state_persistence() {
    let mut node = WasmletteNode::new().unwrap();

    let alice = node.create_account(1, TokenUnit::from_tokens(10.0));
    let bob = node.create_account(2, TokenUnit::from_tokens(10.0));

    // Deploy contract
    let init_supply = 5_000u64;
    let init_args = build_init_args(init_supply);

    let deploy_receipt = node
        .deploy_contract(alice, TOKEN_WASM.to_vec(), init_args, DEFAULT_GAS_LIMIT)
        .unwrap();

    let contract = deploy_receipt.contract_address.unwrap();

    // First transfer
    let args1 = build_transfer_args(&bob, 1_000);
    node.call_contract(
        alice,
        contract,
        "transfer".to_string(),
        args1,
        DEFAULT_GAS_LIMIT,
    )
    .unwrap();

    let alice_tokens_after_first = node
        .get_storage(contract, get_balance_key(&alice))
        .map(|b| u64::from_le_bytes(b.try_into().unwrap()))
        .unwrap();

    // Second transfer builds on first
    let args2 = build_transfer_args(&bob, 500);
    node.call_contract(
        alice,
        contract,
        "transfer".to_string(),
        args2,
        DEFAULT_GAS_LIMIT,
    )
    .unwrap();

    let alice_tokens_after_second = node
        .get_storage(contract, get_balance_key(&alice))
        .map(|b| u64::from_le_bytes(b.try_into().unwrap()))
        .unwrap();

    let bob_tokens_final = node
        .get_storage(contract, get_balance_key(&bob))
        .map(|b| u64::from_le_bytes(b.try_into().unwrap()))
        .unwrap();

    // Verify state persisted across blocks
    assert_eq!(
        alice_tokens_after_first, 4_000,
        "Alice: 5000 - 1000 after first"
    );
    assert_eq!(
        alice_tokens_after_second, 3_500,
        "Alice: 4000 - 500 after second"
    );
    assert_eq!(bob_tokens_final, 1_500, "Bob: 1000 + 500 total");
}

#[test]
fn test_node_contract_gas_refund() {
    let mut node = WasmletteNode::new().unwrap();

    let alice = node.create_account(1, TokenUnit::from_tokens(10.0));

    let initial_balance = node.get_balance(&alice);

    // Deploy with high gas limit
    let init_args = build_init_args(1_000);
    let high_gas_limit = 5_000_000u64; // Much higher than needed

    let receipt = node
        .deploy_contract(alice, TOKEN_WASM.to_vec(), init_args, high_gas_limit)
        .unwrap();

    // Verify only actual gas was charged
    assert!(
        receipt.gas_used < high_gas_limit,
        "Should refund unused gas"
    );

    let final_balance = node.get_balance(&alice);
    let gas_paid = initial_balance - final_balance;

    // Gas paid should match actual gas used, not the limit
    assert_eq!(
        gas_paid, receipt.gas_used,
        "Should only pay for actual gas used"
    );
}

#[test]
fn test_node_multiple_contracts() {
    let mut node = WasmletteNode::new().unwrap();

    let alice = node.create_account(1, TokenUnit::from_tokens(20.0));

    // Deploy first contract
    let init_args1 = build_init_args(1_000);
    let receipt1 = node
        .deploy_contract(alice, TOKEN_WASM.to_vec(), init_args1, DEFAULT_GAS_LIMIT)
        .unwrap();

    let contract1 = receipt1.contract_address.unwrap();
    assert_eq!(node.height(), 1, "First contract deployed");

    // Deploy second contract
    let init_args2 = build_init_args(2_000);
    let receipt2 = node
        .deploy_contract(alice, TOKEN_WASM.to_vec(), init_args2, DEFAULT_GAS_LIMIT)
        .unwrap();

    let contract2 = receipt2.contract_address.unwrap();
    assert_eq!(node.height(), 2, "Second contract deployed");

    // Verify both contracts exist
    assert!(node.contract_exists(&contract1), "Contract 1 exists");
    assert!(node.contract_exists(&contract2), "Contract 2 exists");

    // Verify they have different addresses
    assert_ne!(contract1, contract2, "Contracts have different addresses");

    // Verify each has correct supply
    let supply1 = node
        .get_storage(contract1, b"total_supply".to_vec())
        .map(|b| u64::from_le_bytes(b.try_into().unwrap()))
        .unwrap();

    let supply2 = node
        .get_storage(contract2, b"total_supply".to_vec())
        .map(|b| u64::from_le_bytes(b.try_into().unwrap()))
        .unwrap();

    assert_eq!(supply1, 1_000, "Contract 1 supply");
    assert_eq!(supply2, 2_000, "Contract 2 supply");

    // Verify nonce incremented twice
    assert_eq!(node.get_nonce(&alice), 2, "Alice deployed 2 contracts");
}
