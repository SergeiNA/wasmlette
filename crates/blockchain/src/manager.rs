//! Blockchain management and validation

use crate::{Block, BlockBuilder, BlockchainError, State};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

//  _room_state: std::marker::PhantomData<RoomState>,

pub struct BlockchainManager {
    /// Shared state
    state: Arc<Mutex<State>>,

    /// All blocks indexed by number
    blocks: HashMap<u64, Block>,

    /// Current chain head (highest block)
    head: Option<Block>,
}

impl BlockchainManager {
    /// Create a new blockchain manager and init with genesis block
    /// TODO genesis block should contain predefined state with balances
    pub fn new(state: Arc<Mutex<State>>) -> Result<Self, BlockchainError> {
        let mut init = Self {
            state,
            blocks: HashMap::new(),
            head: None,
        };
        init.initialize_with_genesis()?;
        Ok(init)
    }

    /// Initialize with genesis block
    fn initialize_with_genesis(&mut self) -> Result<(), BlockchainError> {
        if self.head.is_some() {
            return Err(BlockchainError::GenesisAlreadyExists);
        }

        let state = self
            .state
            .lock()
            .map_err(|_| BlockchainError::BlockGenerationError)?;
        let genesis = BlockBuilder::genesis()
            .build(&state)
            .map_err(|_| BlockchainError::BlockGenerationError)?;
        self.blocks.insert(0, genesis.clone());
        self.head = Some(genesis.clone());
        Ok(())
    }

    // Add a new block to the chain
    pub fn add_block(&mut self, block: Block) -> Result<(), BlockchainError> {
        // Validate block
        self.validate_block(&block)?;

        // Store block
        self.blocks.insert(block.number, block.clone());

        // Update head if this is the new highest block
        if self.head.as_ref().map(|h| h.number) < Some(block.number) {
            self.head = Some(block);
        }

        Ok(())
    }

    pub fn validate_block(&self, block: &Block) -> Result<(), BlockchainError> {
        // Check if we have parent (unless it's genesis)
        if block.number == 0 {
            return Ok(());
        }

        let parent = self
            .blocks
            .get(&(block.number - 1))
            .ok_or(BlockchainError::BlockNotFound(block.number - 1))?;

        // Validate parent hash
        if block.parent_hash != parent.hash() {
            return Err(BlockchainError::InvalidParentHash {
                expected: parent.hash(),
                got: block.parent_hash,
            });
        }

        // Validate block number
        let expected_number = parent.number + 1;
        if block.number != expected_number {
            return Err(BlockchainError::InvalidBlockNumber {
                expected: expected_number,
                got: block.number,
            });
        }

        Ok(())
    }

    /// Get a block by number
    pub fn get_block(&self, number: u64) -> Option<&Block> {
        self.blocks.get(&number)
    }

    /// Get the current head block
    pub fn get_head(&self) -> Option<&Block> {
        self.head.as_ref()
    }

    /// Get the current chain height
    pub fn height(&self) -> u64 {
        self.head.as_ref().map(|h| h.number).unwrap_or(0)
    }

    /// Get the genesis block
    pub fn genesis(&self) -> Option<&Block> {
        self.blocks.get(&0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_genesis() {
        let state = Arc::new(Mutex::new(State::new()));
        let chain = BlockchainManager::new(state).unwrap();

        let genesis = chain.head.as_ref().unwrap();
        assert_eq!(genesis.number, 0);
        assert_eq!(chain.height(), 0);
    }

    #[test]
    fn test_add_valid_blocks() {
        let state = Arc::new(Mutex::new(State::new()));
        let mut chain = BlockchainManager::new(state.clone()).unwrap();

        let genesis = chain.head.clone().unwrap();

        let block1 = BlockBuilder::with_parent(&genesis)
            .build(&state.lock().unwrap())
            .unwrap();
        chain.add_block(block1.clone()).unwrap();

        let block2 = BlockBuilder::with_parent(&block1)
            .build(&state.lock().unwrap())
            .unwrap();
        chain.add_block(block2).unwrap();

        assert_eq!(chain.height(), 2);
    }

    #[test]
    fn test_invalid_parent_hash() {
        let state = Arc::new(Mutex::new(State::new()));
        let mut chain = BlockchainManager::new(state.clone()).unwrap();

        let genesis = chain.head.clone().unwrap();

        // Create block with wrong parent hash
        let mut block = BlockBuilder::with_parent(&genesis)
            .build(&state.lock().unwrap())
            .unwrap();
        block.parent_hash = [1u8; 32]; // Wrong!

        let result = chain.add_block(block);
        assert!(result.is_err());
    }

    #[test]
    fn test_skip_block_number() {
        let state = Arc::new(Mutex::new(State::new()));
        let mut chain = BlockchainManager::new(state.clone()).unwrap();

        // Try to add block 2 without block 1
        let mut block = Block::genesis();
        block.number = 2;

        let result = chain.add_block(block);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_blocks() {
        let state = Arc::new(Mutex::new(State::new()));
        let mut chain = BlockchainManager::new(state.clone()).unwrap();

        let genesis = chain.head.clone().unwrap();
        let block1 = BlockBuilder::with_parent(&genesis)
            .build(&state.lock().unwrap())
            .unwrap();
        chain.add_block(block1).unwrap();

        assert_eq!(chain.get_block(0).unwrap().number, 0);
        assert_eq!(chain.get_block(1).unwrap().number, 1);
        assert!(chain.get_block(2).is_none());
    }
}
