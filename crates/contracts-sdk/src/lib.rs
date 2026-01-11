//! SDK for writing smart contracts
//!
//! Provides ergonomic API for contract authors:
//! - Macros for contract entry points
//! - Helper functions for storage, events
//! - Compile to wasm32-unknown-unknown

pub mod api;

// Re-export commonly used items
pub use api::*;

// Re-export procedural macros
pub use wasmlette_contracts_sdk_macros::{contract_call, contract_init, contract_query};
