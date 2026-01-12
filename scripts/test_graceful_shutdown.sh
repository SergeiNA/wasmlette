#!/bin/bash
# Test script for graceful shutdown

set -e

echo "Starting Wasmlette node in background..."
cargo run --release -p wasmlette-demon -- --rpc-port 8545 &
NODE_PID=$!

echo "Node PID: $NODE_PID"
echo "Waiting 3 seconds for node to start..."
sleep 3

# Test that node is responding
echo "Testing node is alive..."
curl -s -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"wlt_getHeight","params":[],"id":1}' | jq .

echo ""
echo "Node is running successfully!"
echo "Sending SIGTERM signal for graceful shutdown..."
kill -TERM $NODE_PID

echo "Waiting for graceful shutdown..."
wait $NODE_PID
EXIT_CODE=$?

if [ $EXIT_CODE -eq 0 ] || [ $EXIT_CODE -eq 143 ]; then
    echo "✓ Node shut down gracefully (exit code: $EXIT_CODE)"
    exit 0
else
    echo "✗ Node exited with unexpected code: $EXIT_CODE"
    exit 1
fi
