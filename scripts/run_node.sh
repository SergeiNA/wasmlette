#!/usr/bin/env bash
# Run the wasmlette demon

set -e

echo "Starting Wasmlette node..."

# Build the demon
cargo build --release --bin wasmlette

# Run the demon
./target/release/wasmlette run "$@"
