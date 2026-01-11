//! Configuration management

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Data directory
    pub data_dir: PathBuf,

    /// RPC port
    pub rpc_port: u16,

    /// Enable networking
    pub networking_enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            data_dir: PathBuf::from("./data"),
            rpc_port: 8545,
            networking_enabled: false,
        }
    }
}
