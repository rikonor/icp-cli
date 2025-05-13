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
}

impl Default for FunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl FunctionRegistry {
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

    /// Look up a resolved function reference by its interface and function name.
    ///
    /// # Arguments
    ///
    /// * `interface_name` - Name of the interface containing the function.
    /// * `function_name` - Name of the function.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(Func))` if the function is registered and resolved.
    /// * `Ok(None)` if the function is registered but not yet resolved.
    /// * `Err(FunctionRegistryError::NotFound)` if the key is not registered.
    pub fn lookup(
        &self,
        interface_name: &str,
        function_name: &str,
    ) -> Result<Option<Func>, FunctionRegistryError> {
        // Create the key using the established format
        let key = Self::create_key(interface_name, function_name);

        // Get the Arc<Mutex<Option<Func>>> from the map
        let reference_arc = self
            .references
            .get(&key)
            .ok_or_else(|| FunctionRegistryError::NotFound(key.clone()))?; // Return NotFound error if key doesn't exist

        // Lock the mutex to access the Option<Func>
        // We expect the lock to succeed unless there's poisoning, which we'll unwrap for now.
        // A production scenario might handle poisoning more gracefully.
        let guard = reference_arc.lock().map_err(|_| {
            // Handle potential mutex poisoning, perhaps return a specific error
            // For now, let's treat it as NotFound or a new error type
            FunctionRegistryError::NotFound(format!("Mutex poisoned for key: {}", key))
        })?;

        // Return the Option<Func> contained within the guard.
        // Since Func is Copy, we can copy it out of the Option.
        Ok(*guard) // Dereferencing the MutexGuard gives &Option<Func>, copying Func if Some.
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
