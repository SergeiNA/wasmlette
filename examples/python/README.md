# Wasmlette Python Client Examples

Python client library and examples for interacting with Wasmlette node via JSON-RPC.

## Installation

### Quick Setup (Recommended)

```bash
# Run the setup script
./setup_venv.sh
```

### Manual Setup

Create and activate a virtual environment:

```bash
# Create virtual environment
python3 -m venv venv

# Activate it (Unix/macOS)
source venv/bin/activate

# Or on Windows
# venv\Scripts\activate

# Install dependencies
pip install -r requirements.txt
```

## Prerequisites

Make sure the Wasmlette node is running:

```bash
# From the project root
cargo run --bin wasmlette run

# Or with custom port
cargo run --bin wasmlette run --rpc-port 9000
```

## Files

- **wasmlette_client.py** - Python client library for Wasmlette JSON-RPC API
- **simple_example.py** - Basic example showing how to query the node
- **requirements.txt** - Python dependencies

## Usage

### Using the Client Library

```python
from wasmlette_client import WasmletteClient

# Connect to node
client = WasmletteClient("http://localhost:8545")

# Get chain height
height = client.get_height()
print(f"Height: {height}")

# Check balance
balance = client.get_balance("0x1111111111111111111111111111111111111111")
print(f"Balance: {balance}")

# Transfer tokens
receipt = client.transfer(
    "0x1111111111111111111111111111111111111111",
    "0x2222222222222222222222222222222222222222",
    1000
)
print(f"Transfer success: {receipt['success']}")
```

### Running the Examples

```bash
# Run the simple example
python simple_example.py

# Run the full client demo
python wasmlette_client.py
```

## API Methods

All methods correspond to the Wasmlette JSON-RPC API:

- `get_height()` - Get current blockchain height
- `get_balance(address)` - Get account balance
- `get_nonce(address)` - Get account nonce
- `get_block(number)` - Get block by number
- `transfer(from, to, amount)` - Transfer tokens
- `deploy(deployer, wasm_hex, init_args_hex)` - Deploy contract
- `call(caller, contract, method, args_hex)` - Call contract method
- `get_receipt(tx_hash)` - Get transaction receipt

## Address Format

All addresses must be hex strings with `0x` prefix and 20 bytes (40 hex characters):

```
0x1111111111111111111111111111111111111111
```

## Error Handling

The client raises exceptions for RPC errors:

```python
try:
    balance = client.get_balance("invalid")
except Exception as e:
    print(f"Error: {e}")
```

## See Also

- Main implementation guide: `../../.claude/guides/json-rpc-implementation.md`
- Integration tests: `../../tests-integration/tests/node/test_rpc.py`
