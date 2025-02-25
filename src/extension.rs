use std::{
    fs::{read, remove_file, write},
    path::PathBuf,
};

use anyhow::Context as _;
use async_trait::async_trait;
use http::Uri;
use wasmtime::Engine;

use crate::manifest::{Extension, Load, ManifestHandle, Store};

enum AdditionType {
    _Uri(Uri),
    File(PathBuf),
}

impl From<&str> for AdditionType {
    fn from(value: &str) -> Self {
        // match value.parse::<Uri>() {
        //     Ok(uri) => Self::Uri(uri),

        //     // Assume local path
        //     Err(_) => Self::File(value.into()),
        // }

        Self::File(value.into())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AddExtensionError {
    #[error("extension with name {0} already installed")]
    AlreadyExists(String),

    #[error("invalid uri: {0}")]
    _InvalidUri(String),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

#[async_trait]
pub trait AddExtension: Sync + Send {
    async fn add(&self, name: &str, p: &str) -> Result<(), AddExtensionError>;
}

pub struct ExtensionAdder {
    ngn: Engine,
    mh: ManifestHandle,
    // extensions_dir: PathBuf,
    // precompiles_dir: PathBuf,
}

impl ExtensionAdder {
    pub fn new(ngn: Engine, mh: ManifestHandle) -> Self {
        Self { ngn, mh }
    }
}

#[async_trait]
impl AddExtension for ExtensionAdder {
    async fn add(&self, name: &str, p: &str) -> Result<(), AddExtensionError> {
        let m = self.mh.load().await.context("failed to load manifest")?;

        if m.xs.iter().any(|x| x.name == name) {
            return Err(AddExtensionError::AlreadyExists(
                name.to_owned(), // name
            ));
        }

        let ext = match AdditionType::from(p) {
            AdditionType::File(path) => {
                read(&path).context(format!("failed to read extension file: {:?}", path))?
            }

            AdditionType::_Uri(_uri) => {
                unimplemented!()
            }
        };

        // TODO? What if its a uri but it's malformed? we should not assume its a file
        // Err(AddExtensionError::InvalidUri(uri.to_string()))

        // Precompile
        let pre = self
            .ngn
            .precompile_component(&ext)
            .context("failed to precompile component")?;

        // Compatibility hash
        // let h = self.ngn.precompile_compatibility_hash();

        // Store extension
        // TODO(or.ricon): Clean this up...
        write("cmpnt.wasm", &ext).context("failed to write extension to disk")?;

        // Store precompile
        // TODO(or.ricon): Clean this up...
        write("pre.bin", &pre).context("failed to write precompile to disk")?;

        // Update manifest
        let mut m = m;

        m.xs.push(Extension {
            name: name.to_string(),
            wasm: "cmpnt.wasm".into(),
            pre: "pre.bin".into(),
        });

        self.mh
            .store(&m)
            .await
            .context("failed to store manifest")?;

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
        let m = self.mh.load().await.context("failed to load manifest")?;

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
            .await
            .context("failed to store manifest")?;

        Ok(())
    }
}
