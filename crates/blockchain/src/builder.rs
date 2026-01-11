//! Block building functionality

use crate::utils::current_timestamp;
use crate::{Block, State, Transaction};
use anyhow::Result;

pub struct BlockBuilder<'a> {
    /// Previous block (parent)
    parent: Option<&'a Block>,

    /// Transactions to include
    transactions: Vec<Transaction>,
}

impl<'a> BlockBuilder<'a> {
    /// Create a new block builder
    pub fn new() -> Self {
        Self {
            parent: None,
            transactions: Vec::new(),
        }
    }

    /// Create builder for genesis block
    pub fn genesis() -> Self {
        Self::new()
    }

    /// Create builder with parent block
    pub fn with_parent(parent: &'a Block) -> Self {
        Self {
            parent: Some(parent),
            transactions: Vec::new(),
        }
    }

    /// Add a transaction to the block
    pub fn add_transaction(&mut self, tx: Transaction) {
        self.transactions.push(tx);
    }

    /// Build the block
    pub fn build(self, state: &State) -> Result<Block> {
        let (block_number, parent_hash) = if let Some(parent) = self.parent {
            (parent.number + 1, parent.hash())
        } else {
            // Genesis block
            (0, [0u8; 32])
        };

        let current_timestamp = current_timestamp();
        let state_root = Self::compute_state_root(state)?;
        Ok(Block::new(
            block_number,
            current_timestamp,
            parent_hash,
            self.transactions,
            state_root,
        ))
    }

    /// Compute state root hash
    ///
    /// The state root is a cryptographic commitment to the world state only.
    /// It does NOT include block metadata (number, timestamp, parent_hash) -
    /// those are already in the Block struct and will be included in Block::hash().
    ///
    /// This follows the industry standard where:
    /// - State root = hash(state only)
    /// - Block hash = hash(block_header including state_root)
    fn compute_state_root(state: &State) -> Result<[u8; 32]> {
        // Serialize and hash state
        let state_data = bincode::encode_to_vec(state, bincode::config::standard())
            .map_err(|e| anyhow::anyhow!("Failed to serialize state: {}", e))?;

        // Hash the serialized state
        Ok(blake3::hash(&state_data).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Address, TransactionKind};

    #[test]
    fn test_genesis_block() {
        let state = State::new();
        let genesis = BlockBuilder::genesis().build(&state).unwrap();

        assert_eq!(genesis.number, 0);
        assert_eq!(genesis.parent_hash, [0u8; 32]);
        assert_eq!(genesis.transactions.len(), 0);
    }

    #[test]
    fn test_block_with_parent() {
        let state = State::new();
        let genesis = BlockBuilder::genesis().build(&state).unwrap();

        let mut builder = BlockBuilder::with_parent(&genesis);

        let tx = Transaction::new(
            Address::zero(),
            0,
            TransactionKind::Transfer {
                to: Address::zero(),
                amount: 100,
            },
            21_000,
            1,
        );
        builder.add_transaction(tx);

        let block = builder.build(&state).unwrap();

        assert_eq!(block.number, 1);
        assert_eq!(block.parent_hash, genesis.hash());
        assert_eq!(block.transactions.len(), 1);
    }

    #[test]
    fn test_block_chain() {
        let state = State::new();

        let genesis = BlockBuilder::genesis().build(&state).unwrap();
        let block1 = BlockBuilder::with_parent(&genesis).build(&state).unwrap();
        let block2 = BlockBuilder::with_parent(&block1).build(&state).unwrap();

        assert_eq!(genesis.number, 0);
        assert_eq!(block1.number, 1);
        assert_eq!(block2.number, 2);

        assert_eq!(block1.parent_hash, genesis.hash());
        assert_eq!(block2.parent_hash, block1.hash());
    }

    #[test]
    fn test_timestamp_increases() {
        let state = State::new();

        let block1 = BlockBuilder::genesis().build(&state).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let block2 = BlockBuilder::with_parent(&block1).build(&state).unwrap();

        assert!(block2.timestamp >= block1.timestamp);
    }
}
