//! Networking layer (P2P and RPC)
//!
//! Optional component for Phase 5:
//! - P2P protocol for demon communication
//! - JSON-RPC server for client interaction
//! - Block synchronization

pub mod node;
pub mod p2p;
pub mod rpc;
pub mod sync;

pub use node::Node;
