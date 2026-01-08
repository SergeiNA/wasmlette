//! State persistence layer
//!
//! Provides abstract storage interface with multiple backends:
//! - In-memory storage (for testing)
//! - Persistent storage (optional: sled)

pub mod in_memory;
pub mod kv;

#[cfg(feature = "sled")]
pub mod sled_store;

pub use in_memory::InMemoryStorage;
pub use kv::Storage;

#[cfg(feature = "sled")]
pub use sled_store::SledStorage;
