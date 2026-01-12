//! Utility functions

use crate::transaction::Address;

/// Generate a contract address from deployer address and nonce
pub fn generate_contract_address(deployer: &Address, nonce: u64) -> Address {
    let mut data = Vec::new();
    data.extend_from_slice(deployer.as_bytes());
    data.extend_from_slice(&nonce.to_le_bytes());

    let hash = blake3::hash(&data);
    Address::from_slice(hash.as_bytes())
}

/// Get current Unix timestamp
pub fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_contract_address() {
        let deployer = Address::zero();
        let addr1 = generate_contract_address(&deployer, 0);
        let addr2 = generate_contract_address(&deployer, 1);

        // Different nonces should produce different addresses
        assert_ne!(addr1, addr2);

        // Same inputs should produce same address
        let addr3 = generate_contract_address(&deployer, 0);
        assert_eq!(addr1, addr3);
    }

    #[test]
    fn test_current_timestamp() {
        let ts1 = current_timestamp();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let ts2 = current_timestamp();

        assert!(ts2 >= ts1);
    }
}
