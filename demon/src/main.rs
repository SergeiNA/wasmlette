//! Wasmlette demon binary

use anyhow::Result;
use clap::Parser;

mod cli;
mod config;

use cli::Cli;

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse CLI arguments
    let cli = Cli::parse();

    // Execute command
    cli.execute()?;

    Ok(())
}
