//! Counter smart contract example
//!
//! A simple contract that maintains a counter and allows incrementing/decrementing

use wasmlette_contracts_sdk::{balance, context, crypto, storage, ADDRESS_LENGTH};

const COUNTER_KEY: &[u8] = b"count";

/// Initialize the contract
#[no_mangle]
pub extern "C" fn init() {
    // Set initial count to 0
    storage::set(COUNTER_KEY, &0u64.to_le_bytes());
}

/// Increment the counter
#[no_mangle]
pub extern "C" fn test_set_count(value: u64) {
    storage::set(COUNTER_KEY, &value.to_le_bytes());
}

/// Get the current count
#[no_mangle]
pub extern "C" fn test_get_count() -> u64 {
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

#[no_mangle]
pub extern "C" fn test_get_balance(address: &[u8; ADDRESS_LENGTH]) -> u64 {
    balance::get(address)
}

#[no_mangle]
pub extern "C" fn test_hash(value: u64, output_ptr: *mut u8) {
    let hash = crypto::hash(&value.to_le_bytes());

    // Write hash to output buffer
    unsafe {
        core::ptr::copy_nonoverlapping(hash.as_ptr(), output_ptr, 32);
    }
}

/// Increment the counter
#[no_mangle]
pub extern "C" fn test_get_caller(output_ptr: *mut u8) {
    let caller_addr = context::get_caller();
    unsafe {
        core::ptr::copy_nonoverlapping(caller_addr.as_ptr(), output_ptr, ADDRESS_LENGTH);
    }
}
