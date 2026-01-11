# Wasmlette

A minimal WebAssembly-based smart contract runtime built with Rust, featuring gas metering, native token support, and a comprehensive SDK for contract development.

## Features

- **WASM Runtime**: Powered by Wasmtime for secure contract execution
- **Gas Metering**: Precise gas tracking with upfront reservation and refunds
- **Token System**: Native tokens with 6-decimal precision for gas payments
- **Rich SDK**: Developer-friendly contract SDK with storage, context, and crypto APIs
- **State Management**: Efficient key-value storage with contract isolation
- **Testing Framework**: Comprehensive integration tests with helper utilities

## Project Structure

```
wasmlette/
├── crates/
│   ├── blockchain/          # Core blockchain primitives
│   │   └── src/
│   │       ├── block.rs         # Block structure and validation
│   │       ├── chain.rs         # Blockchain state machine
│   │       ├── transaction.rs   # Transaction types
│   │       ├── state.rs         # Account and contract state
│   │       └── errors.rs        # Error types
│   │
│   ├── runtime/             # WASM execution environment
│   │   └── src/
│   │       ├── vm.rs            # Wasmtime engine wrapper
│   │       ├── executor.rs      # Transaction execution
│   │       ├── gas_meter.rs     # Gas metering
│   │       ├── gas_fee_calculator.rs  # Gas cost calculation
│   │       ├── host_api.rs      # Host functions for contracts
│   │       └── memory.rs        # Memory management
│   │
│   ├── tokens/              # Token denomination utilities
│   │   └── src/
│   │       └── lib.rs           # Token unit conversions
│   │
│   ├── contracts-sdk/       # SDK for writing smart contracts
│   │   ├── src/
│   │   │   ├── storage.rs       # Storage API
│   │   │   ├── context.rs       # Execution context
│   │   │   └── crypto.rs        # Cryptographic functions
│   │   └── examples/
│   │       ├── counter/         # Simple counter contract
│   │       ├── simple_token/    # ERC20-like token
│   │       └── tester/          # Testing utilities
│   │
│   ├── storage/             # State persistence layer
│   ├── networking/          # P2P and RPC (future)
│   └── node/                # Node binary
│
├── tests-integration/       # Integration tests
└── demon/                   # Demonstration node
```

## Building Contracts

### Prerequisites

Install the WASM target:
```bash
rustup target add wasm32-unknown-unknown
```

### Build Example Contracts

Build all example contracts:
```bash
cargo build --release --target wasm32-unknown-unknown \
  -p counter \
  -p simple_token \
  -p tester
```

Build a specific contract:
```bash
cargo build --release --target wasm32-unknown-unknown -p counter
```

The compiled WASM files will be in:
```
target/wasm32-unknown-unknown/release/
├── counter.wasm
├── simple_token.wasm
└── tester.wasm
```

### Optimize WASM (Optional)

For production, optimize with `wasm-opt`:
```bash
# Install wasm-opt (from binaryen)
cargo install wasm-opt

# Optimize
wasm-opt -Oz -o counter_opt.wasm target/wasm32-unknown-unknown/release/counter.wasm
```

## Example Contracts

### Counter Contract
A simple counter demonstrating state storage:
- `init()` - Initialize counter to 0
- `increment()` - Increment counter by 1
- `get_count() -> u64` - Read current count

**Location**: `crates/contracts-sdk/examples/counter/`

### Simple Token Contract
An ERC20-like fungible token with:
- `init(initial_supply: u64)` - Deploy with initial supply to deployer
- `balance_of(address: &[u8; 20]) -> u64` - Query balance
- `transfer(to: &[u8; 20], amount: u64) -> i32` - Transfer tokens
- `total_supply() -> u64` - Get total supply
- `transfer_count() -> u64` - Get total number of transfers

**Features:**
- Uses `get_caller()` for secure authentication
- Prevents self-transfers and zero-amount transfers
- Tracks total supply and transfer statistics

**Location**: `crates/contracts-sdk/examples/simple_token/`

### Tester Contract
Utility contract for testing host functions:
- Storage operations (`test_set_count`, `test_get_count`)
- Balance queries (`test_get_balance`)
- Cryptographic hashing (`test_hash`)
- Caller identification (`test_get_caller`)

**Location**: `crates/contracts-sdk/examples/tester/`

## Running Tests

Run all tests:
```bash
cargo test --workspace
```

Run specific test suites:
```bash
# Runtime tests
cargo test -p wasmlette-runtime

# Integration tests
cargo test -p wasmlette-integration-tests

# Token unit tests
cargo test -p wasmlette-tokens
```

Run with output:
```bash
cargo test -- --nocapture
```

## Writing Your Own Contract

### 1. Create a new contract

```bash
cargo new --lib my-contract
cd my-contract
```

### 2. Configure Cargo.toml

```toml
[package]
name = "my-contract"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasmlette-contracts-sdk = { path = "../contracts-sdk" }

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
```

### 3. Write your contract

```rust
#![no_std]

use wasmlette_contracts_sdk::{storage, context};

#[no_mangle]
pub extern "C" fn init() {
    storage::set(b"owner", context::get_caller().as_bytes());
}

#[no_mangle]
pub extern "C" fn greet() -> u64 {
    // Your logic here
    42
}
```

### 4. Build to WASM

```bash
cargo build --release --target wasm32-unknown-unknown
```

## Gas & Fee System

### Token Units
- **Denomination**: 6 decimals (micro-tokens)
- **1.0 token** = 1,000,000 units
- **Smallest unit**: 0.000001 tokens

### Gas Mechanism
1. **Reservation**: `gas_limit × gas_price` deducted upfront
2. **Execution**: Gas consumed during contract execution
3. **Refund**: `(gas_limit - gas_used) × gas_price` returned to sender

### Example Transaction
```rust
Transaction {
    gas_limit: 100_000,    // Max gas units
    gas_price: 10,         // 10 units per gas
    // Max cost: 1,000,000 units (1.0 token)
}

// If only 60,000 gas used:
// Charged: 600,000 units
// Refunded: 400,000 units
```

## License

MIT License

Copyright (c) 2025 Wasmlette Contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
