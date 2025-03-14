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
    fn test_interface_specific_errors() {
        // Test ComponentModelDisabled error
        let error = Error::Interface(InterfaceError::ComponentModelDisabled);
        assert_eq!(
            error.to_string(),
            "component model not enabled in the engine"
        );

        // Test InvalidFormat error
        let error = Error::Interface(InterfaceError::InvalidFormat("missing type".to_string()));
        assert_eq!(error.to_string(), "invalid component format: missing type");

        // Test ParseFailure error
        let error = Error::Interface(InterfaceError::ParseFailure("syntax error".to_string()));
        assert_eq!(error.to_string(), "failed to parse interface: syntax error");

        // Test MissingElement error
        let error = Error::Interface(InterfaceError::MissingElement("memory".to_string()));
        assert_eq!(
            error.to_string(),
            "missing required component element: memory"
        );

        // Test DuplicateInterface error
        let error = Error::Interface(InterfaceError::DuplicateInterface("math/lib".to_string()));
        assert_eq!(
            error.to_string(),
            "duplicate interface name detected: math/lib"
        );

        // Test NestedInstance error
        let error = Error::Interface(InterfaceError::NestedInstance("utils/lib".to_string()));
        assert_eq!(
            error.to_string(),
            "nested instance not fully supported: utils/lib"
        );
    }

    #[test]
    fn test_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Error>();
        assert_send_sync::<InterfaceError>();
    }
}

use thiserror::Error;

/// Interface detection specific errors
#[derive(Debug, Error)]
pub enum InterfaceError {
    /// Component model is not enabled in the engine
    #[error("component model not enabled in the engine")]
    ComponentModelDisabled,

    /// Invalid component format
    #[error("invalid component format: {0}")]
    InvalidFormat(String),

    /// Failed to parse interface
    #[error("failed to parse interface: {0}")]
    ParseFailure(String),

    /// Missing required component element
    #[error("missing required component element: {0}")]
    MissingElement(String),

    /// Duplicate interface name detected
    #[error("duplicate interface name detected: {0}")]
    DuplicateInterface(String),

    /// Nested instance not supported
    #[error("nested instance not fully supported: {0}")]
    NestedInstance(String),
}

/// Core error type for dfx-core operations
#[derive(Debug, Error)]
pub enum Error {
    /// An error occurred during interface detection
    #[error("interface detection error: {0}")]
    InterfaceDetection(String),

    /// A specific interface detection error
    #[error("{0}")]
    Interface(#[from] InterfaceError),

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
