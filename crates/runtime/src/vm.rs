//! WASM VM wrapper using wasmtime

use crate::constants::MAX_CONTRACT_SIZE;
use anyhow::Result;
use std::cell::RefCell;
use std::rc::Rc;
use wasmlette_blockchain::transaction::Address;
use wasmlette_blockchain::State;
use wasmtime::*;

/// Runtime context passed to host functions
pub struct RuntimeContext {
    /// Address of the caller
    pub caller_address: Address,

    /// Address of the contract being executed
    pub contract_address: Address,

    /// Shared state
    pub state: Rc<RefCell<State>>,

    /// Remaining gas
    pub gas_remaining: u64,
}

/// WASM engine wrapper
pub struct WasmEngine {
    engine: Engine,
}

impl WasmEngine {
    /// Create a new WASM engine
    pub fn new() -> Result<Self> {
        let mut config = Config::new();

        // Enable fuel metering for gas tracking
        config.consume_fuel(true);

        // Disable features we don't need
        config.wasm_multi_memory(false);
        config.wasm_threads(false);

        config.wasm_memory64(false); // 32-bit memory only
        config.max_wasm_stack(MAX_CONTRACT_SIZE); // 1024KB stack

        let engine = Engine::new(&config)?;

        Ok(WasmEngine { engine })
    }

    /// Create a new store with the given context
    pub fn create_store(&self, context: RuntimeContext) -> Store<RuntimeContext> {
        Store::new(&self.engine, context)
    }

    /// Load a WASM module from bytes
    pub fn load_module(&self, wasm_code: &[u8]) -> Result<Module> {
        Module::new(&self.engine, wasm_code)
    }

    /// Get a reference to the engine
    pub fn engine(&self) -> &Engine {
        &self.engine
    }
}

impl Default for WasmEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create WASM engine")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = WasmEngine::new();
        assert!(engine.is_ok());
    }

    #[test]
    fn test_store_creation() {
        let engine = WasmEngine::new().unwrap();
        let state = Rc::new(RefCell::new(State::new()));
        let context = RuntimeContext {
            caller_address: Address::zero(),
            contract_address: Address::zero(),
            state,
            gas_remaining: 1000,
        };

        let _store = engine.create_store(context);
    }
}
