use anyhow::Error;
use async_trait::async_trait;
use wasmtime::{component::Component, Engine};

#[derive(Debug, PartialEq)]
pub struct Interface {
    /// Name of the interface (e.g., "math/lib")
    pub name: String,

    /// Functions provided by this interface
    pub funcs: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub struct ComponentInterfaces {
    pub imports: Vec<Interface>,
    pub exports: Vec<Interface>,
}

#[async_trait]
pub trait DetectIfaces: Sync + Send {
    async fn detect(
        &self,
        engine: &Engine,
        component: &Component,
    ) -> Result<ComponentInterfaces, Error>;
}

pub struct IfaceDetector;

#[async_trait]
impl DetectIfaces for IfaceDetector {
    async fn detect(
        &self,
        engine: &Engine,
        component: &Component,
    ) -> Result<ComponentInterfaces, Error> {
        // For testing, just return empty interfaces
        Ok(ComponentInterfaces {
            imports: vec![],
            exports: vec![],
        })
    }
}
