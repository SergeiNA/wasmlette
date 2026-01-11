//! Block structure and validation logic

use crate::transaction::Transaction;
use serde::{Deserialize, Serialize};

/// Represents a single block in the blockchain
#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct Block {
    /// Block number (height)
    pub number: u64,

    /// Unix timestamp
    pub timestamp: u64,

    /// Hash of the parent block
    pub parent_hash: [u8; 32],

    /// Transactions included in this block
    pub transactions: Vec<Transaction>,

    /// State root (Merkle root of the state)
    pub state_root: [u8; 32],
}

impl Block {
    /// Create a new block
    pub fn new(
        number: u64,
        timestamp: u64,
        parent_hash: [u8; 32],
        transactions: Vec<Transaction>,
        state_root: [u8; 32],
    ) -> Self {
        Block {
            number,
            timestamp,
            parent_hash,
            transactions,
            state_root,
        }
    }

    /// Calculate the hash of this block
    /// Uses bincode for deterministic binary serialization
    pub fn hash(&self) -> [u8; 32] {
        let serialized = bincode::encode_to_vec(self, bincode::config::standard())
            .expect("Failed to serialize block");
        blake3::hash(&serialized).into()
    }

    /// Create genesis block
    pub fn genesis() -> Self {
        Block {
            number: 0,
            timestamp: 0,
            parent_hash: [0u8; 32],
            transactions: vec![],
            // TODO: Compute actual state root (Phase 4 - see Guide 12)
            state_root: [0u8; 32],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_block() {
        let genesis = Block::genesis();
        assert_eq!(genesis.number, 0);
        assert_eq!(genesis.transactions.len(), 0);
    }

    #[test]
    fn test_block_hash() {
        let block = Block::genesis();
        let hash1 = block.hash();
        let hash2 = block.hash();
        assert_eq!(hash1, hash2, "Hash should be deterministic");
    }

    #[test]
    fn test_create_blockchain() {
        let block = Block::genesis();
        let block1 = Block::new(1, 1625247600, block.hash(), vec![], [0u8; 32]);
        assert_eq!(block1.number, 1);
        assert_eq!(block1.parent_hash, block.hash());
    }
}
