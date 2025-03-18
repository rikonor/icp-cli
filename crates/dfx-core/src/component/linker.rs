use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::{anyhow, Context};
use thiserror::Error;
use wasmtime::component::{Instance, Linker};
use wasmtime::Store;

use super::function_registry::{FunctionRegistry, FunctionRegistryError};
use crate::interface::LIBRARY_SUFFIX;
use crate::manifest::{ExportedInterface, ImportedInterface};

/// Errors that can occur during dynamic linking operations
#[derive(Debug, Error)]
pub enum DynamicLinkingError {
    /// Function reference not resolved
    #[error("function reference not resolved: {0}")]
    UnresolvedReference(String),

    /// Function reference error
    #[error(transparent)]
    FunctionRegistryError(#[from] FunctionRegistryError),

    /// Unexpected error
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

/// Dynamic linker for WebAssembly components
///
/// The dynamic linker manages function references between extensions,
/// handling both import linking and export resolution.
pub struct DynamicLinker {
    /// Registry for function references
    registry: FunctionRegistry,

    /// Map from extension name to resolved status
    resolved_exports: HashMap<String, bool>,
}

impl DynamicLinker {
    /// Create a new dynamic linker
    ///
    /// # Arguments
    ///
    /// * `registry` - Function registry to use for managing references
    pub fn new(registry: FunctionRegistry) -> Self {
        Self {
            registry,
            resolved_exports: HashMap::new(),
        }
    }

    /// Link imports for an extension
    ///
    /// # Arguments
    ///
    /// * `lnk` - Wasmtime linker to add imports to
    /// * `imps` - List of imported interfaces to link
    ///
    /// # Returns
    ///
    /// * `Ok(())` if linking succeeded
    /// * `Err(DynamicLinkingError)` if linking failed
    pub fn link_imports<T: Send>(
        &mut self,
        lnk: &mut Linker<T>,
        imps: Vec<ImportedInterface>,
    ) -> Result<(), DynamicLinkingError> {
        for imp in imps {
            // Skip non-library interfaces
            if !imp.name.ends_with(LIBRARY_SUFFIX) {
                continue;
            }

            for f in imp.functions {
                let k = FunctionRegistry::create_key(
                    &imp.name, // interface
                    &f,        // function
                );

                // Create a function reference
                let fref = Arc::new(Mutex::new(None));

                // Register the function reference
                self.registry.register(k.clone(), Arc::clone(&fref))?;

                let fname = f.clone();

                lnk.instance(&imp.name)?.func_new_async(
                    &f,
                    move |mut store, params, results| {
                        let fname = fname.clone();
                        let fref = Arc::clone(&fref);

                        Box::new(async move {
                            let f = {
                                let g = fref.lock().unwrap();
                                *g.as_ref().ok_or_else(|| {
                                    DynamicLinkingError::UnresolvedReference(fname)
                                })?
                            };

                            f.call_async(&mut store, params, results)
                                .await
                                .context("call failed")?;

                            f.post_return_async(&mut store)
                                .await
                                .context("post-return failed")?;

                            Ok(())
                        })
                    },
                )?;
            }
        }

        Ok(())
    }

    /// Resolve exports for an extension
    ///
    /// # Arguments
    ///
    /// * `store` - Wasmtime store
    /// * `extension` - Name of the extension
    /// * `inst` - Component instance
    /// * `exports` - List of exported interfaces to resolve
    ///
    /// # Returns
    ///
    /// * `Ok(())` if resolution succeeded
    /// * `Err(DynamicLinkingError)` if resolution failed
    pub fn resolve_exports<T>(
        &mut self,
        mut store: &mut Store<T>,
        extension: &str,
        inst: &Instance,
        exports: &[ExportedInterface],
    ) -> Result<(), DynamicLinkingError> {
        // Skip if already resolved
        if self
            .resolved_exports
            .get(extension)
            .copied()
            .unwrap_or(false)
        {
            return Ok(());
        }

        for exp in exports {
            let e = inst
                .get_export(
                    &mut store, // store
                    None,       // instance
                    &exp.name,  // name
                )
                .ok_or(anyhow!("missing export"))?;

            for fname in &exp.funcs {
                let k = FunctionRegistry::create_key(
                    &exp.name, // interface
                    fname,     // function
                );

                let e = inst
                    .get_export(
                        &mut store, // store
                        Some(&e),   // instance
                        fname,      // name
                    )
                    .ok_or(anyhow!("missing export"))?;

                let f = inst
                    .get_func(
                        &mut store, // store
                        e,          // name
                    )
                    .ok_or(anyhow!("missing function"))?;

                self.registry.resolve(&k, f)?;
            }
        }

        // Mark as resolved
        self.resolved_exports.insert(extension.to_string(), true);

        Ok(())
    }

    /// Get the number of resolved exports
    pub fn resolved_export_count(&self) -> usize {
        self.resolved_exports.values().filter(|&&r| r).count()
    }

    /// Get the total number of exports
    pub fn export_count(&self) -> usize {
        self.resolved_exports.len()
    }

    /// Check if an extension's exports are resolved
    pub fn is_extension_resolved(&self, extension: &str) -> bool {
        self.resolved_exports
            .get(extension)
            .copied()
            .unwrap_or(false)
    }

    /// Get a reference to the function registry
    pub fn registry(&self) -> &FunctionRegistry {
        &self.registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_linker() {
        let registry = FunctionRegistry::new();
        let linker = DynamicLinker::new(registry);
        assert_eq!(linker.resolved_export_count(), 0);
        assert_eq!(linker.export_count(), 0);
    }

    #[test]
    fn test_extension_resolution_tracking() {
        let registry = FunctionRegistry::new();
        let mut linker = DynamicLinker::new(registry);

        assert!(!linker.is_extension_resolved("test"));
        linker.resolved_exports.insert("test".to_string(), true);
        assert!(linker.is_extension_resolved("test"));
    }
}
