//! Wasmlette demon binary

use anyhow::Result;
use clap::Parser;
use std::sync::{Arc, Mutex};
use wasmlette_node::WasmletteNode;

mod cli;
mod rpc;

use cli::{Cli, CliResult};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse CLI arguments
    let cli = Cli::parse();

    // Execute command
    let CliResult::RunServer { data_dir, rpc_port } = cli.execute()?;

    tracing::info!("Starting Wasmlette node...");
    tracing::info!("Data directory: {:?}", data_dir);
    tracing::info!("RPC port: {}", rpc_port);

    // Create node
    let node = WasmletteNode::new()?;
    let node = Arc::new(Mutex::new(node));

    // Start RPC server
    let handle = rpc::start_server(node.clone(), rpc_port).await?;

    println!("✓ Node started (press Ctrl+C to stop)");

    // Keep server running
    handle.stopped().await;

    Ok(())
}
