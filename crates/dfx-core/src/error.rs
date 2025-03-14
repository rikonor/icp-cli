//! Error types for the dfx-core crate.

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as StdError;

    #[test]
    fn test_interface_detection_error() {
        let error = Error::InterfaceDetection("failed to parse interface".to_string());
        assert_eq!(
            error.to_string(),
            "interface detection error: failed to parse interface"
        );
    }

    #[test]
    fn test_manifest_error() {
        let error = Error::Manifest("invalid manifest format".to_string());
        assert_eq!(error.to_string(), "manifest error: invalid manifest format");
    }

    #[test]
    fn test_dependency_error() {
        let error = Error::Dependency("circular dependency detected".to_string());
        assert_eq!(
            error.to_string(),
            "dependency error: circular dependency detected"
        );
    }

    #[test]
    fn test_unexpected_error() {
        let source = anyhow::Error::msg("unexpected failure");
        let error = Error::Unexpected(source);
        // The error string contains the original message due to #[error(transparent)]
        assert_eq!(error.to_string(), "unexpected failure");
    }

    #[test]
    fn test_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Error>();
    }
}

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
