#!/usr/bin/env bash
# Run all tests-integration in the workspace

set -e

echo "Running tests..."

cargo test --workspace

echo "✓ All tests passed"
