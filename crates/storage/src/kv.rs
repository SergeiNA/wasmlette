//! Key-value storage trait

use anyhow::Result;

/// Abstract key-value storage interface
pub trait Storage: Send + Sync {
    /// Get a value by key
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;

    /// Set a value for a key
    fn set(&mut self, key: &[u8], value: Vec<u8>) -> Result<()>;

    /// Delete a key
    fn delete(&mut self, key: &[u8]) -> Result<()>;

    /// Check if a key exists
    fn exists(&self, key: &[u8]) -> Result<bool> {
        Ok(self.get(key)?.is_some())
    }

    /// Clear all data (useful for testing)
    fn clear(&mut self) -> Result<()>;
}
