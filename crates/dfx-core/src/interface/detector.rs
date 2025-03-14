//! Interface detection for WebAssembly components.

use anyhow::Error;
use async_trait::async_trait;
use wasmtime::{component::Component, Engine};

/// Represents a WebAssembly component interface
#[derive(Debug, PartialEq)]
pub struct Interface {
    /// Name of the interface (e.g., "math/lib")
    pub name: String,

    /// Functions provided by this interface
    pub funcs: Vec<String>,
}

/// Represents all interfaces of a WebAssembly component
#[derive(Debug, PartialEq)]
pub struct ComponentInterfaces {
    /// Interfaces imported by the component
    pub imports: Vec<Interface>,

    /// Interfaces exported by the component
    pub exports: Vec<Interface>,
}

/// Trait for detecting interfaces in WebAssembly components
#[async_trait]
pub trait DetectIfaces: Sync + Send {
    /// Detect interfaces in a WebAssembly component
    async fn detect(
        &self,
        engine: &Engine,
        component: &Component,
    ) -> Result<ComponentInterfaces, Error>;
}

/// Default implementation of interface detection
pub struct IfaceDetector;

#[async_trait]
impl DetectIfaces for IfaceDetector {
    async fn detect(
        &self,
        _engine: &Engine,
        _component: &Component,
    ) -> Result<ComponentInterfaces, Error> {
        // For initial implementation, just return empty interfaces
        // This will be expanded in Stage 2 with actual detection logic
        Ok(ComponentInterfaces {
            imports: vec![],
            exports: vec![],
        })
    }
}
