//! Counter smart contract example
//!
//! A simple contract that maintains a counter and allows incrementing/decrementing

use wasmlette_contracts_sdk::storage;

const COUNTER_KEY: &[u8] = b"count";

/// Initialize the contract
#[no_mangle]
pub extern "C" fn init() {
    // Set initial count to 0
    storage::set(COUNTER_KEY, &0u64.to_le_bytes());
}

/// Increment the counter
#[no_mangle]
pub extern "C" fn increment() {
    let count = get_count();
    storage::set(COUNTER_KEY, &(count + 1).to_le_bytes());
}

/// Decrement the counter
#[no_mangle]
pub extern "C" fn decrement() {
    let count = get_count();
    if count > 0 {
        storage::set(COUNTER_KEY, &(count - 1).to_le_bytes());
    }
}

/// Increment the counter
#[no_mangle]
pub extern "C" fn set_count(value: u64) {
    storage::set(COUNTER_KEY, &(value).to_le_bytes());
}

/// Get the current count
#[no_mangle]
pub extern "C" fn get_count() -> u64 {
    storage::get(COUNTER_KEY)
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
