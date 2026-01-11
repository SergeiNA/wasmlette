//! Simple Token with Caller Context
//!
//! A minimal fungible token that uses context::get_caller()
//! to ensure users can only transfer their own tokens.
//!
//! This contract demonstrates the use of the new procedural macros:
//! - #[contract_init] for initialization functions
//! - #[contract_call] for state-changing functions
//! - #[contract_query] for read-only functions

#![no_std]

use wasmlette_contracts_sdk::{
    context, contract_call, contract_init, contract_query, storage, ADDRESS_LENGTH,
};

// Storage keys
const TOTAL_SUPPLY_KEY: &[u8] = b"total_supply";
const BALANCE_KEY: &[u8] = b"balance";
const TRANSFER_COUNT_KEY: &[u8] = b"transfer_count";

const KEY_NAME_LEN: usize = BALANCE_KEY.len();
const TOTAL_KEY_LEN: usize = KEY_NAME_LEN + ADDRESS_LENGTH;

/// Initialize the token with fixed supply
/// All tokens go to the deployer
#[contract_init]
fn init(initial_supply: u64) {
    let deployer = context::get_caller();

    // Set total supply (immutable after init)
    storage::set(TOTAL_SUPPLY_KEY, &initial_supply.to_le_bytes());

    // Give all tokens to deployer
    set_balance(&deployer, initial_supply);

    // Initialize transfer counter
    storage::set(TRANSFER_COUNT_KEY, &0u64.to_le_bytes());
}

/// Get the total supply (fixed)
#[contract_query]
fn total_supply() -> u64 {
    get_u64(TOTAL_SUPPLY_KEY).unwrap_or(0)
}

/// Get balance of an address
#[contract_query]
fn balance_of(address: &[u8; ADDRESS_LENGTH]) -> u64 {
    get_balance(address)
}

/// Get total number of transfers
#[contract_query]
fn transfer_count() -> u64 {
    get_u64(TRANSFER_COUNT_KEY).unwrap_or(0)
}

/// Transfer tokens from caller to recipient
///
/// # Arguments
/// * `to` - array of 20-byte recipient address in WASM memory
/// * `amount` - Amount to transfer
///
/// # Returns
/// 0 on success, negative error code on failure
#[contract_call]
fn transfer(to: &[u8; ADDRESS_LENGTH], amount: u64) -> i32 {
    // ✅ Get the ACTUAL caller from runtime - guaranteed by host!
    let from = context::get_caller();

    // Validate inputs
    if amount == 0 {
        return -1; // Error: zero amount
    }

    if &from == to {
        return -2; // Error: self-transfer
    }

    // Check sender balance
    let from_balance = get_balance(&from);
    if from_balance < amount {
        return -3; // Error: insufficient balance
    }

    // Get recipient balance
    let to_balance = get_balance(&to);

    // Check for overflow
    if to_balance > u64::MAX - amount {
        return -4; // Error: balance overflow
    }

    // Update balances
    set_balance(&from, from_balance - amount);
    set_balance(&to, to_balance + amount);

    // Increment transfer counter
    let count = transfer_count();
    storage::set(TRANSFER_COUNT_KEY, &(count + 1).to_le_bytes());

    0 // Success
}

// ========== Helper Functions ==========

/// Get balance for an address
fn get_balance(address: &[u8; ADDRESS_LENGTH]) -> u64 {
    let mut key = [0u8; TOTAL_KEY_LEN]; // "balance" (7) + address (20)
    key[..KEY_NAME_LEN].copy_from_slice(BALANCE_KEY);
    key[KEY_NAME_LEN..].copy_from_slice(address);
    get_u64(&key).unwrap_or(0)
}

/// Set balance for an address
fn set_balance(address: &[u8; ADDRESS_LENGTH], amount: u64) {
    let mut key = [0u8; TOTAL_KEY_LEN];
    key[..KEY_NAME_LEN].copy_from_slice(BALANCE_KEY);
    key[KEY_NAME_LEN..].copy_from_slice(address);
    storage::set(&key, &amount.to_le_bytes());
}

/// Read u64 from storage
fn get_u64(key: &[u8]) -> Option<u64> {
    storage::get(key).and_then(|bytes| {
        if bytes.len() == 8 {
            let mut arr = [0u8; 8];
            arr.copy_from_slice(&bytes);
            Some(u64::from_le_bytes(arr))
        } else {
            None
        }
    })
}
