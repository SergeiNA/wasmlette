//! Command-line interface

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Result type for CLI execution
pub enum CliResult {
    /// Start RPC server (needs async context)
    RunServer { data_dir: PathBuf, rpc_port: u16 },
}

#[derive(Parser)]
#[command(name = "wasmlette")]
#[command(about = "Wasmlette WASM smart contract runtime", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the demon
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
    pub fn execute(&self) -> Result<CliResult> {
        match &self.command {
            Commands::Run { data_dir, rpc_port } => Ok(CliResult::RunServer {
                data_dir: data_dir.clone(),
                rpc_port: *rpc_port,
            }),
        }
    }
}
