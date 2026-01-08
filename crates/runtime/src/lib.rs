//! WASM execution runtime
//!
//! This crate provides the WASM execution environment:
//! - WASM engine wrapper (wasmtime)
//! - Gas metering and fuel tracking
//! - Host functions exposed to contracts
//! - Contract execution orchestration

pub mod executor;
pub mod gas_meter;
pub mod host_api;
pub mod memory;
pub mod vm;
pub mod constants;
mod token_units;

// Re-export main types
pub use executor::ContractExecutor;
pub use gas_meter::{DeployGasMeter, TransferGasMeter};
pub use vm::{RuntimeContext, WasmEngine};
