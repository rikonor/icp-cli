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
    pub fn register(&mut self, key: String, fref: Arc<Mutex<Option<Func>>>) {
        self.references.insert(
            key,  // key
            fref, // reference
        );
    }

    /// Resolve a function reference
    pub fn resolve(&self, k: &str, f: Func) -> bool {
        if let Some(reference) = self.references.get(k) {
            let mut g = reference.lock().unwrap();
            *g = Some(f);
            true
        } else {
            false
        }
    }

    /// Create a key for a function reference
    pub fn create_key(interface: &str, function: &str) -> String {
        format!("{}:{}", interface, function)
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_key() {
        let key = FunctionRegistry::create_key("math/lib", "add");
        assert_eq!(key, "math/lib:add");
    }
}
