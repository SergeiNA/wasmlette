//! Wasmlette demon binary

use anyhow::Result;
use clap::Parser;
use std::sync::{Arc, Mutex};
use tokio::signal;
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

    // Wait for shutdown signal
    shutdown_signal().await;

    // Graceful shutdown
    tracing::info!("Shutdown signal received, stopping server...");
    handle.stop()?;

    tracing::info!("Waiting for server to finish...");
    handle.stopped().await;

    tracing::info!("Server stopped gracefully");
    println!("✓ Node stopped gracefully");

    Ok(())
}

/// Wait for shutdown signal (Ctrl+C or SIGTERM)
///
/// This function blocks until one of the following signals is received:
/// - SIGINT (Ctrl+C): Triggered when user presses Ctrl+C
/// - SIGTERM: Standard termination signal (used by systemd, Docker, etc.)
///
/// On Unix systems (Linux/macOS), both signals are handled.
/// On Windows, only Ctrl+C is handled.
///
/// # Graceful Shutdown Flow
/// 1. User presses Ctrl+C or sends SIGTERM
/// 2. This function returns
/// 3. Main function calls `handle.stop()` to stop accepting new connections
/// 4. In-flight requests are allowed to complete
/// 5. Resources are cleaned up
/// 6. Process exits with code 0
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C signal");
        },
        _ = terminate => {
            tracing::info!("Received SIGTERM signal");
        },
    }
}
