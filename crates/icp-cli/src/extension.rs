use std::{
    fs::{create_dir_all, exists, read, remove_file, write},
    path::PathBuf,
    sync::Arc,
};

use anyhow::{Context as _, Error};
use async_trait::async_trait;
use http::Uri;
use reqwest::get;
use wasmtime::{component::Component, Engine};

use icp_core::{
    dependency::{DependencyError, DependencyGraph},
    interface::{ComponentInterfaces, DetectIfaces, LIBRARY_SUFFIX},
    manifest::{
        self, ExportedInterface, Extension, ImportedInterface, Load, ManifestHandle, Store,
    },
};

enum AdditionType {
    Uri(Uri),
    File(PathBuf),
}

impl TryFrom<&str> for AdditionType {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // File
        if exists(value)? {
            return Ok(Self::File(value.into()));
        }

        // Uri
        let u = value.parse::<Uri>()?;
        Ok(Self::Uri(u))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AddExtensionError {
    #[error("extension with name {0} already installed")]
    AlreadyExists(String),

    #[error("invalid uri: {0}")]
    _InvalidUri(String),

    #[error(transparent)]
    DependencyValidationFailed(#[from] DependencyError),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

#[async_trait]
pub trait AddExtension: Sync + Send {
    async fn add(&self, name: &str, p: &str) -> Result<(), AddExtensionError>;
}

pub struct ExtensionAdder {
    ngn: Engine,

    // Manifest
    mh: ManifestHandle,

    // Dirs
    extensions_dir: PathBuf,
    precompiles_dir: PathBuf,

    // Library interface detector
    detector: Arc<dyn DetectIfaces>,
}

impl ExtensionAdder {
    pub fn new(
        ngn: Engine,
        mh: ManifestHandle,
        extensions_dir: PathBuf,
        precompiles_dir: PathBuf,
        detector: Arc<dyn DetectIfaces>,
    ) -> Self {
        Self {
            ngn,
            mh,
            extensions_dir,
            precompiles_dir,
            detector,
        }
    }
}

#[async_trait]
impl AddExtension for ExtensionAdder {
    async fn add(&self, name: &str, p: &str) -> Result<(), AddExtensionError> {
        let m = self.mh.load().context("failed to load manifest")?;

        if m.xs.iter().any(|x| x.name == name) {
            return Err(AddExtensionError::AlreadyExists(
                name.to_owned(), // name
            ));
        }

        let ext = match AdditionType::try_from(p)? {
            AdditionType::File(path) => {
                read(&path).context(format!("failed to read extension file: {:?}", path))?
            }

            AdditionType::Uri(uri) => get(uri.to_string())
                .await
                .context("failed to download file")?
                .bytes()
                .await
                .context("failed to read body")?
                .to_vec(),
        };

        // Precompile
        let pre = self
            .ngn
            .precompile_component(&ext)
            .context("failed to precompile component")?;

        // Compatibility hash
        // let h = self.ngn.precompile_compatibility_hash();

        // Store extension
        let ext_path = self.extensions_dir.join(format!("{name}.component.wasm"));
        create_dir_all(&self.extensions_dir).context("failed to create extensions directory")?;
        write(&ext_path, &ext).context("failed to write extension to disk")?;

        // Store precompile
        let pre_path = self.precompiles_dir.join(format!("{name}.precompile.bin"));
        create_dir_all(&self.precompiles_dir).context("failed to create precompiles directory")?;
        write(&pre_path, &pre).context("failed to write precompile to disk")?;

        let cmpnt = unsafe {
            Component::deserialize(&self.ngn, &pre)
                .context("failed to deserialize precompiled component")?
        };

        let ComponentInterfaces { imports, exports } = self
            .detector
            .detect(&self.ngn, &cmpnt)
            .await
            .context("failed to detect library interfaces")?;

        let [imports, exports] = [imports, exports].map(|ifaces| {
            ifaces
                .into_iter()
                .filter(|x| x.name.ends_with(LIBRARY_SUFFIX))
                .collect::<Vec<_>>()
        });

        // Create a new extension with detected library interfaces
        let imports = imports
            .into_iter()
            .map(|imp| ImportedInterface {
                name: imp.name,
                provider: "TODO".to_string(),
                functions: imp.funcs,
            })
            .collect();

        let exports = exports
            .into_iter()
            .map(|exp| ExportedInterface {
                name: exp.name,
                funcs: exp.funcs,
            })
            .collect();

        let x = Extension {
            name: name.to_string(),
            wasm: ext_path.clone(),
            pre: pre_path.clone(),
            imports,
            exports,
        };

        // Validate dependencies
        if let Err(err) = DependencyGraph::new(&m)
            .context("failed to create dependency graph")?
            .validate_extension_dependencies(&x, &m)
        {
            // Clean up temporary files since we're not adding the extension
            for p in [&ext_path, &pre_path] {
                if p.exists() {
                    remove_file(p).context("failed to remove temporary file")?;
                }
            }

            return Err(err.into());
        }

        // Update manifest
        let mut m = m;
        m.xs.push(x);

        self.mh
            .store(&m)
            .context("failed to store extensions manifest")?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RemoveExtensionError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

#[async_trait]
pub trait RemoveExtension: Sync + Send {
    async fn remove(&self, name: &str) -> Result<(), RemoveExtensionError>;
}

pub struct ExtensionRemover {
    mh: ManifestHandle,
}

impl ExtensionRemover {
    pub fn new(mh: ManifestHandle) -> Self {
        Self { mh }
    }
}

#[async_trait]
impl RemoveExtension for ExtensionRemover {
    async fn remove(&self, name: &str) -> Result<(), RemoveExtensionError> {
        let m = self
            .mh
            .load()
            .context("failed to load extensions manifest")?;

        let x =
            m.xs.iter()
                .find(|x| x.name == name)
                .ok_or(RemoveExtensionError::NotFound(name.to_owned()))?;

        // Clean files
        for p in [&x.pre, &x.wasm] {
            remove_file(p).context("failed to remove precompile")?;
        }

        // Update manifest
        let mut m = m;

        m.xs.retain(|x| x.name != name);

        self.mh
            .store(&m)
            .context("failed to store extensions manifest")?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ListExtensionsError {
    #[error(transparent)]
    LoadError(#[from] manifest::LoadError),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

#[async_trait]
pub trait ListExtensions: Sync + Send {
    async fn list(&self) -> Result<Vec<String>, ListExtensionsError>;
}

pub struct ExtensionLister {
    mh: ManifestHandle,
}

impl ExtensionLister {
    pub fn new(mh: ManifestHandle) -> Self {
        Self { mh }
    }
}

#[async_trait]
impl ListExtensions for ExtensionLister {
    async fn list(&self) -> Result<Vec<String>, ListExtensionsError> {
        let m = self
            .mh
            .load()
            .context("failed to load extensions manifest")?;

        Ok(m.xs
            .iter()
            .cloned()
            .map(|x| x.name)
            .collect::<Vec<String>>())
    }
}
