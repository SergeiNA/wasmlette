//! Merkle tree implementation for state commitment

/// Calculate Merkle root of a list of hashes
pub fn merkle_root(hashes: &[[u8; 32]]) -> [u8; 32] {
    if hashes.is_empty() {
        return [0u8; 32];
    }

    if hashes.len() == 1 {
        return hashes[0];
    }

    let mut current_level = hashes.to_vec();

    while current_level.len() > 1 {
        let mut next_level = Vec::new();

        for chunk in current_level.chunks(2) {
            let hash = if chunk.len() == 2 {
                hash_pair(&chunk[0], &chunk[1])
            } else {
                // Odd number of elements, hash with itself
                hash_pair(&chunk[0], &chunk[0])
            };
            next_level.push(hash);
        }

        current_level = next_level;
    }

    current_level[0]
}

/// Hash two 32-byte arrays together
fn hash_pair(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut combined = [0u8; 64];
    combined[..32].copy_from_slice(left);
    combined[32..].copy_from_slice(right);
    blake3::hash(&combined).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_merkle_root() {
        let root = merkle_root(&[]);
        assert_eq!(root, [0u8; 32]);
    }

    #[test]
    fn test_single_element() {
        let hash = [1u8; 32];
        let root = merkle_root(&[hash]);
        assert_eq!(root, hash);
    }

    #[test]
    fn test_two_elements() {
        let hash1 = [1u8; 32];
        let hash2 = [2u8; 32];
        let root = merkle_root(&[hash1, hash2]);

        // Should be deterministic
        let root2 = merkle_root(&[hash1, hash2]);
        assert_eq!(root, root2);
    }

    #[test]
    fn test_multiple_elements() {
        let hashes: Vec<[u8; 32]> = (0..7)
            .map(|i| {
                let mut h = [0u8; 32];
                h[0] = i;
                h
            })
            .collect();

        let root = merkle_root(&hashes);

        // Should be deterministic
        let root2 = merkle_root(&hashes);
        assert_eq!(root, root2);
    }
}
