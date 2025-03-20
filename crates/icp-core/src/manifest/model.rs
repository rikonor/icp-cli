//! Data model for icp extension manifests.

use std::{
    fs::{create_dir_all, read, write},
    io::ErrorKind,
    path::PathBuf,
};

use anyhow::{anyhow, Context as _};
use serde::{Deserialize, Serialize};
use serde_json::{from_slice, to_vec_pretty};

/// Represents an interface exported by an extension
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExportedInterface {
    /// Name of the interface (e.g., "math/lib")
    pub name: String,

    /// Names of functions provided by this interface
    pub funcs: Vec<String>,
}

/// Represents an interface imported by an extension
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ImportedInterface {
    /// Name of the interface (e.g., "math/lib")
    pub name: String,

    /// Name of the extension providing this interface
    pub provider: String,

    /// Names of functions used from this interface
    pub functions: Vec<String>,
}

/// Represents an extension in the manifest
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Extension {
    /// Name of the extension
    pub name: String,

    /// Path to the WebAssembly component
    pub wasm: PathBuf,

    /// Path to the pre-compiled component
    pub pre: PathBuf,

    /// Interfaces imported by this extension
    #[serde(default)]
    pub imports: Vec<ImportedInterface>,

    /// Interfaces exported by this extension
    #[serde(default)]
    pub exports: Vec<ExportedInterface>,
}

/// Represents the manifest of installed extensions
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Manifest {
    /// List of installed extensions
    #[serde(rename = "extensions")]
    pub xs: Vec<Extension>,
}

/// Errors that can occur during manifest loading
#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    /// Manifest file not found
    #[error("not found: {0}")]
    NotFound(PathBuf),

    /// Unexpected error during loading
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

/// Trait for loading manifests
pub trait Load: Sync + Send {
    /// Load a manifest
    fn load(&self) -> Result<Manifest, LoadError>;
}

/// Errors that can occur during manifest storage
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// Unexpected error during storage
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

/// Trait for storing manifests
pub trait Store: Sync + Send {
    /// Store a manifest
    fn store(&self, m: &Manifest) -> Result<(), StoreError>;
}

/// Handle for loading and storing manifests
#[derive(Clone)]
pub struct ManifestHandle(pub PathBuf);

impl Load for ManifestHandle {
    fn load(&self) -> Result<Manifest, LoadError> {
        // Read
        let bs = read(&self.0).map_err(|err| match err.kind() {
            // NotFound
            ErrorKind::NotFound => LoadError::NotFound(self.0.to_owned()),

            // _
            err => LoadError::UnexpectedError(anyhow!("failed to load manifest: {err}")),
        })?;

        // Parse
        Ok(from_slice(&bs).context("failed to parse manifest")?)
    }
}

impl Store for ManifestHandle {
    fn store(&self, m: &Manifest) -> Result<(), StoreError> {
        let bs = to_vec_pretty(m).context("failed to serialize manifest")?;

        let md = self
            .0
            .parent()
            .context("failed to infer manifest directory")?;

        create_dir_all(md).context("failed to create manifest directory")?;

        write(
            &self.0, // path
            bs,      // content
        )
        .context("failed to write manifest")?;

        Ok(())
    }
}
