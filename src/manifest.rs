use std::{
    fs::{read, write},
    io::ErrorKind,
    path::PathBuf,
};

use anyhow::{anyhow, Context as _};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{from_slice, to_vec_pretty};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Extension {
    pub name: String,
    pub wasm: PathBuf,
    pub pre: PathBuf,
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

#[async_trait]
pub trait Load: Sync + Send {
    async fn load(&self) -> Result<Manifest, LoadError>;
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

#[async_trait]
pub trait Store: Sync + Send {
    async fn store(&self, m: &Manifest) -> Result<(), StoreError>;
}

#[derive(Clone)]
pub struct ManifestHandle(pub PathBuf);

#[async_trait]
impl Load for ManifestHandle {
    async fn load(&self) -> Result<Manifest, LoadError> {
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

#[async_trait]
impl Store for ManifestHandle {
    async fn store(&self, m: &Manifest) -> Result<(), StoreError> {
        let bs = to_vec_pretty(m).context("failed to serialize manifest")?;

        write(
            &self.0, // path
            bs,      // content
        )
        .context("failed to write manifest")?;

        Ok(())
    }
}
