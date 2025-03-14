//! Core functionality for dfx component model tools.
//!
//! This crate provides the core functionality used by dfx-cli and other tools
//! for working with WebAssembly components and extensions.

pub mod dependency;
mod error;
pub mod interface;
pub mod manifest;

// Re-export core types and traits
pub use dependency::{DependencyError, DependencyGraph};
pub use error::Error;
pub use interface::{ComponentInterfaces, DetectIfaces, Interface};
pub use manifest::{
    ExportedInterface, Extension, ImportedInterface, Load, LoadError, Manifest, ManifestHandle,
    Store,
};

/// Version of the dfx-core crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
