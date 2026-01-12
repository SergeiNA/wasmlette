//! JSON-RPC server for Wasmlette node

use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::server::{Server, ServerHandle};
use std::sync::{Arc, Mutex};
use wasmlette_blockchain::Address;
use wasmlette_node::WasmletteNode;

/// Wasmlette JSON-RPC API
#[rpc(server)]
pub trait WasmletteRpc {
    /// Get balance of an account
    #[method(name = "wlt_getBalance")]
    fn get_balance(&self, address: String) -> RpcResult<String>;

    /// Transfer tokens between accounts
    #[method(name = "wlt_transfer")]
    fn transfer(&self, from: String, to: String, amount: String) -> RpcResult<String>;

    /// Deploy a contract
    #[method(name = "wlt_deploy")]
    fn deploy(
        &self,
        deployer: String,
        wasm_hex: String,
        init_args_hex: String,
    ) -> RpcResult<String>;

    /// Call a contract method
    #[method(name = "wlt_call")]
    fn call(
        &self,
        caller: String,
        contract: String,
        method: String,
        args_hex: String,
    ) -> RpcResult<String>;

    /// Get block by number
    #[method(name = "wlt_getBlock")]
    fn get_block(&self, block_number: u64) -> RpcResult<Option<BlockInfo>>;

    /// Get current chain height
    #[method(name = "wlt_getHeight")]
    fn get_height(&self) -> RpcResult<u64>;

    /// Get account nonce
    #[method(name = "wlt_getNonce")]
    fn get_nonce(&self, address: String) -> RpcResult<u64>;

    /// Get transaction receipt
    #[method(name = "wlt_getReceipt")]
    fn get_receipt(&self, tx_hash: String) -> RpcResult<Option<ReceiptInfo>>;

    /// Create an account with initial balance (for testing)
    #[method(name = "wlt_createAccount")]
    fn create_account(&self, seed: u8, balance: String) -> RpcResult<String>;
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct BlockInfo {
    pub number: u64,
    pub hash: String,
    pub parent_hash: String,
    pub timestamp: u64,
    pub transactions_count: usize,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ReceiptInfo {
    pub success: bool,
    pub gas_used: u64,
    pub contract_address: Option<String>,
    pub error_message: Option<String>,
}

/// RPC server implementation
pub struct WasmletteRpcImpl {
    node: Arc<Mutex<WasmletteNode>>,
}

impl WasmletteRpcImpl {
    pub fn new(node: Arc<Mutex<WasmletteNode>>) -> Self {
        Self { node }
    }
}

impl WasmletteRpcServer for WasmletteRpcImpl {
    fn get_balance(&self, address: String) -> RpcResult<String> {
        let addr = parse_address(&address)?;
        let node = self.node.lock().map_err(|e| {
            jsonrpsee::types::ErrorObjectOwned::owned(
                -32603,
                format!("Failed to acquire node lock: {}", e),
                None::<()>,
            )
        })?;
        let balance = node.get_balance(&addr);
        Ok(balance.to_string())
    }

    fn transfer(&self, from: String, to: String, amount: String) -> RpcResult<String> {
        let from_addr = parse_address(&from)?;
        let to_addr = parse_address(&to)?;
        let amount_val = amount.parse::<u64>().map_err(|_| {
            jsonrpsee::types::ErrorObjectOwned::owned(-32602, "Invalid amount", None::<()>)
        })?;

        let mut node = self.node.lock().map_err(|e| {
            jsonrpsee::types::ErrorObjectOwned::owned(
                -32603,
                format!("Failed to acquire node lock: {}", e),
                None::<()>,
            )
        })?;
        let receipt = node.transfer(from_addr, to_addr, amount_val).map_err(|e| {
            jsonrpsee::types::ErrorObjectOwned::owned(
                -32603,
                format!("Transfer failed: {}", e),
                None::<()>,
            )
        })?;

        Ok(format!(
            "{{\"success\": {}, \"gas_used\": {}}}",
            receipt.success, receipt.gas_used
        ))
    }

    fn deploy(
        &self,
        deployer: String,
        wasm_hex: String,
        init_args_hex: String,
    ) -> RpcResult<String> {
        let deployer_addr = parse_address(&deployer)?;
        let wasm_code = hex::decode(wasm_hex.trim_start_matches("0x")).map_err(|_| {
            jsonrpsee::types::ErrorObjectOwned::owned(-32602, "Invalid WASM hex", None::<()>)
        })?;
        let init_args = hex::decode(init_args_hex.trim_start_matches("0x")).map_err(|_| {
            jsonrpsee::types::ErrorObjectOwned::owned(-32602, "Invalid args hex", None::<()>)
        })?;

        let mut node = self.node.lock().map_err(|e| {
            jsonrpsee::types::ErrorObjectOwned::owned(
                -32603,
                format!("Failed to acquire node lock: {}", e),
                None::<()>,
            )
        })?;
        let receipt = node
            .deploy_contract(deployer_addr, wasm_code, init_args, 1_000_000)
            .map_err(|e| {
                jsonrpsee::types::ErrorObjectOwned::owned(
                    -32603,
                    format!("Deploy failed: {}", e),
                    None::<()>,
                )
            })?;

        let contract_addr = receipt
            .contract_address
            .map(|a| format!("0x{}", hex::encode(a.as_bytes())))
            .unwrap_or_else(|| "null".to_string());

        Ok(format!(
            "{{\"success\": {}, \"contract_address\": \"{}\", \"gas_used\": {}}}",
            receipt.success, contract_addr, receipt.gas_used
        ))
    }

    fn call(
        &self,
        caller: String,
        contract: String,
        method: String,
        args_hex: String,
    ) -> RpcResult<String> {
        let caller_addr = parse_address(&caller)?;
        let contract_addr = parse_address(&contract)?;
        let args = hex::decode(args_hex.trim_start_matches("0x")).map_err(|_| {
            jsonrpsee::types::ErrorObjectOwned::owned(-32602, "Invalid args hex", None::<()>)
        })?;

        let mut node = self.node.lock().map_err(|e| {
            jsonrpsee::types::ErrorObjectOwned::owned(
                -32603,
                format!("Failed to acquire node lock: {}", e),
                None::<()>,
            )
        })?;
        let receipt = node
            .call_contract(caller_addr, contract_addr, method, args, 1_000_000)
            .map_err(|e| {
                jsonrpsee::types::ErrorObjectOwned::owned(
                    -32603,
                    format!("Call failed: {}", e),
                    None::<()>,
                )
            })?;

        let return_data_hex = format!("0x{}", hex::encode(&receipt.return_data));
        Ok(format!(
            "{{\"success\": {}, \"return_data\": \"{}\", \"gas_used\": {}}}",
            receipt.success, return_data_hex, receipt.gas_used
        ))
    }

    fn get_block(&self, block_number: u64) -> RpcResult<Option<BlockInfo>> {
        let node = self.node.lock().map_err(|e| {
            jsonrpsee::types::ErrorObjectOwned::owned(
                -32603,
                format!("Failed to acquire node lock: {}", e),
                None::<()>,
            )
        })?;
        let block = node.get_block(block_number);

        Ok(block.map(|b| BlockInfo {
            number: b.number,
            hash: format!("0x{}", hex::encode(b.hash())),
            parent_hash: format!("0x{}", hex::encode(b.parent_hash)),
            timestamp: b.timestamp,
            transactions_count: b.transactions.len(),
        }))
    }

    fn get_height(&self) -> RpcResult<u64> {
        let node = self.node.lock().map_err(|e| {
            jsonrpsee::types::ErrorObjectOwned::owned(
                -32603,
                format!("Failed to acquire node lock: {}", e),
                None::<()>,
            )
        })?;
        Ok(node.height())
    }

    fn get_nonce(&self, address: String) -> RpcResult<u64> {
        let addr = parse_address(&address)?;
        let node = self.node.lock().map_err(|e| {
            jsonrpsee::types::ErrorObjectOwned::owned(
                -32603,
                format!("Failed to acquire node lock: {}", e),
                None::<()>,
            )
        })?;
        Ok(node.get_nonce(&addr))
    }

    fn get_receipt(&self, _tx_hash: String) -> RpcResult<Option<ReceiptInfo>> {
        // TODO: Implement receipt storage and retrieval
        Ok(None)
    }

    fn create_account(&self, seed: u8, balance: String) -> RpcResult<String> {
        let balance_val = balance.parse::<u64>().map_err(|_| {
            jsonrpsee::types::ErrorObjectOwned::owned(-32602, "Invalid balance", None::<()>)
        })?;

        let mut node = self.node.lock().map_err(|e| {
            jsonrpsee::types::ErrorObjectOwned::owned(
                -32603,
                format!("Failed to acquire node lock: {}", e),
                None::<()>,
            )
        })?;
        let address = node.create_account(seed, balance_val);

        Ok(format!("0x{}", hex::encode(address.as_bytes())))
    }
}

fn parse_address(s: &str) -> Result<Address, jsonrpsee::types::ErrorObjectOwned> {
    let bytes = hex::decode(s.trim_start_matches("0x")).map_err(|_| {
        jsonrpsee::types::ErrorObjectOwned::owned(-32602, "Invalid address hex", None::<()>)
    })?;

    if bytes.len() != 20 {
        return Err(jsonrpsee::types::ErrorObjectOwned::owned(
            -32602,
            "Address must be 20 bytes",
            None::<()>,
        ));
    }

    Ok(Address::from_slice(&bytes))
}

/// Start the JSON-RPC server
pub async fn start_server(
    node: Arc<Mutex<WasmletteNode>>,
    port: u16,
) -> anyhow::Result<ServerHandle> {
    let rpc_impl = WasmletteRpcImpl::new(node);

    let server = Server::builder()
        .build(format!("127.0.0.1:{}", port))
        .await?;

    let handle = server.start(rpc_impl.into_rpc());

    tracing::info!("JSON-RPC server listening on http://127.0.0.1:{}", port);

    Ok(handle)
}
