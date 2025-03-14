use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use anyhow::anyhow;
use wasmtime::component::{Instance, Linker};
use wasmtime::Store;

use crate::function_registry::FunctionRegistry;
use crate::iface::LIBRARY_SUFFIX;
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
    pub fn new(registry: FunctionRegistry) -> Self {
        Self {
            registry,
            resolved_exports: HashMap::new(),
        }
    }

    /// Link imports for an extension
    pub fn link_imports<T>(
        &mut self,
        lnk: &mut Linker<T>,
        x: &str,
        imps: Vec<ImportedInterface>,
    ) -> Result<(), DynamicLinkingError> {
        for imp in imps {
            // Skip non-library interfaces
            if !imp.name.ends_with(LIBRARY_SUFFIX) {
                continue;
            }

            for f in imp.functions {
                let k = FunctionRegistry::create_key(
                    x,         // extension
                    &imp.name, // interface
                    &f,        // function
                );

                // Create a function reference
                let fref = Arc::new(Mutex::new(None));

                // Register the function reference
                self.registry.register(
                    k,                 // key
                    Arc::clone(&fref), // reference
                );

                let fname = f.clone();

                lnk.instance(&imp.name)?
                    .func_new(&f, move |mut store, params, results| {
                        let fref = fref.lock().unwrap();

                        let f = fref.as_ref().ok_or_else(|| {
                            DynamicLinkingError::UnresolvedReference(fname.clone())
                        })?;

                        f.call(&mut store, params, results)?;
                        f.post_return(&mut store)?;

                        Ok(())
                    })?;
            }
        }

        Ok(())
    }

    /// Resolve exports for an extension
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
                    extension, // extension
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

                println!("registering {k}");
                self.registry.resolve(&k, f);
            }
        }

        // Mark as resolved
        self.resolved_exports.insert(extension.to_string(), true);

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
