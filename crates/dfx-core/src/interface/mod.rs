//! Interface detection and management for WebAssembly components.

mod detector;

pub use detector::{ComponentInterfaces, DetectIfaces, IfaceDetector, Interface};

/// Suffix used for library interfaces
pub const LIBRARY_SUFFIX: &str = "/lib";
