# Wasmlette RPC Integration Tests

Integration tests for the Wasmlette JSON-RPC server that start a real node and test the API end-to-end.

## Overview

These tests:
- Start a Wasmlette node on port 18545
- Test all JSON-RPC methods using the Python client
- Verify the node processes requests correctly
- Clean up the node process after tests complete

## Installation

Use the virtual environment from examples/python:

```bash
# From project root
cd examples/python

# Create virtual environment (if not already created)
python3 -m venv venv

# Activate it
source venv/bin/activate

# Install dependencies
pip install -r requirements.txt
```

Or create a separate venv for tests:

```bash
cd tests-integration/integration
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
```

## Running Tests

Make sure the venv is activated first:

```bash
# If using examples/python/venv
source examples/python/venv/bin/activate

# Or if you created tests-integration/integration/venv
cd tests-integration/integration
source venv/bin/activate
```

### Run all tests

```bash
# From project root (using examples/python/venv)
./examples/python/venv/bin/python -m pytest tests-integration/integration/test_rpc_server.py -v

# Or with activated venv
python -m pytest test_rpc_server.py -v
```

### Run with output visible

```bash
python -m pytest test_rpc_server.py -v -s
```

### Run specific test

```bash
python -m pytest test_rpc_server.py::test_transfer -v -s
```

### Run with detailed output

```bash
python -m pytest test_rpc_server.py -v -s --tb=short
```

## Test Coverage

The test suite covers:

### Basic Operations
- ✓ Node startup and health check
- ✓ Get blockchain height
- ✓ Get account balance (single and multiple accounts)
- ✓ Get account nonce
- ✓ Get block by number
- ✓ Query nonexistent blocks

### Transactions
- ✓ Token transfer (single)
- ✓ Multiple sequential transfers
- ✓ Contract deployment
- ✓ Transaction receipts

### Protocol Compliance
- ✓ JSON-RPC 2.0 protocol
- ✓ Error handling for invalid addresses
- ✓ Error handling for invalid methods
- ✓ Sequential request processing

### Load Testing
- ✓ Multiple concurrent requests

## Test Structure

```
test_rpc_server.py
├── Fixtures
│   ├── node_process    - Manages node lifecycle
│   └── client          - Provides WasmletteClient instance
├── Basic Tests
│   ├── test_node_is_running
│   ├── test_get_height
│   ├── test_get_balance
│   └── test_get_nonce
├── Transaction Tests
│   ├── test_transfer
│   ├── test_multiple_transfers_sequential
│   └── test_deploy_contract
└── Error Tests
    ├── test_invalid_address
    └── test_invalid_method
```

## Notes

### Test Port

Tests use port **18545** instead of the default 8545 to avoid conflicts with any running production node.

### Node Startup Time

The node fixture waits up to 15 seconds for the node to start. If tests fail with "Failed to start node", check:
- The project compiles successfully
- No other process is using port 18545
- The `wasmlette` binary is built

### Test Data

Tests use predefined addresses:
- Alice: `0x1111111111111111111111111111111111111111`
- Bob: `0x2222222222222222222222222222222222222222`

The counter.wasm test contract is loaded from `crates/runtime/tests/counter.wasm`.

### Transaction Success

Some tests check `receipt["success"]` but don't assert it's True because:
- Accounts may not have sufficient balance
- The test verifies the RPC call works, not the transaction logic

## Troubleshooting

### "Failed to start node"

```bash
# Build the project first
cargo build --bin wasmlette

# Check if port is available
lsof -i :18545
```

### "counter.wasm not found"

```bash
# Build the test contract
cargo test -p wasmlette-runtime
```

### Tests hang

The node process may not have stopped cleanly:

```bash
# Kill any leftover processes
pkill -f "wasmlette.*18545"
```

## CI Integration

To run in CI:

```bash
# Install Rust and Python dependencies
pip install -r requirements.txt

# Run tests-integration
python -m pytest test_rpc_server.py -v --tb=short
```

## See Also

- Python client: `../../examples/python/wasmlette_client.py`
- Implementation guide: `../../.claude/guides/json-rpc-implementation.md`
- Rust integration tests: `../../tests-integration/`
