#!/usr/bin/env bash
# Format all Rust code in the workspace

set -e

echo "Formatting code..."

cargo fmt --all

echo "✓ Code formatted"
