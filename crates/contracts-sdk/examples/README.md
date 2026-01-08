# Contract Examples

This directory contains example smart contracts demonstrating different approaches to contract development.

## Examples

### With SDK (Recommended for Production)

These contracts use the `wasmlette-contracts-sdk` for safe, high-level APIs:

- **`counter/`** - Simple counter contract using SDK
  - Shows storage API: `storage::get()`, `storage::set()`
  - Safe abstractions and error handling
  - Build: `cd counter && cargo build --target wasm32-unknown-unknown --release`

- **`token/`** - Basic ERC20-like token contract
  - Shows complex state management
  - Multiple storage keys per account
  - Build: `cd token && cargo build --target wasm32-unknown-unknown --release`

### Without SDK (Raw WASM)

These contracts call host functions directly, useful for:
- Testing host function implementations
- Learning low-level WASM interface
- Understanding what the SDK abstracts away
- Minimizing binary size

- **`counter-raw/`** - Counter without SDK (**see Guide 08b**)
  - Direct `extern "C"` host function calls
  - Manual memory and pointer handling
  - Useful for testing runtime implementation
  - Build: `cd counter-raw && cargo build --target wasm32-unknown-unknown --release`
  - See: `.claude/guides/08b-raw-contracts-no-sdk.md`

## Quick Start

### Prerequisites
```bash
# Install WASM target (one time)
rustup target add wasm32-unknown-unknown
```

### Build All Examples
```bash
# Build SDK examples
cd counter && cargo build --target wasm32-unknown-unknown --release
cd ../token && cargo build --target wasm32-unknown-unknown --release

# Build raw example
cd ../counter_raw && cargo build --target wasm32-unknown-unknown --release
```

### Test with Runtime
```bash
# Copy compiled WASM to runtime tests
cp counter/target/wasm32-unknown-unknown/release/counter.wasm ../../runtime/tests/
cp counter_raw/target/wasm32-unknown-unknown/release/counter_raw.wasm ../../runtime/tests/

# Run integration tests
cd ../../../
cargo test --package wasmlette-runtime
```

## Comparison

| Feature | SDK (`counter/`) | Raw (`counter-raw/`) |
|---------|------------------|----------------------|
| **Safety** | Safe Rust | Unsafe pointers |
| **API** | `storage::get()` | `extern "C" fn get_storage(...)` |
| **Binary Size** | ~15-20 KB | ~2-5 KB |
| **Dev Speed** | Fast | Slow |
| **Use Case** | Production | Testing/Learning |

## File Locations

After building, WASM binaries are located at:
```
counter/target/wasm32-unknown-unknown/release/counter.wasm
counter-raw/target/wasm32-unknown-unknown/release/counter_raw.wasm
token/target/wasm32-unknown-unknown/release/token.wasm
```

## Documentation

- **Guide 08b** - Raw Contracts (No SDK): `.claude/guides/08b-raw-contracts-no-sdk.md`
- **Guide 09** - Contract SDK: `.claude/guides/09-contract-sdk.md`

## Next Steps

1. Study `counter-raw/` to understand the low-level interface
2. Compare with `counter/` to see SDK benefits
3. Build your own contracts using the SDK
4. Deploy and test with the CLI: `wasmlette-node deploy contract.wasm`
