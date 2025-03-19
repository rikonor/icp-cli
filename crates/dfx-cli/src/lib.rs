//! Library interface for dfx-cli functionality

mod extension;
mod spec;

// Re-export core functionality
pub use dfx_core::{
    dependency::DependencyGraph,
    interface::DetectIfaces,
    manifest::{Load, LoadError, Manifest, ManifestHandle, Store},
};

// Re-export cli functionality
pub use extension::{
    AddExtension, ExtensionAdder, ExtensionLister, ExtensionRemover, ListExtensions,
    RemoveExtension,
};

pub use spec::CommandSpec;
