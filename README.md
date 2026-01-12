# Wasmlette

A minimal WebAssembly-based smart contract runtime built with Rust, featuring gas metering, native token support, JSON-RPC API, and a comprehensive SDK for contract development.

## Features

- **WASM Runtime**: Powered by Wasmtime for secure contract execution
- **Gas Metering**: Precise gas tracking with upfront reservation and refunds
- **Token System**: Native tokens with 6-decimal precision for gas payments
- **JSON-RPC API**: Full-featured JSON-RPC 2.0 server for node interaction
- **Rich SDK**: Developer-friendly contract SDK with storage, context, and crypto APIs
- **State Management**: Thread-safe key-value storage with contract isolation
- **Testing Framework**: Comprehensive integration tests with helper utilities
- **Python Client**: Ready-to-use Python client for easy integration

## Project Structure

```
wasmlette/
├── crates/
│   ├── blockchain/          # Core blockchain primitives
│   │   └── src/
│   │       ├── block.rs         # Block structure and validation
│   │       ├── manager.rs       # Blockchain manager
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
│   ├── networking/          # P2P networking (future)
│   └── node/                # High-level node API
│
├── demon/                   # Node daemon with JSON-RPC server
│   ├── src/
│   │   ├── main.rs              # Server entry point
│   │   ├── rpc/                 # JSON-RPC implementation
│   │   └── cli.rs               # CLI configuration
│   └── Cargo.toml
│
├── examples/python/         # Python client examples
│   ├── wasmlette_client.py      # Python RPC client
│   └── simple_example.py        # Usage examples
│
└── tests-integration/       # Integration tests
```

## Quick Start

### 1. Run the Node

```bash
# Build and run the node with JSON-RPC server
cargo run --release -p demon -- --rpc-port 8545

# Or with custom data directory
cargo run --release -p demon -- --data-dir ./data --rpc-port 8545
```

The node will start with:
- JSON-RPC server listening on `http://127.0.0.1:8545`
- Genesis block initialized
- Ready to accept transactions

**Graceful Shutdown:**
- Press `Ctrl+C` or send `SIGTERM` signal to stop the node
- The node will:
  1. Stop accepting new RPC requests
  2. Wait for in-flight requests to complete
  3. Clean up resources
  4. Exit gracefully with status message

### 2. Interact via Python Client

```bash
# Setup Python environment
cd examples/python
./setup_venv.sh

# Run examples
source venv/bin/activate
python simple_example.py
```

### 3. Or use curl

```bash
# Get chain height
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"wlt_getHeight","params":[],"id":1}'

# Get balance
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"wlt_getBalance","params":["0x0101010101010101010101010101010101010101"],"id":1}'
```

## JSON-RPC API

The node exposes a full JSON-RPC 2.0 API:

### Account Management
- `wlt_getBalance(address: String) -> String` - Get account balance
- `wlt_getNonce(address: String) -> u64` - Get account nonce
- `wlt_createAccount(seed: u8, balance: String) -> String` - Create test account

### Transactions
- `wlt_transfer(from: String, to: String, amount: String) -> String` - Transfer tokens
- `wlt_deploy(deployer: String, wasm_hex: String, init_args_hex: String) -> String` - Deploy contract
- `wlt_call(caller: String, contract: String, method: String, args_hex: String) -> String` - Call contract

### Blockchain
- `wlt_getBlock(block_number: u64) -> Option<BlockInfo>` - Get block by number
- `wlt_getHeight() -> u64` - Get current chain height
- `wlt_getReceipt(tx_hash: String) -> Option<ReceiptInfo>` - Get transaction receipt (TODO)

### Python Client Example

```python
from wasmlette_client import WasmletteClient

# Connect to node
client = WasmletteClient("http://127.0.0.1:8545")

# Create accounts
alice = client.create_account(seed=1, balance=1000000)
bob = client.create_account(seed=2, balance=0)

# Transfer tokens
receipt = client.transfer(alice, bob, 500000)
print(f"Transfer success: {receipt['success']}")

# Deploy contract
with open("counter.wasm", "rb") as f:
    wasm_code = f.read().hex()

result = client.deploy(alice, wasm_code, "")
contract_addr = result['contract_address']

# Call contract
result = client.call(alice, contract_addr, "increment", "")
print(f"Increment success: {result['success']}")
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

### Rust Tests

Run all tests:
```bash
# Run all unit and integration tests
cargo test --workspace

# Run with output
cargo test --workspace -- --nocapture
```

Run specific test suites:
```bash
# Runtime tests (executor, gas, host functions)
cargo test -p wasmlette-runtime

# WASM integration tests
cargo test -p wasmlette-runtime --test wasm_integration_test
cargo test -p wasmlette-runtime --test test_transfer
cargo test -p wasmlette-runtime --test host_function_test

# Node tests
cargo test -p wasmlette-node

# Token unit tests
cargo test -p wasmlette-tokens
```

### Python Integration Tests

```bash
# Setup (first time only)
cd tests/integration
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt

# Build contract for integration tests
cargo build --release --target wasm32-unknown-unknown -p simple_token

# Run integration tests (requires running node)
# Terminal 1: Start node
cargo run --release -p wasmlette-demon -- --rpc-port 18545

# Terminal 2: Run tests
cd tests/integration
source venv/bin/activate
pytest test_rpc_server.py -v

# Or run a specific test
pytest test_rpc_server.py::test_deploy_and_call_contract -v -s
```

**What's tested:**
- Node connectivity and basic RPC calls
- Account creation and balance queries
- Native token transfers
- Contract deployment (simple_token)
- Contract method calls (balance_of, transfer, total_supply)
- Gas metering and refunds
- Error handling for invalid requests

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
1. **Validation**: Check balance ≥ `gas_limit × gas_price`
2. **Reservation**: `gas_limit × gas_price` deducted upfront
3. **Execution**: Gas consumed during contract execution
4. **Refund**: `(gas_limit - gas_used) × gas_price` returned to sender
5. **Nonce**: Account nonce incremented for valid transactions

### Gas Costs
- **Transfer**: 11,000 gas
- **Deploy**: Base cost + per-byte code cost
- **Contract Call**: Wasm execution cost + host function costs
- **Storage Write**: Per-byte storage cost
- **Storage Read**: Per-byte read cost
- **Hash (Blake3)**: Per-byte hash cost
- **Balance Query**: Fixed cost

### Example Transaction
```rust
Transaction {
    gas_limit: 100_000,    // Max gas units
    gas_price: 1,          // 1 unit per gas
    // Max cost: 100,000 units (0.1 token)
}

// If only 60,000 gas used:
// Charged: 60,000 units (0.06 tokens)
// Refunded: 40,000 units (0.04 tokens)
```

### Failed Transactions
- **Invalid nonce/signature**: Transaction rejected, no gas charged, nonce not incremented
- **Insufficient balance**: Transaction rejected, no gas charged
- **Execution failure**: Fixed failure fee charged (5,000 gas), nonce incremented

## Architecture

### Thread Safety
The runtime uses `Arc<Mutex<State>>` for thread-safe state management, enabling concurrent RPC requests. While this introduces some locking overhead, it's necessary for:
- **Wasmtime constraints**: Host functions require shared state semantics
- **Nested calls**: WASM can call back into host functions recursively
- **RPC server**: Multiple concurrent JSON-RPC requests

### Error Handling
Production code uses proper error propagation with:
- `Result<T, E>` with `?` operator for operations that can fail
- `.map_err()` for context-specific error messages
- Graceful degradation in query functions (return safe defaults)
- JSON-RPC error codes for API failures

## Operational Features

### Graceful Shutdown
The node daemon implements proper signal handling for clean shutdown:

```bash
# Start node
cargo run --release -p wasmlette-demon -- --rpc-port 8545

# Graceful shutdown (any of these):
# - Press Ctrl+C
# - Send SIGTERM: kill -TERM <pid>
# - Docker stop (sends SIGTERM by default)
```

**Shutdown Behavior:**
1. Catches SIGINT (Ctrl+C) and SIGTERM signals
2. Stops accepting new connections
3. Allows in-flight RPC requests to complete
4. Logs shutdown progress
5. Exits with status code 0

**Testing:**
```bash
# Automated test
./scripts/test_graceful_shutdown.sh
```

Cross-platform support:
- Unix/Linux/macOS: Handles both SIGINT and SIGTERM
- Windows: Handles Ctrl+C gracefully

## Development

### Project Structure Overview
- **demon**: Node daemon with JSON-RPC server (main binary)
- **crates/node**: High-level node API
- **crates/runtime**: WASM execution and transaction processing
- **crates/blockchain**: Block, transaction, and state primitives
- **crates/contracts-sdk**: Contract development SDK
- **crates/tokens**: Token denomination utilities
- **crates/storage**: State persistence (in-memory)
- **examples/python**: Python client and examples
- **tests/integration**: Python integration tests

### Building the Project
```bash
# Build all workspace members
cargo build --workspace --release

# Build only the node daemon
cargo build --release -p demon

# Build contracts
cargo build --release --target wasm32-unknown-unknown -p counter
```

### Code Style
- Use `tracing` for logging (not `println!`)
- Proper error handling (no `.unwrap()` in production code)
- Thread-safe state management with `Arc<Mutex<>>`
- Graceful shutdown with signal handling
- Comprehensive tests for all features

## Contributing

Contributions are welcome! Please ensure:
1. All tests pass: `cargo test --workspace`
2. Code is formatted: `cargo fmt --all`
3. No clippy warnings: `cargo clippy --workspace --all-targets`
4. Contracts build: `cargo build --release --target wasm32-unknown-unknown -p counter -p simple_token -p tester`
5. Integration tests pass (requires contracts built and node running)

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
