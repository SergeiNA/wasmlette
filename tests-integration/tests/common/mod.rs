//! Common test utilities and helpers

use std::cell::RefCell;
use std::rc::Rc;
use wasmlette_blockchain::transaction::{Transaction, TransactionKind};
use wasmlette_blockchain::{Address, State};
use wasmlette_runtime::ContractExecutor;

const DEFAULT_GAS_LIMIT: u64 = 1_000_000;
const DEFAULT_GAS_PRICE: u64 = 1;

/// Test fixture for contract testing
pub struct TestEnv {
    pub state: Rc<RefCell<State>>,
    pub executor: ContractExecutor,
}

impl TestEnv {
    /// Create a new test environment
    pub fn new() -> Self {
        Self {
            state: Rc::new(RefCell::new(State::new())),
            executor: ContractExecutor::new().unwrap(),
        }
    }

    /// Create an address with initial balance
    pub fn create_account(&self, seed: u8, balance: u64) -> Address {
        let address = Address::from_slice(&[seed; Address::LENGTH]);
        self.state.borrow_mut().set_balance(address, balance);
        address
    }

    /// Deploy a contract
    pub fn deploy_contract(
        &self,
        deployer: Address,
        nonce: u64,
        wasm_code: &[u8],
        init_args: Vec<u8>,
    ) -> Result<Address, String> {
        let deploy_tx = Transaction {
            sender: deployer,
            nonce,
            kind: TransactionKind::Deploy {
                wasm_code: wasm_code.to_vec(),
                init_args,
            },
            gas_limit: DEFAULT_GAS_LIMIT,
            gas_price: DEFAULT_GAS_PRICE,
        };

        let receipt = self
            .executor
            .execute_transaction(self.state.clone(), &deploy_tx)
            .map_err(|e| format!("Deploy transaction failed: {}", e))?;

        if !receipt.success {
            return Err(format!("Deploy failed: {:?}", receipt.error_message));
        }

        receipt
            .contract_address
            .ok_or_else(|| "No contract address in receipt".to_string())
    }

    /// Call a contract method
    pub fn call_contract(
        &self,
        caller: Address,
        nonce: u64,
        contract: Address,
        method: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, String> {
        let call_tx = Transaction {
            sender: caller,
            nonce,
            kind: TransactionKind::Call {
                contract,
                method: method.to_string(),
                args,
            },
            gas_limit: DEFAULT_GAS_LIMIT,
            gas_price: DEFAULT_GAS_PRICE,
        };

        let receipt = self
            .executor
            .execute_transaction(self.state.clone(), &call_tx)
            .map_err(|e| format!("Call transaction failed: {}", e))?;

        if !receipt.success {
            return Err(format!("Call failed: {:?}", receipt.error_message));
        }

        Ok(receipt.return_data)
    }

    /// Get storage value
    pub fn get_storage(&self, contract: Address, key: &[u8]) -> Option<Vec<u8>> {
        self.state.borrow().get_storage(contract, key.to_vec())
    }

    /// Get storage as u64
    pub fn get_storage_u64(&self, contract: Address, key: &[u8]) -> Option<u64> {
        self.get_storage(contract, key).and_then(|bytes| {
            if bytes.len() == 8 {
                let mut arr = [0u8; 8];
                arr.copy_from_slice(&bytes);
                Some(u64::from_le_bytes(arr))
            } else {
                None
            }
        })
    }

    /// Get balance of address
    pub fn get_balance(&self, address: &Address) -> u64 {
        self.state.borrow().get_balance(address)
    }

    /// Check if contract exists
    pub fn contract_exists(&self, address: &Address) -> bool {
        self.state.borrow().contract_exists(address)
    }
}

/// Helper to build contract method arguments
pub struct ArgsBuilder {
    data: Vec<u8>,
}

impl ArgsBuilder {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn add_address(mut self, addr: &Address) -> Self {
        self.data.extend_from_slice(addr.as_bytes());
        self
    }

    pub fn add_u64(mut self, value: u64) -> Self {
        self.data.extend_from_slice(&value.to_le_bytes());
        self
    }

    pub fn add_bytes(mut self, bytes: &[u8]) -> Self {
        self.data.extend_from_slice(bytes);
        self
    }

    pub fn build(self) -> Vec<u8> {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_creation() {
        let env = TestEnv::new();
        assert!(env.state.borrow().contract_exists(&Address::zero()) == false);
    }

    #[test]
    fn test_create_account() {
        let env = TestEnv::new();
        let alice = env.create_account(1, 1_000_000);
        assert_eq!(env.get_balance(&alice), 1_000_000);
    }

    #[test]
    fn test_args_builder() {
        let alice = Address::from_slice(&[1u8; Address::LENGTH]);
        let args = ArgsBuilder::new().add_address(&alice).add_u64(1000).build();

        assert_eq!(args.len(), 28); // 20 + 8
    }
}
