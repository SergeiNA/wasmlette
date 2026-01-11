//! Core blockchain data structures and logic
//!
//! This crate provides the fundamental building blocks for the blockchain:
//! - Block structure and validation
//! - Transaction types and processing
//! - State management (accounts, balances, nonces)
//! - Merkle tree for state commitment

pub mod block;
pub mod builder;
pub mod crypto;
pub mod errors;
pub mod manager;
pub mod merkle;
pub mod state;
pub mod transaction;
pub mod utils;

// Re-export main types
pub use block::Block;
pub use builder::BlockBuilder;
pub use crypto::{Keypair, TransactionSignature, verify_signature};
pub use errors::BlockchainError;
pub use manager::BlockchainManager;
pub use state::State;
pub use transaction::{Address, Transaction, TransactionKind, TransactionReceipt};
