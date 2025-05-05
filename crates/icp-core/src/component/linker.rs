use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::{anyhow, Context};
use wasmtime::component::{Instance, Linker};
use wasmtime::Store;

use crate::{
    interface::{parse_interface_name, LIBRARY_SUFFIX},
    FunctionRegistry, FunctionRegistryError, Interface,
};

/// Errors that can occur during dynamic linking operations
#[derive(Debug, thiserror::Error)]
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

    /// Link imports and exports for an extension
    ///
    /// # Arguments
    ///
    /// * `lnk` - Wasmtime linker to add imports to
    /// * `imps` - List of imported interfaces to link
    /// * `exps` - List of exported interfaces to link
    ///
    /// # Returns
    ///
    /// * `Ok(())` if linking succeeded
    /// * `Err(DynamicLinkingError)` if linking failed
    pub fn link<T: Send>(
        &mut self,
        lnk: &mut Linker<T>,
        ifaces: Vec<Interface>,
    ) -> Result<(), DynamicLinkingError> {
        // Link imports
        for iface in ifaces {
            // Skip non-library interfaces (check base name)
            let (name, _) = parse_interface_name(&iface.name);
            if !name.ends_with(LIBRARY_SUFFIX) {
                continue;
            }

            for f in iface.funcs {
                let k = FunctionRegistry::create_key(
                    &iface.name, // interface
                    &f,          // function
                );

                if self.registry.contains(k.as_str()) {
                    continue;
                }

                // Create a function reference
                let fref = Arc::new(Mutex::new(None));

                // Register the function reference
                self.registry.register(k.clone(), Arc::clone(&fref))?;

                let fname = f.clone();

                lnk.instance(&iface.name)?.func_new_async(
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
    pub fn resolve<T>(
        &mut self,
        mut store: &mut Store<T>,
        extension: &str,
        inst: &Instance,
        exports: &[Interface],
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
    use anyhow::Error;
    use wasmtime::{Config, Engine};

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

    #[tokio::test]
    async fn test_link_duplicate_function_definition_fails() -> Result<(), Error> {
        // WASM Configuration
        let mut cfg = Config::new();
        let cfg = cfg.async_support(true);

        // Engine
        let ngn = Engine::new(cfg)?;

        // Linker
        let mut lnk: Linker<()> = Linker::new(&ngn);

        // Create function registry
        let reg = FunctionRegistry::new();

        // Create dynamic linker
        let mut dynlnk = DynamicLinker::new(reg);

        let imp = Interface {
            name: "my-namespace:my-package-1/lib@0.0.1".to_string(),
            funcs: vec![
                "fn-1".to_string(), //
                "fn-2".to_string(),
            ],
        };

        let exp = Interface {
            name: "my-namespace:my-package-2/lib@0.0.1".to_string(),
            funcs: vec![
                "fn-1".to_string(), //
                "fn-2".to_string(),
            ],
        };

        dynlnk.link(
            &mut lnk,
            vec![imp, exp], // interfaces
        )?;

        Ok(())
    }
}
