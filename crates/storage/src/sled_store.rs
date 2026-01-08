//! Persistent storage using sled
//!
//! Only available when the "sled" feature is enabled

use crate::Storage;
use anyhow::Result;

/// Sled-based persistent storage
pub struct SledStorage {
    // TODO: Add sled::Db when implementing
}

impl SledStorage {
    /// Create a new sled storage at the given path
    pub fn new(_path: &str) -> Result<Self> {
        // TODO: Implement with sled
        unimplemented!("Sled storage not yet implemented")
    }
}

impl Storage for SledStorage {
    fn get(&self, _key: &[u8]) -> Result<Option<Vec<u8>>> {
        unimplemented!()
    }

    fn set(&mut self, _key: &[u8], _value: Vec<u8>) -> Result<()> {
        unimplemented!()
    }

    fn delete(&mut self, _key: &[u8]) -> Result<()> {
        unimplemented!()
    }

    fn clear(&mut self) -> Result<()> {
        unimplemented!()
    }
}
