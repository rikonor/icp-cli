use std::{
    fs::{create_dir_all, read, write},
    io::ErrorKind,
    path::PathBuf,
};

use anyhow::{anyhow, Context as _};
use serde::{Deserialize, Serialize};
use serde_json::{from_slice, to_vec_pretty};

use crate::library::LibraryInterface;

/// Represents an interface exported by an extension
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExportedInterface {
    /// Name of the interface (e.g., "math/lib")
    pub name: String,
    /// Names of functions provided by this interface
    pub functions: Vec<String>,
}

impl ExportedInterface {
    /// Creates a new exported interface
    pub fn new(name: String, functions: Vec<String>) -> Self {
        Self { name, functions }
    }

    /// Converts a LibraryInterface to an ExportedInterface
    pub fn from_library_interface(interface: &LibraryInterface) -> Self {
        let functions = interface
            .functions
            .values()
            .map(|f| f.name.clone())
            .collect();

        Self {
            name: interface.name.clone(),
            functions,
        }
    }
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

impl ImportedInterface {
    /// Creates a new imported interface
    pub fn new(name: String, provider: String, functions: Vec<String>) -> Self {
        Self {
            name,
            provider,
            functions,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Extension {
    pub name: String,
    pub wasm: PathBuf,
    pub pre: PathBuf,
    #[serde(default)]
    pub exported_interfaces: Vec<ExportedInterface>,
    #[serde(default)]
    pub imported_interfaces: Vec<ImportedInterface>,
}

impl Extension {
    /// Creates a new extension
    pub fn new(name: String, wasm: PathBuf, pre: PathBuf) -> Self {
        Self {
            name,
            wasm,
            pre,
            exported_interfaces: Vec::new(),
            imported_interfaces: Vec::new(),
        }
    }

    /// Adds an exported interface to the extension
    pub fn add_exported_interface(&mut self, interface: ExportedInterface) {
        self.exported_interfaces.push(interface);
    }

    /// Adds an imported interface to the extension
    pub fn add_imported_interface(&mut self, interface: ImportedInterface) {
        self.imported_interfaces.push(interface);
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Manifest {
    #[serde(rename = "extensions")]
    pub xs: Vec<Extension>,
}

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("not found: {0}")]
    NotFound(PathBuf),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

pub trait Load: Sync + Send {
    fn load(&self) -> Result<Manifest, LoadError>;
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

pub trait Store: Sync + Send {
    fn store(&self, m: &Manifest) -> Result<(), StoreError>;
}

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
