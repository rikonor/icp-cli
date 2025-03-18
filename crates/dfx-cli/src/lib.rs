//! Library interface for dfx-cli functionality

mod extension;
mod iface;
mod spec;

// Re-export core functionality
pub use dfx_core::{
    component::{DynamicLinker, FunctionRegistry},
    dependency::DependencyGraph,
    interface::DetectIfaces,
    manifest::{Load, LoadError, Manifest, ManifestHandle, Store},
};

// Re-export cli functionality
pub use extension::{
    AddExtension, ExtensionAdder, ExtensionLister, ExtensionRemover, ListExtensions,
    RemoveExtension,
};
pub use iface::IfaceDetector;
pub use spec::CommandSpec;
