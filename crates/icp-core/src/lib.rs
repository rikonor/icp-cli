//! Core functionality for icp component model tools.
//!
//! This crate provides the core functionality used by icp-cli and other tools
//! for working with WebAssembly components and extensions.

pub mod component;
pub mod dependency;
mod error;
pub mod interface;
pub mod manifest;

// Re-export core types and traits
pub use component::{DynamicLinker, DynamicLinkingError, FunctionRegistry, FunctionRegistryError};
pub use dependency::{DependencyError, DependencyGraph};
pub use error::Error;
pub use interface::{ComponentInterfaces, DetectIfaces};
pub use manifest::{Extension, Interface, Load, LoadError, Manifest, ManifestHandle, Store};

/// Version of the icp-core crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
