#!/usr/bin/env bash
# Run the wasmlette node

set -e

echo "Starting Wasmlette node..."

# Build the node
cargo build --release --bin wasmlette

# Run the node
./target/release/wasmlette run "$@"
