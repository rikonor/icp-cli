use anyhow::Context as _;
use async_trait::async_trait;
use fluent_uri::{component::Scheme, Uri};
use serde::{Deserialize, Serialize};
use wasmtime::Engine;

const SCHEME_HTTP: &Scheme = Scheme::new_or_panic("http");
const SCHEME_HTTPS: &Scheme = Scheme::new_or_panic("https");
const SCHEME_GIT: &Scheme = Scheme::new_or_panic("git");
const SCHEME_FILE: &Scheme = Scheme::new_or_panic("file");

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Extension {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Manifest {
    #[serde(rename = "extensions")]
    pub xs: Vec<Extension>,
}

#[derive(Debug, thiserror::Error)]
pub enum AddExtensionError {
    #[error("invalid uri: {0}")]
    InvalidUri(String),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

#[async_trait]
pub trait AddExtension: Sync + Send {
    async fn add(&self, name: &str, path: &str) -> Result<(), AddExtensionError>;
}

pub struct Adders {
    pub http: Http,
    pub local: Local,
}

pub struct ExtensionAdder {
    ngn: Engine,
    adders: Adders,
}

impl ExtensionAdder {
    pub fn new(ngn: Engine, adders: Adders) -> Self {
        Self { ngn, adders }
    }
}

#[async_trait]
impl AddExtension for ExtensionAdder {
    async fn add(&self, name: &str, uri: &str) -> Result<(), AddExtensionError> {
        let uri = Uri::parse(uri).context("failed to parse uri")?;

        // http
        if uri.scheme() == SCHEME_HTTP || uri.scheme() == SCHEME_HTTPS {
            return self.adders.http.add(uri.as_str()).await;
        }

        // file
        if uri.scheme() == SCHEME_FILE {
            return self.adders.local.add(uri.as_str()).await;
        }

        Err(AddExtensionError::InvalidUri(uri.to_string()))
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

pub struct ExtensionRemover;

#[async_trait]
impl RemoveExtension for ExtensionRemover {
    async fn remove(&self, name: &str) -> Result<(), RemoveExtensionError> {
        println!("Removing {name}");

        Ok(())
    }
}

pub struct Http;

impl Http {
    async fn add(&self, uri: &str) -> Result<(), AddExtensionError> {
        println!("adding http: {uri}");
        Ok(())
    }
}

pub struct Local;

impl Local {
    pub async fn add(&self, path: &str) -> Result<(), AddExtensionError> {
        println!("adding local: {path}");
        Ok(())
    }
}
