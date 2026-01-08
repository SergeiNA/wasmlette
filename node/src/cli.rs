//! Command-line interface

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "wasmlette")]
#[command(about = "Wasmlette WASM smart contract runtime", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new blockchain
    Init {
        /// Data directory
        #[arg(short, long, default_value = "./data")]
        data_dir: PathBuf,
    },

    /// Deploy a smart contract
    Deploy {
        /// Path to WASM file
        wasm_file: PathBuf,

        /// Sender address
        #[arg(short, long)]
        from: String,
    },

    /// Call a smart contract function
    Call {
        /// Contract address
        contract: String,

        /// Function name
        function: String,

        /// Arguments (JSON)
        #[arg(short, long)]
        args: Option<String>,

        /// Sender address
        #[arg(short, long)]
        from: String,
    },

    /// Query contract state
    Query {
        /// Contract address
        contract: String,

        /// Storage key
        key: String,
    },

    /// Start the node
    Run {
        /// Data directory
        #[arg(short, long, default_value = "./data")]
        data_dir: PathBuf,

        /// RPC port
        #[arg(short, long, default_value = "8545")]
        rpc_port: u16,
    },
}

impl Cli {
    pub fn execute(&self) -> Result<()> {
        match &self.command {
            Commands::Init { data_dir } => {
                tracing::info!("Initializing blockchain at {:?}", data_dir);
                // TODO: Implement initialization
                println!("✓ Blockchain initialized");
                Ok(())
            }

            Commands::Deploy { wasm_file, from } => {
                tracing::info!("Deploying contract from {:?}", wasm_file);
                tracing::info!("Sender: {}", from);
                // TODO: Implement deployment
                println!("✓ Contract deployed");
                Ok(())
            }

            Commands::Call {
                contract,
                function,
                args,
                from,
            } => {
                tracing::info!("Calling {}::{}", contract, function);
                tracing::info!("Args: {:?}", args);
                tracing::info!("From: {}", from);
                // TODO: Implement call
                println!("✓ Function called");
                Ok(())
            }

            Commands::Query { contract, key } => {
                tracing::info!("Querying {} key: {}", contract, key);
                // TODO: Implement query
                println!("✓ Value: <not implemented>");
                Ok(())
            }

            Commands::Run { data_dir, rpc_port } => {
                tracing::info!("Starting node...");
                tracing::info!("Data directory: {:?}", data_dir);
                tracing::info!("RPC port: {}", rpc_port);
                // TODO: Implement node runtime
                println!("✓ Node started (press Ctrl+C to stop)");
                Ok(())
            }
        }
    }
}
