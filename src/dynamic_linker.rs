use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use wasmtime::component::{Func, Instance, Linker, Val};
use wasmtime::Store;

use crate::function_registry::FunctionRegistry;
use crate::manifest::{ExportedInterface, ImportedInterface};

/// Error type for dynamic linking operations
#[derive(Debug, thiserror::Error)]
pub enum DynamicLinkingError {
    /// Function reference not resolved
    #[error("function reference not resolved: {0}")]
    UnresolvedReference(String),

    /// Unexpected error
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

/// Dynamic linker for WebAssembly components
pub struct DynamicLinker {
    /// Registry for function references
    registry: FunctionRegistry,
    /// Map from extension name to resolved status
    resolved_exports: HashMap<String, bool>,
}

impl DynamicLinker {
    /// Create a new dynamic linker
    pub fn new() -> Self {
        Self {
            registry: FunctionRegistry::new(),
            resolved_exports: HashMap::new(),
        }
    }

    /// Get a reference to the function registry
    pub fn registry(&self) -> &FunctionRegistry {
        &self.registry
    }

    /// Get a mutable reference to the function registry
    pub fn registry_mut(&mut self) -> &mut FunctionRegistry {
        &mut self.registry
    }

    /// Link imports for an extension
    pub fn link_imports<T>(
        &mut self,
        linker: &mut Linker<T>,
        extension_name: &str,
        imports: &[ImportedInterface],
    ) -> Result<()> {
        for import in imports {
            // Skip non-library interfaces
            if !import.name.ends_with("/lib") {
                continue;
            }

            // For each function in the interface
            for function_name in &import.functions {
                // Create a key for this function reference
                let key = FunctionRegistry::create_key(extension_name, &import.name, function_name);

                // Register the function reference
                let function_ref = self.registry.register(key);
                let function_name_clone = function_name.clone();

                // Use the ergonomic pattern to define the proxy function
                linker.instance(&import.name)?.func_new(
                    function_name,
                    move |mut store, params, results| {
                        // Get the function reference
                        let function_ref_guard = function_ref.lock().unwrap();
                        let f = function_ref_guard.as_ref().ok_or_else(|| {
                            DynamicLinkingError::UnresolvedReference(function_name_clone.clone())
                        })?;

                        // Forward the call to the actual function
                        f.call(&mut store, params, results)?;
                        f.post_return(&mut store)?;

                        Ok(())
                    },
                )?;
            }
        }

        Ok(())
    }

    /// Resolve exports for an extension
    pub fn resolve_exports<T>(
        &mut self,
        linker: &Linker<T>,
        extension_name: &str,
        exports: &[ExportedInterface],
        store: &mut Store<T>,
    ) -> Result<()> {
        // // Skip if already resolved
        // if self
        //     .resolved_exports
        //     .get(extension_name)
        //     .copied()
        //     .unwrap_or(false)
        // {
        //     return Ok(());
        // }

        // for export in exports {
        //     // Skip non-library interfaces
        //     if !export.name.ends_with("/lib") {
        //         continue;
        //     }

        //     // For each function in the interface
        //     for function_name in &export.functions {
        //         // Create a key for this function reference
        //         let key = FunctionRegistry::create_key(extension_name, &export.name, function_name);

        //         // Try to get the instance from the linker
        //         match linker.instance(&export.name) {
        //             Ok(instance) => {
        //                 // In Wasmtime's component model, we need to use the instance to get functions
        //                 // This is a simplified approach - in a real implementation, we would need to
        //                 // use the correct method to get the function based on the component model

        //                 // For now, we'll create a proxy function that will be resolved later
        //                 // when we have access to the actual function
        //                 let func = Func::new(
        //                     store,
        //                     // Function type (params and results would need to be determined)
        //                     wasmtime::component::FuncType::new(
        //                         None, // params
        //                         None, // results
        //                     ),
        //                     // Function implementation
        //                     move |_caller, _params, _results| {
        //                         // This is a placeholder - in a real implementation, we would
        //                         // call the actual function from the instance
        //                         Ok(())
        //                     },
        //                 );

        //                 // Resolve the function reference
        //                 self.registry.resolve(&key, func);

        //                 println!(
        //                     "Resolved export: {} from extension {}",
        //                     function_name, extension_name
        //                 );
        //             }
        //             Err(_) => {
        //                 // Log warning but continue
        //                 eprintln!(
        //                     "Warning: Could not find instance for interface '{}'",
        //                     export.name
        //                 );
        //             }
        //         }
        //     }
        // }

        // // Mark as resolved
        // self.resolved_exports
        //     .insert(extension_name.to_string(), true);

        Ok(())
    }

    /// Print information about function references
    pub fn print_function_refs(&self) {
        println!("\nFunction References:");
        println!(
            "  Resolved: {}/{}",
            self.registry.resolved_count(),
            self.registry.len()
        );

        // Print resolved exports
        println!("\nResolved Exports:");
        for (name, resolved) in &self.resolved_exports {
            println!("  {}: {}", name, if *resolved { "Yes" } else { "No" });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{ExportedInterface, ImportedInterface};

    #[test]
    fn test_new() {
        let linker = DynamicLinker::new();
        assert!(linker.registry().is_empty());
    }

    // Note: More comprehensive tests would require mocking Wasmtime components,
    // which is beyond the scope of these unit tests. Integration tests would
    // be more appropriate for testing the actual linking functionality.
}
