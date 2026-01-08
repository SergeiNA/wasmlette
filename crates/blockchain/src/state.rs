//! Account and contract state management

use crate::errors::BlockchainError;
use crate::transaction::Address;
use std::collections::HashMap;

/// Contract metadata
#[derive(Clone, Debug)]
pub struct ContractInfo {
    /// Contract address
    pub address: Address,

    /// WASM bytecode
    pub code: Vec<u8>,

    /// Hash of the code
    pub code_hash: [u8; 32],
}

/// Global state manager
pub struct State {
    /// Account balances in units of smallest denomination (micro-tokens)
    balances: HashMap<Address, u64>, 

    /// Deployed contracts
    contracts: HashMap<Address, ContractInfo>,

    /// Contract storage: (contract_address, key) -> value
    storage: HashMap<(Address, Vec<u8>), Vec<u8>>,

    /// Account nonces (for replay protection)
    nonces: HashMap<Address, u64>,
}

impl State {
    /// Create a new empty state
    pub fn new() -> Self {
        State {
            balances: HashMap::new(),
            contracts: HashMap::new(),
            storage: HashMap::new(),
            nonces: HashMap::new(),
        }
    }

    // Account operations

    /// Get the balance of an account
    pub fn get_balance(&self, address: &Address) -> u64 {
        *self.balances.get(address).unwrap_or(&0u64)
    }

    /// Set the balance of an account
    pub fn set_balance(&mut self, address: Address, balance: u64) {
        self.balances.insert(address, balance);
    }

    /// Transfer tokens between accounts
    pub fn transfer(
        &mut self,
        from: Address,
        to: Address,
        amount: u64,
    ) -> Result<(), BlockchainError> {
        let from_balance = self.get_balance(&from);
        if from_balance < amount {
            return Err(BlockchainError::InsufficientBalance {
                address: from,
                required: amount,
                available: from_balance,
            });
        }

        self.set_balance(from, from_balance - amount);
        let to_balance = self.get_balance(&to);
        self.set_balance(to, to_balance + amount);
        Ok(())
    }

    // Contract operations

    /// Deploy a new contract
    pub fn deploy_contract(
        &mut self,
        address: Address,
        code: Vec<u8>,
    ) -> Result<(), BlockchainError> {
        if self.contract_exists(&address) {
            return Err(BlockchainError::ContractAlreadyExists(address));
        }

        let code_hash = blake3::hash(&code).into();
        self.contracts.insert(
            address,
            ContractInfo {
                address,
                code,
                code_hash,
            },
        );
        Ok(())
    }

    /// Get contract information
    pub fn get_contract(&self, address: &Address) -> Option<&ContractInfo> {
        self.contracts.get(address)
    }

    /// Check if a contract exists at an address
    pub fn contract_exists(&self, address: &Address) -> bool {
        self.contracts.contains_key(address)
    }

    // Storage operations

    /// Get a value from contract storage
    pub fn get_storage(&self, contract: Address, key: Vec<u8>) -> Option<Vec<u8>> {
        self.storage.get(&(contract, key)).cloned()
    }

    /// Set a value in contract storage
    pub fn set_storage(&mut self, contract: Address, key: Vec<u8>, value: Vec<u8>) {
        self.storage.insert((contract, key), value);
    }

    /// Remove a value from contract storage
    pub fn remove_storage(&mut self, contract: Address, key: Vec<u8>) {
        self.storage.remove(&(contract, key));
    }

    // Nonce operations

    /// Get the nonce for an account
    pub fn get_nonce(&self, address: &Address) -> u64 {
        *self.nonces.get(address).unwrap_or(&0)
    }

    /// Increment the nonce for an account
    pub fn increment_nonce(&mut self, address: &Address) {
        let nonce = self.get_nonce(address);
        self.nonces.insert(*address, nonce + 1);
    }

    /// Set the nonce for an account
    pub fn set_nonce(&mut self, address: Address, nonce: u64) {
        self.nonces.insert(address, nonce);
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balance_operations() {
        let mut state = State::new();
        let addr = Address::zero();

        assert_eq!(state.get_balance(&addr), 0u64);

        state.set_balance(addr, 100u64);
        assert_eq!(state.get_balance(&addr), 100u64);
    }

    #[test]
    fn test_transfer() {
        let mut state = State::new();
        let from = Address::from_slice(&[1u8; Address::LENGTH]);
        let to = Address::from_slice(&[2u8; Address::LENGTH]);

        state.set_balance(from, 100u64);

        state.transfer(from, to, 30u64).unwrap();

        assert_eq!(state.get_balance(&from), 70u64);
        assert_eq!(state.get_balance(&to), 30u64);
    }

    #[test]
    fn test_insufficient_balance() {
        let mut state = State::new();
        let from = Address::from_slice(&[1u8; Address::LENGTH]);
        let to = Address::from_slice(&[2u8; Address::LENGTH]);

        state.set_balance(from, 50u64);

        let result = state.transfer(from, to, 100u64);
        assert!(result.is_err());
    }

    #[test]
    fn test_nonce() {
        let mut state = State::new();
        let addr = Address::zero();

        assert_eq!(state.get_nonce(&addr), 0);

        state.increment_nonce(&addr);
        assert_eq!(state.get_nonce(&addr), 1);

        state.increment_nonce(&addr);
        assert_eq!(state.get_nonce(&addr), 2);
    }

    #[test]
    fn test_storage() {
        let mut state = State::new();
        let contract = Address::zero();

        assert_eq!(state.get_storage(contract, b"key".into()), None);

        state.set_storage(contract, b"key".to_vec(), b"value".to_vec());
        assert_eq!(
            state.get_storage(contract, b"key".into()),
            Some(b"value".to_vec())
        );

        state.remove_storage(contract, b"key".into());
        assert_eq!(state.get_storage(contract, b"key".into()), None);
    }
}
