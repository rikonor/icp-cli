use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use thiserror::Error;
use wasmtime::component::Func;

/// Errors that can occur during function registry operations
#[derive(Debug, Error)]
pub enum FunctionRegistryError {
    /// Function reference not found
    #[error("function reference not found: {0}")]
    NotFound(String),

    /// Function reference already exists
    #[error("function reference already exists: {0}")]
    AlreadyExists(String),
}

/// Registry for tracking function references between extensions
pub struct FunctionRegistry {
    /// Map from reference key to function reference
    references: HashMap<String, Arc<Mutex<Option<Func>>>>,
}

impl FunctionRegistry {
    /// Create a new function registry
    pub fn new() -> Self {
        Self {
            references: HashMap::new(),
        }
    }

    /// Register a function reference
    ///
    /// # Arguments
    ///
    /// * `key` - Unique identifier for the function reference
    /// * `fref` - Function reference to register
    ///
    /// # Returns
    ///
    /// * `Ok(())` if registration succeeded
    /// * `Err(FunctionRegistryError::AlreadyExists)` if key already exists
    pub fn register(
        &mut self,
        key: String,
        fref: Arc<Mutex<Option<Func>>>,
    ) -> Result<(), FunctionRegistryError> {
        if self.references.contains_key(&key) {
            return Err(FunctionRegistryError::AlreadyExists(key));
        }

        self.references.insert(key, fref);
        Ok(())
    }

    /// Resolve a function reference
    ///
    /// # Arguments
    ///
    /// * `k` - Key of the function reference to resolve
    /// * `f` - Function to resolve the reference to
    ///
    /// # Returns
    ///
    /// * `Ok(())` if resolution succeeded
    /// * `Err(FunctionRegistryError::NotFound)` if key not found
    pub fn resolve(&self, k: &str, f: Func) -> Result<(), FunctionRegistryError> {
        let reference = self
            .references
            .get(k)
            .ok_or_else(|| FunctionRegistryError::NotFound(k.to_string()))?;

        let mut g = reference.lock().unwrap();
        *g = Some(f);
        Ok(())
    }

    /// Create a key for a function reference
    ///
    /// # Arguments
    ///
    /// * `interface` - Name of the interface containing the function
    /// * `function` - Name of the function
    ///
    /// # Returns
    ///
    /// A string key in the format "interface:function"
    pub fn create_key(interface: &str, function: &str) -> String {
        format!("{}:{}", interface, function)
    }

    /// Get the number of references
    pub fn len(&self) -> usize {
        self.references.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.references.is_empty()
    }

    /// Get the number of resolved references
    pub fn resolved_count(&self) -> usize {
        self.references
            .values()
            .filter(|r| r.lock().unwrap().is_some())
            .count()
    }

    /// Check if a reference exists
    pub fn contains(&self, k: &str) -> bool {
        self.references.contains_key(k)
    }

    /// Check if a reference is resolved
    pub fn is_resolved(&self, k: &str) -> bool {
        self.references
            .get(k)
            .map(|r| r.lock().unwrap().is_some())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_key() {
        let key = FunctionRegistry::create_key("math/lib", "add");
        assert_eq!(key, "math/lib:add");
    }

    #[test]
    fn test_register_and_resolve() {
        let mut registry = FunctionRegistry::new();
        let key = "test:func".to_string();
        let fref = Arc::new(Mutex::new(None));

        // Test registration
        assert!(registry.register(key.clone(), fref.clone()).is_ok());
        assert!(registry.contains(&key));
        assert!(!registry.is_resolved(&key));

        // Test duplicate registration
        assert!(matches!(
            registry.register(key.clone(), fref.clone()),
            Err(FunctionRegistryError::AlreadyExists(_))
        ));

        // Test resolution of non-existent key
        let dummy_func = unsafe { std::mem::zeroed() }; // Just for testing
        assert!(matches!(
            registry.resolve("nonexistent", dummy_func),
            Err(FunctionRegistryError::NotFound(_))
        ));
    }

    #[test]
    fn test_empty_and_len() {
        let mut registry = FunctionRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);

        let fref = Arc::new(Mutex::new(None));
        registry
            .register("test:func".to_string(), fref)
            .expect("Failed to register");

        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);
    }
}
