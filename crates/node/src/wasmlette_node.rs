//! High-level demon API
//!
//! Provides simple interface for blockchain operations.

use crate::constants::{DEFAULT_GAS_LIMIT_TRANSFER, DEFAULT_GAS_PRICE};
use std::sync::{Arc, Mutex};
use wasmlette_blockchain::{
    Address, Block, BlockBuilder, BlockchainManager, State, Transaction, TransactionKind,
    TransactionReceipt,
};
use wasmlette_runtime::ContractExecutor;

pub struct WasmletteNode {
    /// Blockchain manager
    chain: BlockchainManager,

    /// Transaction processor
    executor: ContractExecutor,

    /// Shared state
    state: Arc<Mutex<State>>,
}

impl WasmletteNode {
    /// Create a new demon and initialize with genesis
    pub fn new() -> anyhow::Result<Self> {
        let state = Arc::new(Mutex::new(State::new()));
        let executor = ContractExecutor::new()?;
        let chain = BlockchainManager::new(state.clone())?;

        Ok(Self {
            chain,
            executor,
            state,
        })
    }

    /// Deploy a contract (creates transaction → block)
    pub fn deploy_contract(
        &mut self,
        deployer: Address,
        wasm_code: Vec<u8>,
        init_args: Vec<u8>,
        gas_limit: u64,
    ) -> anyhow::Result<TransactionReceipt> {
        // Get nonce
        let nonce = self
            .state
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire state lock: {}", e))?
            .get_nonce(&deployer);

        // Create transaction
        let tx = Transaction::new(
            deployer,
            nonce,
            TransactionKind::Deploy {
                wasm_code,
                init_args,
            },
            gas_limit,
            DEFAULT_GAS_PRICE,
        );

        // Execute transaction and create block
        self.execute_transaction(tx)
    }

    /// Call a contract method (creates transaction → block)
    pub fn call_contract(
        &mut self,
        caller: Address,
        contract: Address,
        method: String,
        args: Vec<u8>,
        gas_limit: u64,
    ) -> anyhow::Result<TransactionReceipt> {
        let nonce = self
            .state
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire state lock: {}", e))?
            .get_nonce(&caller);

        let tx = Transaction::new(
            caller,
            nonce,
            TransactionKind::Call {
                contract,
                method,
                args,
            },
            gas_limit,
            DEFAULT_GAS_PRICE,
        );

        self.execute_transaction(tx)
    }

    /// Transfer native tokens (creates transaction → block)
    pub fn transfer(
        &mut self,
        from: Address,
        to: Address,
        amount: u64,
    ) -> anyhow::Result<TransactionReceipt> {
        let nonce = self
            .state
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire state lock: {}", e))?
            .get_nonce(&from);

        let tx = Transaction::new(
            from,
            nonce,
            TransactionKind::Transfer { to, amount },
            DEFAULT_GAS_LIMIT_TRANSFER,
            DEFAULT_GAS_PRICE,
        );

        self.execute_transaction(tx)
    }

    /// Execute a transaction and create a block
    fn execute_transaction(&mut self, tx: Transaction) -> anyhow::Result<TransactionReceipt> {
        // Execute transaction
        let receipt = self.executor.execute_transaction(self.state.clone(), &tx)?;

        // Create block with this transaction
        let parent = self
            .chain
            .get_head()
            .ok_or_else(|| anyhow::anyhow!("No genesis block"))?
            .clone();

        let mut builder = BlockBuilder::with_parent(&parent);
        builder.add_transaction(tx);
        let state = self
            .state
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire state lock: {}", e))?;
        let block = builder.build(&*state)?;

        // Add block to chain
        self.chain.add_block(block)?;

        Ok(receipt)
    }

    // Query methods (don't create transactions/blocks)

    /// Get account balance
    pub fn get_balance(&self, address: &Address) -> u64 {
        self.state
            .lock()
            .map(|state| state.get_balance(address))
            .unwrap_or(0)
    }

    /// Get account nonce
    pub fn get_nonce(&self, address: &Address) -> u64 {
        self.state
            .lock()
            .map(|state| state.get_nonce(address))
            .unwrap_or(0)
    }

    /// Check if contract exists
    pub fn contract_exists(&self, address: &Address) -> bool {
        self.state
            .lock()
            .map(|state| state.contract_exists(address))
            .unwrap_or(false)
    }

    /// Get contract storage value
    pub fn get_storage(&self, contract: Address, key: Vec<u8>) -> Option<Vec<u8>> {
        self.state
            .lock()
            .ok()
            .and_then(|state| state.get_storage(contract, key))
    }

    /// Get block by number
    pub fn get_block(&self, number: u64) -> Option<&Block> {
        self.chain.get_block(number)
    }

    /// Get current chain height
    pub fn height(&self) -> u64 {
        self.chain.height()
    }

    /// Get genesis block
    pub fn genesis(&self) -> Option<&Block> {
        self.chain.genesis()
    }

    /// Create an account with initial balance (for testing)
    pub fn create_account(&mut self, seed: u8, balance: u64) -> Address {
        let address = Address::from_slice(&[seed; Address::LENGTH]);
        if let Ok(mut state) = self.state.lock() {
            state.set_balance(address, balance);
        }
        address
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = WasmletteNode::new().unwrap();
        assert_eq!(node.height(), 0);
        assert!(node.genesis().is_some());
    }

    #[test]
    fn test_account_creation() {
        let mut node = WasmletteNode::new().unwrap();

        let alice = node.create_account(1, 1_000_000);
        assert_eq!(node.get_balance(&alice), 1_000_000);
        assert_eq!(node.get_nonce(&alice), 0);
    }

    #[test]
    fn test_transfer() {
        let mut node = WasmletteNode::new().unwrap();

        let alice = node.create_account(1, 1_000_000);
        let bob = node.create_account(2, 0);

        let receipt = node.transfer(alice, bob, 500_000).unwrap();

        assert!(receipt.success);
        assert_eq!(node.height(), 1); // Block created
        assert_eq!(node.get_balance(&bob), 500_000);
    }

    #[test]
    fn test_deploy_creates_block() {
        let mut node = WasmletteNode::new().unwrap();
        let deployer = node.create_account(1, 10_000_000);

        let wasm_code =
            include_bytes!("../../../target/wasm32-unknown-unknown/release/tester.wasm");

        let receipt = node
            .deploy_contract(deployer, wasm_code.to_vec(), vec![], 1_000_000)
            .unwrap();

        assert!(receipt.success);
        assert!(receipt.contract_address.is_some());
        assert_eq!(node.height(), 1); // Genesis was 0, now 1

        // Verify block contains transaction
        let block = node.get_block(1).unwrap();
        assert_eq!(block.transactions.len(), 1);
    }

    #[test]
    fn test_nonce_increments() {
        let mut node = WasmletteNode::new().unwrap();
        let alice = node.create_account(1, 10_000_000);
        let bob = node.create_account(2, 0);

        assert_eq!(node.get_nonce(&alice), 0);

        node.transfer(alice, bob, 100).unwrap();
        assert_eq!(node.get_nonce(&alice), 1);

        node.transfer(alice, bob, 100).unwrap();
        assert_eq!(node.get_nonce(&alice), 2);
    }
}
