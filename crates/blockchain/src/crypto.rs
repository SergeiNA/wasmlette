//! Cryptographic functions for contracts

use blake3;

/// Hash data using Blake3
pub fn hash_blake3(data: &[u8]) -> [u8; 32] {
    blake3::hash(data).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_blake3() {
        let data = b"hello world";
        let hash1 = hash_blake3(data);
        let hash2 = hash_blake3(data);

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, [0u8; 32]);
    }
}
