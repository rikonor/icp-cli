//! Error types for the dfx-core crate.

use thiserror::Error;

/// Core error type for dfx-core operations
#[derive(Debug, Error)]
pub enum Error {
    /// An error occurred during interface detection
    #[error("interface detection error: {0}")]
    InterfaceDetection(String),

    /// An error occurred during manifest operations
    #[error("manifest error: {0}")]
    Manifest(String),

    /// An error occurred during dependency resolution
    #[error("dependency error: {0}")]
    Dependency(String),

    /// An unexpected error occurred
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}
