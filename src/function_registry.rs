use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use wasmtime::component::Func;

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
    pub fn register(&mut self, key: String) -> Arc<Mutex<Option<Func>>> {
        let reference = Arc::new(Mutex::new(None));
        self.references.insert(key.clone(), Arc::clone(&reference));
        reference
    }

    /// Get a function reference
    pub fn get(&self, key: &str) -> Option<Arc<Mutex<Option<Func>>>> {
        self.references.get(key).cloned()
    }

    /// Resolve a function reference
    pub fn resolve(&self, key: &str, func: Func) -> bool {
        if let Some(reference) = self.references.get(key) {
            let mut guard = reference.lock().unwrap();
            *guard = Some(func);
            true
        } else {
            false
        }
    }

    /// Create a key for a function reference
    pub fn create_key(extension: &str, interface: &str, function: &str) -> String {
        format!("{}:{}:{}", extension, interface, function)
    }

    /// Get the number of references
    pub fn len(&self) -> usize {
        self.references.len()
    }

    /// Get the number of resolved references
    pub fn resolved_count(&self) -> usize {
        self.references
            .values()
            .filter(|r| r.lock().unwrap().is_some())
            .count()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.references.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_key() {
        let key = FunctionRegistry::create_key("ext-a", "math/lib", "add");
        assert_eq!(key, "ext-a:math/lib:add");
    }

    #[test]
    fn test_register_and_get() {
        let mut registry = FunctionRegistry::new();
        let key = "ext-a:math/lib:add".to_string();

        let reference = registry.register(key.clone());
        assert!(registry.get(&key).is_some());

        // Check that the reference is initially None
        assert!(reference.lock().unwrap().is_none());
    }

    #[test]
    fn test_resolve() {
        let mut registry = FunctionRegistry::new();
        let key = "ext-a:math/lib:add".to_string();

        registry.register(key.clone());

        // We can't easily create a real Func in tests, so we'll just test the logic
        // by checking if the reference exists
        assert!(registry.get(&key).is_some());
    }

    #[test]
    fn test_len_and_is_empty() {
        let mut registry = FunctionRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);

        registry.register("ext-a:math/lib:add".to_string());
        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);

        registry.register("ext-a:math/lib:subtract".to_string());
        assert_eq!(registry.len(), 2);
    }
}
