//! In-memory storage implementation

use crate::Storage;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// In-memory storage backend
#[derive(Clone)]
pub struct InMemoryStorage {
    data: Arc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>,
}

impl InMemoryStorage {
    /// Create a new in-memory storage
    pub fn new() -> Self {
        InMemoryStorage {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl Storage for InMemoryStorage {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let data = self.data.read().unwrap();
        Ok(data.get(key).cloned())
    }

    fn set(&mut self, key: &[u8], value: Vec<u8>) -> Result<()> {
        let mut data = self.data.write().unwrap();
        data.insert(key.to_vec(), value);
        Ok(())
    }

    fn delete(&mut self, key: &[u8]) -> Result<()> {
        let mut data = self.data.write().unwrap();
        data.remove(key);
        Ok(())
    }

    fn clear(&mut self) -> Result<()> {
        let mut data = self.data.write().unwrap();
        data.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_storage() {
        let mut storage = InMemoryStorage::new();

        // Test set and get
        storage.set(b"key1", b"value1".to_vec()).unwrap();
        assert_eq!(storage.get(b"key1").unwrap(), Some(b"value1".to_vec()));

        // Test exists
        assert!(storage.exists(b"key1").unwrap());
        assert!(!storage.exists(b"key2").unwrap());

        // Test delete
        storage.delete(b"key1").unwrap();
        assert_eq!(storage.get(b"key1").unwrap(), None);

        // Test clear
        storage.set(b"key1", b"value1".to_vec()).unwrap();
        storage.set(b"key2", b"value2".to_vec()).unwrap();
        storage.clear().unwrap();
        assert_eq!(storage.get(b"key1").unwrap(), None);
        assert_eq!(storage.get(b"key2").unwrap(), None);
    }
}
