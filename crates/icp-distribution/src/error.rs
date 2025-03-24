use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DistributionError {
    #[error("Binaries dir not found: {0}")]
    BinaryNotFound(PathBuf),

    #[error("Extensions dir not found: {0}")]
    ExtensionNotFound(PathBuf),

    #[error("Invalid binary format: {0}")]
    InvalidFormat(String),

    #[error("Checksum verification failed for: {0}")]
    ChecksumMismatch(String),

    #[error("Template error: {0}")]
    TemplateError(#[from] handlebars::RenderError),

    #[error("Template parsing error: {0}")]
    TemplateParseError(#[from] handlebars::TemplateError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Missing required file: {0}")]
    MissingFile(PathBuf),

    #[error("URL construction error: {0}")]
    UrlError(String),

    #[error("Invalid distribution value: {0}")]
    InvalidDistribution(String),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, DistributionError>;
