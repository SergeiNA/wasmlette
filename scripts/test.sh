#!/usr/bin/env bash
# Run all tests in the workspace

set -e

echo "Running tests..."

cargo test --workspace

echo "✓ All tests passed"
