//! Contract-facing API

use wasmlette_blockchain::transaction::Address;

pub const ADDRESS_LENGTH: usize = Address::LENGTH;

/// Storage API
pub mod storage {
    //! Safe Rust API for contract development

    use crate::api::{get_storage, set_storage};

    /// Read value from contract storage
    ///
    /// # Arguments
    /// * `key` - Storage key
    ///
    /// # Returns
    /// * `Some(value)` if key exists
    /// * `None` if key not found
    pub fn get(key: &[u8]) -> Option<Vec<u8>> {
        // Allocate buffer for result (max 1KB)
        let mut buffer = [0u8; 1024];

        unsafe {
            // Call host function
            let len = get_storage(
                key.as_ptr(),
                key.len() as u32,
                buffer.as_mut_ptr(),
                buffer.len() as u32,
            );

            if len < 0 {
                // Error occurred
                None
            } else if len == 0 {
                // Key not found
                None
            } else {
                // Return the value
                Some(buffer[..len as usize].to_vec())
            }
        }
    }

    /// Write value to contract storage
    ///
    /// # Arguments
    /// * `key` - Storage key
    /// * `value` - Value to store
    pub fn set(key: &[u8], value: &[u8]) {
        unsafe {
            set_storage(
                key.as_ptr(),
                key.len() as u32,
                value.as_ptr(),
                value.len() as u32,
            );
        }
    }
}

/// Get balance of an address
///
/// # Arguments
/// * `address` - 20-byte address
///
/// # Returns
/// Balance in smallest units
pub mod balance {
    use crate::api::get_balance;
    use crate::ADDRESS_LENGTH;

    /// Get the balance of an address
    pub fn get(address: &[u8; ADDRESS_LENGTH]) -> u64 {
        unsafe {
            let balance = get_balance(address.as_ptr());
            balance as u64
        }
    }
}

/// Hash data using Blake3
///
/// # Arguments
/// * `data` - Data to hash
///
/// # Returns
/// 32-byte hash
pub mod crypto {
    use crate::api::hash_blake3;

    /// Hash data with Blake3
    pub fn hash(data: &[u8]) -> [u8; 32] {
        let mut hash = [0u8; 32];
        unsafe {
            hash_blake3(data.as_ptr(), data.len() as u32, hash.as_mut_ptr());
        }
        hash
    }
}

/// Context functions - caller information
pub mod context {
    use super::get_caller as host_get_caller;
    use crate::ADDRESS_LENGTH;

    /// Get the address that called this contract
    ///
    /// # Returns
    /// The 20-byte address of the caller
    pub fn get_caller() -> [u8; ADDRESS_LENGTH] {
        let mut addr = [0u8; ADDRESS_LENGTH];
        unsafe {
            host_get_caller(addr.as_mut_ptr());
        }
        addr
    }
}

// Raw host function declarations
extern "C" {
    fn get_storage(key_ptr: *const u8, key_len: u32, output_ptr: *mut u8, capacity: u32) -> i32;
    fn set_storage(key_ptr: *const u8, key_len: u32, value_ptr: *const u8, value_len: u32);
    fn get_balance(address_ptr: *const u8) -> i64;
    fn hash_blake3(data_ptr: *const u8, data_len: u32, output_ptr: *mut u8);
    fn get_caller(output_ptr: *mut u8);
}
