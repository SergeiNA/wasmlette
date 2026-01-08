//! SDK for writing smart contracts
//!
//! Provides ergonomic API for contract authors:
//! - Macros for contract entry points
//! - Helper functions for storage, events
//! - Compile to wasm32-unknown-unknown

pub mod api;
pub mod macros;

// Re-export commonly used items
pub use api::*;
