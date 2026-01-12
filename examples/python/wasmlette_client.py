#!/usr/bin/env python3
"""
Wasmlette JSON-RPC Python client

This client provides a simple interface to interact with a Wasmlette node via JSON-RPC 2.0.

Example usage:
    client = WasmletteClient()
    height = client.get_height()
    balance = client.get_balance("0x1111111111111111111111111111111111111111")
"""

import requests
import json
from typing import Optional, Dict, Any, List


class WasmletteClient:
    """Client for interacting with Wasmlette node via JSON-RPC"""

    def __init__(self, url: str = "http://localhost:8545"):
        """
        Initialize the client.

        Args:
            url: The URL of the Wasmlette node RPC endpoint
        """
        self.url = url
        self.request_id = 0

    def _call(self, method: str, params: List[Any] = None) -> Any:
        """
        Make a JSON-RPC call to the node.

        Args:
            method: The RPC method name
            params: List of parameters for the method

        Returns:
            The result from the RPC call

        Raises:
            Exception: If the RPC call fails or returns an error
        """
        self.request_id += 1

        payload = {
            "jsonrpc": "2.0",
            "method": method,
            "params": params or [],
            "id": self.request_id
        }

        response = requests.post(
            self.url,
            json=payload,
            headers={"Content-Type": "application/json"}
        )

        response.raise_for_status()
        result = response.json()

        if "error" in result:
            error = result["error"]
            raise Exception(f"RPC Error [{error.get('code', 'unknown')}]: {error.get('message', 'Unknown error')}")

        return result.get("result")

    def get_balance(self, address: str) -> int:
        """
        Get the balance of an account.

        Args:
            address: The account address (hex string with 0x prefix)

        Returns:
            The account balance as an integer
        """
        result = self._call("wlt_getBalance", [address])
        return int(result)

    def transfer(self, from_addr: str, to_addr: str, amount: int) -> Dict[str, Any]:
        """
        Transfer tokens from one account to another.

        Args:
            from_addr: Sender address (hex string with 0x prefix)
            to_addr: Recipient address (hex string with 0x prefix)
            amount: Amount to transfer

        Returns:
            Transaction receipt as a dictionary with keys:
            - success: bool
            - gas_used: int
        """
        result_str = self._call("wlt_transfer", [from_addr, to_addr, str(amount)])
        return json.loads(result_str)

    def deploy(self, deployer: str, wasm_hex: str, init_args_hex: str = "0x") -> Dict[str, Any]:
        """
        Deploy a WASM smart contract.

        Args:
            deployer: Deployer address (hex string with 0x prefix)
            wasm_hex: WASM bytecode as hex string (with 0x prefix)
            init_args_hex: Initialization arguments as hex string (with 0x prefix)

        Returns:
            Deployment receipt as a dictionary with keys:
            - success: bool
            - contract_address: str (hex with 0x prefix)
            - gas_used: int
        """
        result_str = self._call("wlt_deploy", [deployer, wasm_hex, init_args_hex])
        return json.loads(result_str)

    def call(self, caller: str, contract: str, method: str, args_hex: str = "0x") -> Dict[str, Any]:
        """
        Call a smart contract method.

        Args:
            caller: Caller address (hex string with 0x prefix)
            contract: Contract address (hex string with 0x prefix)
            method: Method name to call
            args_hex: Method arguments as hex string (with 0x prefix)

        Returns:
            Call receipt as a dictionary with keys:
            - success: bool
            - return_data: str (hex with 0x prefix)
            - gas_used: int
        """
        result_str = self._call("wlt_call", [caller, contract, method, args_hex])
        return json.loads(result_str)

    def get_block(self, block_number: int) -> Optional[Dict[str, Any]]:
        """
        Get a block by its number.

        Args:
            block_number: The block number

        Returns:
            Block information as a dictionary, or None if block doesn't exist.
            Block dict contains:
            - number: int
            - hash: str (hex with 0x prefix)
            - parent_hash: str (hex with 0x prefix)
            - timestamp: int
            - transactions_count: int
        """
        return self._call("wlt_getBlock", [block_number])

    def get_height(self) -> int:
        """
        Get the current blockchain height.

        Returns:
            The current block height as an integer
        """
        return self._call("wlt_getHeight", [])

    def get_nonce(self, address: str) -> int:
        """
        Get the transaction nonce for an account.

        Args:
            address: The account address (hex string with 0x prefix)

        Returns:
            The account's current nonce
        """
        return self._call("wlt_getNonce", [address])

    def get_receipt(self, tx_hash: str) -> Optional[Dict[str, Any]]:
        """
        Get a transaction receipt by hash.

        Args:
            tx_hash: Transaction hash (hex string with 0x prefix)

        Returns:
            Receipt information as a dictionary, or None if not found.
            Receipt dict contains:
            - success: bool
            - gas_used: int
            - contract_address: Optional[str]
            - error_message: Optional[str]
        """
        return self._call("wlt_getReceipt", [tx_hash])

    def create_account(self, seed: int, balance: int) -> str:
        """
        Create an account with initial balance (for testing).

        Args:
            seed: Seed byte (0-255) used to generate the address
            balance: Initial balance to set

        Returns:
            The created account address (hex string with 0x prefix)
        """
        return self._call("wlt_createAccount", [seed, str(balance)])


def main():
    """Example usage of the Wasmlette client"""

    # Create client
    client = WasmletteClient()

    print("=== Wasmlette Client Example ===\n")

    # Get chain height
    height = client.get_height()
    print(f"Chain height: {height}")

    # Example addresses
    alice = "0x1111111111111111111111111111111111111111"
    bob = "0x2222222222222222222222222222222222222222"

    # Check balance
    balance = client.get_balance(alice)
    print(f"Alice balance: {balance}")

    # Get nonce
    nonce = client.get_nonce(alice)
    print(f"Alice nonce: {nonce}")

    # Get genesis block
    block = client.get_block(0)
    if block:
        print(f"\nGenesis block:")
        print(f"  Number: {block['number']}")
        print(f"  Hash: {block['hash']}")
        print(f"  Timestamp: {block['timestamp']}")
        print(f"  Transactions: {block['transactions_count']}")

    # Transfer example (commented out - uncomment if you have funded accounts)
    # print(f"\nTransferring 1000 tokens from Alice to Bob...")
    # receipt = client.transfer(alice, bob, 1000)
    # print(f"Transfer receipt: {receipt}")

    # Deploy contract example (commented out - uncomment if you have a WASM file)
    # print(f"\nDeploying contract...")
    # with open("contract.wasm", "rb") as f:
    #     wasm_bytes = f.read()
    #     wasm_hex = "0x" + wasm_bytes.hex()
    #
    # deploy_result = client.deploy(alice, wasm_hex)
    # print(f"Deploy result: {deploy_result}")
    # contract_addr = deploy_result["contract_address"]
    #
    # # Call contract
    # print(f"\nCalling contract method...")
    # call_result = client.call(alice, contract_addr, "get_value", "0x")
    # print(f"Call result: {call_result}")


if __name__ == "__main__":
    main()
