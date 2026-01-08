//! Token smart contract example
//!
//! A simple fungible token implementation

use wasmlette_contracts_sdk::{storage, ADDRESS_LENGTH};

const TOTAL_SUPPLY_KEY: &[u8] = b"total_supply";

/// Initialize the token with a total supply
#[no_mangle]
pub extern "C" fn init(initial_supply: u64) {
    storage::set(TOTAL_SUPPLY_KEY, &initial_supply.to_le_bytes());
}

/// Get the total supply
#[no_mangle]
pub extern "C" fn total_supply() -> u64 {
    storage::get(TOTAL_SUPPLY_KEY)
        .and_then(|bytes| {
            if bytes.len() == 8 {
                let mut arr = [0u8; 8];
                arr.copy_from_slice(&bytes);
                Some(u64::from_le_bytes(arr))
            } else {
                None
            }
        })
        .unwrap_or(0)
}

/// Get balance of an address
#[no_mangle]
pub extern "C" fn balance_of(address: &[u8; ADDRESS_LENGTH]) -> u64 {
    let mut key = b"balance:".to_vec();
    key.extend_from_slice(address);

    storage::get(&key)
        .and_then(|bytes| {
            if bytes.len() == 8 {
                let mut arr = [0u8; 8];
                arr.copy_from_slice(&bytes);
                Some(u64::from_le_bytes(arr))
            } else {
                None
            }
        })
        .unwrap_or(0)
}

/// Transfer tokens
#[no_mangle]
pub extern "C" fn transfer(to: &[u8; ADDRESS_LENGTH], amount: u64) {
    // TODO: Implement transfer logic
    // 1. Get caller address
    // 2. Check balance
    // 3. Update balances
    let _ = (to, amount); // Silence unused warnings
}
