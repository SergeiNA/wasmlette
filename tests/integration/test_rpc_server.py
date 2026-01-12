#!/usr/bin/env python3
"""
Integration tests for Wasmlette JSON-RPC server

These tests start a Wasmlette node and test the JSON-RPC API using the Python client.

Run with: python -m pytest test_rpc_server.py -v
Or:      python -m pytest test_rpc_server.py -v -s  (to see output)

Requirements:
    pip install requests pytest
"""

import pytest
import subprocess
import time
import requests
import json
import sys
import os
from pathlib import Path

# Add examples/python to path to import the client
examples_path = Path(__file__).parent.parent.parent / "examples" / "python"
sys.path.insert(0, str(examples_path))

from wasmlette_client import WasmletteClient


@pytest.fixture(scope="module")
def node_process():
    """
    Start the Wasmlette node for testing.

    This fixture starts the node before tests run and stops it after all tests complete.
    """
    # Find the project root (2 levels up from this file)
    project_root = Path(__file__).parent.parent.parent

    print("\n=== Starting Wasmlette node for testing ===")

    # Start node in background on test port to avoid conflicts
    process = subprocess.Popen(
        ["cargo", "run", "--bin", "wasmlette", "--", "run", "--rpc-port", "18545"],
        cwd=str(project_root),
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )

    # Wait for node to start and be ready
    max_attempts = 30
    for attempt in range(max_attempts):
        try:
            response = requests.post(
                "http://localhost:18545",
                json={
                    "jsonrpc": "2.0",
                    "method": "wlt_getHeight",
                    "params": [],
                    "id": 1
                },
                timeout=1
            )
            if response.status_code == 200:
                print(f"✓ Node started successfully after {attempt + 1} attempts")
                break
        except (requests.exceptions.ConnectionError, requests.exceptions.Timeout):
            time.sleep(0.5)
    else:
        process.kill()
        pytest.fail("Failed to start node after 30 attempts")

    yield process

    # Cleanup
    print("\n=== Stopping Wasmlette node ===")
    process.kill()
    process.wait()


@pytest.fixture
def client(node_process):
    """Create a client connected to the test node"""
    return WasmletteClient("http://localhost:18545")


def test_node_is_running(node_process):
    """Test that the node process is running"""
    assert node_process.poll() is None, "Node process should be running"


def test_get_height(client):
    """Test getting chain height"""
    height = client.get_height()
    assert isinstance(height, int), "Height should be an integer"
    assert height >= 0, "Height should be non-negative"
    print(f"  Chain height: {height}")


def test_get_balance(client):
    """Test getting account balance"""
    address = "0x1111111111111111111111111111111111111111"
    balance = client.get_balance(address)
    assert isinstance(balance, int), "Balance should be an integer"
    assert balance >= 0, "Balance should be non-negative"
    print(f"  Balance: {balance}")


def test_get_balance_multiple_accounts(client):
    """Test getting balances for multiple accounts"""
    addresses = [
        "0x1111111111111111111111111111111111111111",
        "0x2222222222222222222222222222222222222222",
        "0x3333333333333333333333333333333333333333",
    ]

    for addr in addresses:
        balance = client.get_balance(addr)
        assert isinstance(balance, int)
        assert balance >= 0
        print(f"  {addr}: {balance}")


def test_get_nonce(client):
    """Test getting account nonce"""
    address = "0x1111111111111111111111111111111111111111"
    nonce = client.get_nonce(address)

    assert isinstance(nonce, int), "Nonce should be an integer"
    assert nonce >= 0, "Nonce should be non-negative"
    print(f"  Nonce: {nonce}")


def test_get_block(client):
    """Test getting a block"""
    # Get genesis block
    block = client.get_block(0)

    assert block is not None, "Genesis block should exist"
    assert block["number"] == 0, "Block number should be 0"
    assert "hash" in block, "Block should have hash"
    assert "parent_hash" in block, "Block should have parent_hash"
    assert "timestamp" in block, "Block should have timestamp"
    assert "transactions_count" in block, "Block should have transactions_count"

    print(f"  Block 0: hash={block['hash'][:18]}..., tx_count={block['transactions_count']}")


def test_get_nonexistent_block(client):
    """Test getting a block that doesn't exist"""
    height = client.get_height()
    # Try to get a block far beyond current height
    block = client.get_block(height + 1000)
    assert block is None, "Nonexistent block should return None"


def test_transfer(client):
    """Test transferring tokens"""
    # Create funded accounts for testing
    alice = client.create_account(1, 10_000_000)
    bob = client.create_account(2, 0)

    print(f"  Alice: {alice}, Bob: {bob}")

    initial_height = client.get_height()
    print(f"  Initial height: {initial_height}")

    # Check initial balances
    alice_balance = client.get_balance(alice)
    print(f"  Alice initial balance: {alice_balance}")

    # Perform transfer
    receipt = client.transfer(alice, bob, 1000)

    assert "success" in receipt, "Receipt should have success field"
    assert "gas_used" in receipt, "Receipt should have gas_used field"
    assert isinstance(receipt["gas_used"], int), "gas_used should be integer"
    assert receipt["success"] is True, "Transfer should succeed with funded account"

    print(f"  Transfer success: {receipt['success']}, gas_used: {receipt['gas_used']}")

    # Check height increased
    new_height = client.get_height()
    assert new_height == initial_height + 1, "Height should increase by 1"
    print(f"  New height: {new_height}")

    # Check bob received the tokens
    bob_balance = client.get_balance(bob)
    assert bob_balance == 1000, f"Bob should have 1000 tokens, got {bob_balance}"
    print(f"  Bob final balance: {bob_balance}")


def test_multiple_transfers_sequential(client):
    """Test multiple transfers in sequence"""
    # Create funded accounts for testing
    alice = client.create_account(3, 10_000_000)
    bob = client.create_account(4, 0)

    print(f"  Alice: {alice}, Bob: {bob}")

    initial_height = client.get_height()
    print(f"  Initial height: {initial_height}")

    # Make 3 transfers
    for i in range(3):
        receipt = client.transfer(alice, bob, 100)
        print(f"  Transfer {i+1}: success={receipt['success']}, gas={receipt['gas_used']}")
        assert receipt["success"] is True, f"Transfer {i+1} should succeed"

    # Height should have increased by 3
    final_height = client.get_height()
    expected_height = initial_height + 3
    assert final_height == expected_height, f"Expected height {expected_height}, got {final_height}"
    print(f"  Final height: {final_height} (+3)")

    # Check bob received all tokens
    bob_balance = client.get_balance(bob)
    assert bob_balance == 300, f"Bob should have 300 tokens, got {bob_balance}"
    print(f"  Bob final balance: {bob_balance}")


def test_deploy_contract(client):
    """Test deploying a contract"""
    # Create funded account for deploying
    deployer = client.create_account(5, 100_000_000)
    print(f"  Deployer: {deployer}")

    # Use the counter.wasm test contract
    # Find it relative to project root
    project_root = Path(__file__).parent.parent.parent
    wasm_path = project_root / "crates" / "runtime" / "tests" / "counter.wasm"

    if not wasm_path.exists():
        pytest.skip(f"Counter WASM not found at {wasm_path}")

    with open(wasm_path, "rb") as f:
        wasm_bytes = f.read()
        wasm_hex = "0x" + wasm_bytes.hex()

    print(f"  Deploying contract ({len(wasm_bytes)} bytes)...")

    result = client.deploy(deployer, wasm_hex, "0x")

    assert "success" in result, "Result should have success field"
    assert "gas_used" in result, "Result should have gas_used field"
    assert "contract_address" in result, "Result should have contract_address field"
    assert result["success"] is True, "Deploy should succeed with funded account"

    print(f"  Deploy success: {result['success']}")
    print(f"  Gas used: {result['gas_used']}")
    print(f"  Contract address: {result['contract_address']}")

    assert result["contract_address"] is not None, "Successful deploy should have contract address"


def test_invalid_address(client):
    """Test that invalid addresses cause errors"""
    with pytest.raises(Exception) as exc_info:
        client.get_balance("invalid_address")

    assert "RPC Error" in str(exc_info.value), "Should raise RPC error for invalid address"
    print(f"  Got expected error: {exc_info.value}")


def test_invalid_method(client):
    """Test calling a non-existent RPC method"""
    with pytest.raises(Exception) as exc_info:
        client._call("wlt_nonExistentMethod", [])

    error_msg = str(exc_info.value)
    assert "RPC Error" in error_msg, "Should raise RPC error"
    print(f"  Got expected error: {exc_info.value}")


def test_get_receipt(client):
    """Test getting transaction receipt"""
    # Note: This returns None currently as receipt storage is not implemented
    receipt = client.get_receipt("0x1234567890abcdef")
    # Current implementation returns None
    assert receipt is None or isinstance(receipt, dict), "Receipt should be None or dict"
    print(f"  Receipt: {receipt}")


def test_rpc_protocol_direct(node_process):
    """Test JSON-RPC 2.0 protocol directly without client"""
    # Test the raw JSON-RPC protocol
    payload = {
        "jsonrpc": "2.0",
        "method": "wlt_getHeight",
        "params": [],
        "id": 42
    }

    response = requests.post(
        "http://localhost:18545",
        json=payload,
        headers={"Content-Type": "application/json"}
    )

    assert response.status_code == 200, "Should return 200 OK"

    result = response.json()
    assert "jsonrpc" in result, "Response should have jsonrpc field"
    assert result["jsonrpc"] == "2.0", "Should be JSON-RPC 2.0"
    assert "result" in result or "error" in result, "Should have result or error"
    assert "id" in result, "Response should have id field"
    assert result["id"] == 42, "Response id should match request id"

    print(f"  Raw response: {json.dumps(result, indent=2)}")


def test_concurrent_requests(client):
    """Test that multiple requests work (even though processed sequentially)"""
    # Make several requests in quick succession
    # The server processes them one at a time, but they should all succeed

    results = []
    for i in range(5):
        height = client.get_height()
        results.append(height)
        print(f"  Request {i+1}: height = {height}")

    # All requests should succeed
    assert len(results) == 5, "All 5 requests should complete"

    # Heights should be non-decreasing (may increase if transfers were processed)
    for i in range(1, len(results)):
        assert results[i] >= results[i-1], "Height should never decrease"


if __name__ == "__main__":
    # Run tests with verbose output
    pytest.main([__file__, "-v", "-s"])
