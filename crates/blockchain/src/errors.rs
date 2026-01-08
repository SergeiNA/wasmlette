//! Error types for blockchain operations

use crate::transaction::Address;
use thiserror::Error;

/// Errors that can occur during blockchain operations
#[derive(Debug, Error)]
pub enum BlockchainError {
    #[error("Insufficient balance: address {address}, required {required}, available {available}")]
    InsufficientBalance {
        address: Address,
        required: u64,
        available: u64,
    },

    #[error("Contract already exists at address {0}")]
    ContractAlreadyExists(Address),

    #[error("Contract not found at address {0}")]
    ContractNotFound(Address),

    #[error("Invalid nonce: expected {expected}, got {got}")]
    InvalidNonce { expected: u64, got: u64 },

    #[error("Gas limit exceeded")]
    GasLimitExceeded,

    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),

    #[error("Block validation failed: {0}")]
    BlockValidationFailed(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),
}
