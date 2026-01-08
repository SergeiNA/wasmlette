//! Blockchain state machine

use crate::errors::BlockchainError;
use crate::{Block, State};

/// Manages the blockchain state
pub struct Blockchain {
    /// All blocks in the chain
    blocks: Vec<Block>,

    /// Current state
    state: State,
}

impl Blockchain {
    /// Create a new blockchain with genesis block
    pub fn new() -> Self {
        let genesis = Block::genesis();
        Blockchain {
            blocks: vec![genesis],
            state: State::new(),
        }
    }

    /// Get the current block number
    pub fn current_block_number(&self) -> u64 {
        self.blocks.len() as u64 - 1
    }

    /// Get the latest block
    pub fn latest_block(&self) -> &Block {
        self.blocks
            .last()
            .expect("Blockchain should have at least genesis")
    }

    /// Add a new block to the chain
    pub fn add_block(&mut self, block: Block) -> Result<(), BlockchainError> {
        // TODO: Validate block
        self.blocks.push(block);
        Ok(())
    }

    /// Get a reference to the current state
    pub fn state(&self) -> &State {
        &self.state
    }

    /// Get a mutable reference to the current state
    pub fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }
}

impl Default for Blockchain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_blockchain() {
        let chain = Blockchain::new();
        assert_eq!(chain.current_block_number(), 0);
    }
}
