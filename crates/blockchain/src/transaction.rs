//! Transaction types and processing

use serde::{Deserialize, Serialize};

const ADDRESS_LENGTH: usize = 20;

/// 20-byte address (similar to Ethereum)
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    bincode::Encode,
    bincode::Decode,
)]
pub struct Address([u8; ADDRESS_LENGTH]);

impl Address {
    pub const LENGTH: usize = ADDRESS_LENGTH;
    /// Create an address from a slice
    pub fn from_slice(bytes: &[u8]) -> Self {
        let mut addr = [0u8; ADDRESS_LENGTH];
        let len = bytes.len().min(ADDRESS_LENGTH);
        addr[..len].copy_from_slice(&bytes[..len]);
        Address(addr)
    }

    /// Create a zero address
    pub fn zero() -> Self {
        Address([0u8; ADDRESS_LENGTH])
    }

    /// Get the address as bytes
    pub fn as_bytes(&self) -> &[u8; ADDRESS_LENGTH] {
        &self.0
    }

    /// Generate a random address (for testing)
    #[cfg(test)]
    pub fn random() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; ADDRESS_LENGTH];
        rng.fill(&mut bytes);
        Address(bytes)
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(self.0))
    }
}

/// Different types of transactions
#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub enum TransactionKind {
    /// Deploy a new contract
    Deploy {
        wasm_code: Vec<u8>,
        init_args: Vec<u8>,
    },

    /// Call an existing contract
    Call {
        contract: Address,
        method: String,
        args: Vec<u8>,
    },

    /// Transfer native tokens
    Transfer { to: Address, amount: u64 },
}

/// A transaction in the blockchain
#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct Transaction {
    /// Sender's address
    pub sender: Address,

    /// Nonce for replay protection
    pub nonce: u64,

    /// Type of transaction
    pub kind: TransactionKind,

    /// Maximum gas to use
    pub gas_limit: u64,

    /// Price per unit of gas in units(micro-tokens) per 1 gas
    pub gas_price: u64,
}

impl Transaction {
    /// Create a new transaction
    pub fn new(
        sender: Address,
        nonce: u64,
        kind: TransactionKind,
        gas_limit: u64,
        gas_price: u64,
    ) -> Self {
        Transaction {
            sender,
            nonce,
            kind,
            gas_limit,
            gas_price,
        }
    }

    /// Calculate the hash of this transaction
    /// Uses bincode for deterministic binary serialization
    pub fn hash(&self) -> [u8; 32] {
        let serialized = bincode::encode_to_vec(self, bincode::config::standard())
            .expect("Failed to serialize transaction");
        blake3::hash(&serialized).into()
    }
}

/// Transaction execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionReceipt {
    /// Hash of the transaction
    pub tx_hash: [u8; 32],

    /// Whether execution was successful
    pub success: bool,

    /// Gas used
    pub gas_used: u64,

    /// Return data from contract execution
    pub return_data: Vec<u8>,

    /// Error message if failed
    pub error_message: Option<String>,

    /// Contract address if this was a Deploy transaction
    pub contract_address: Option<Address>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_creation() {
        let addr = Address::zero();
        assert_eq!(addr.as_bytes(), &[0u8; ADDRESS_LENGTH]);
    }

    #[test]
    fn test_address_random() {
        let addr = Address::random();
        assert_ne!(addr.as_bytes(), &[0u8; ADDRESS_LENGTH]);
    }

    #[test]
    fn test_transaction_hash() {
        {
            let tx = Transaction::new(
                Address::zero(),
                0,
                TransactionKind::Transfer {
                    to: Address::zero(),
                    amount: 100u64,
                },
                1000,
                1u64,
            );

            let hash1 = tx.hash();
            let hash2 = tx.hash();
            assert_eq!(hash1, hash2, "Hash should be deterministic");
        }
        {
            let tx_0 = Transaction::new(
                Address::zero(),
                0,
                TransactionKind::Transfer {
                    to: Address::zero(),
                    amount: 100u64,
                },
                1000,
                1u64,
            );
            let tx_1 = Transaction::new(
                Address::zero(),
                1,
                TransactionKind::Transfer {
                    to: Address::zero(),
                    amount: 100u64,
                },
                1000,
                1u64,
            );
            assert_ne!(tx_0.hash(), tx_1.hash(), "Hash should be deterministic");
        }
    }

    #[test]
    fn test_transaction_receipt() {
        let tx = Transaction::new(
            Address::zero(),
            0,
            TransactionKind::Transfer {
                to: Address::zero(),
                amount: 100u64,
            },
            1000,
            1u64,
        );

        let receipt = TransactionReceipt {
            tx_hash: tx.hash(),
            success: true,
            gas_used: 21000,
            return_data: vec![],
            error_message: None,
            contract_address: None,
        };

        assert_eq!(receipt.tx_hash, tx.hash());
        assert!(receipt.success);
        assert_eq!(receipt.gas_used, 21000);
    }
}
