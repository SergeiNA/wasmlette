#!/usr/bin/env python3
"""
Simple example demonstrating Wasmlette client usage.

Before running:
1. Install dependencies: pip install -r requirements.txt
2. Start the Wasmlette node: cargo run --bin wasmlette run
3. Run this script: python simple_example.py
"""

from wasmlette_client import WasmletteClient


def main():
    # Connect to local node
    client = WasmletteClient("http://localhost:8545")

    print("=== Simple Wasmlette Example ===\n")

    try:
        # Check node is running
        height = client.get_height()
        print(f"✓ Connected to node at height: {height}\n")

        # Example addresses
        alice = "0x1111111111111111111111111111111111111111"
        bob = "0x2222222222222222222222222222222222222222"

        # Query balances
        print("Checking balances:")
        alice_balance = client.get_balance(alice)
        bob_balance = client.get_balance(bob)
        print(f"  Alice: {alice_balance}")
        print(f"  Bob:   {bob_balance}")

        # Query nonces
        print("\nChecking nonces:")
        alice_nonce = client.get_nonce(alice)
        bob_nonce = client.get_nonce(bob)
        print(f"  Alice: {alice_nonce}")
        print(f"  Bob:   {bob_nonce}")

        # Get latest blocks
        print("\nRecent blocks:")
        for i in range(min(3, height + 1)):
            block = client.get_block(i)
            if block:
                print(f"  Block {block['number']}: {block['transactions_count']} tx")

        print("\n✓ Example completed successfully!")

    except Exception as e:
        print(f"\n✗ Error: {e}")
        print("\nMake sure the Wasmlette node is running:")
        print("  cargo run --bin wasmlette run")


if __name__ == "__main__":
    main()
