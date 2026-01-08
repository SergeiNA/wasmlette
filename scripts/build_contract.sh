#!/usr/bin/env bash
# Build a contract to WASM

set -e

CONTRACT_NAME=$1

if [ -z "$CONTRACT_NAME" ]; then
    echo "Usage: $0 <contract_name>"
    echo "Example: $0 counter"
    exit 1
fi

CONTRACT_DIR="crates/contracts-sdk/examples/$CONTRACT_NAME"

if [ ! -d "$CONTRACT_DIR" ]; then
    echo "Error: Contract '$CONTRACT_NAME' not found in $CONTRACT_DIR"
    exit 1
fi

echo "Building contract: $CONTRACT_NAME"
cd "$CONTRACT_DIR"

# Build for wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown

WASM_FILE="target/wasm32-unknown-unknown/release/${CONTRACT_NAME}.wasm"

if [ -f "../../../../$WASM_FILE" ]; then
    echo "✓ Contract built successfully"
    echo "WASM file: $WASM_FILE"

    # Show file size
    SIZE=$(ls -lh "../../../../$WASM_FILE" | awk '{print $5}')
    echo "File size: $SIZE"
else
    echo "Error: WASM file not found"
    exit 1
fi
