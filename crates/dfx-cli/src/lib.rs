//! Library interface for dfx-cli functionality

mod dependency;
mod dynamic_linker;
mod extension;
mod function_registry;
mod iface;
mod manifest;
mod spec;

// Re-export all items needed by main
pub use dependency::DependencyGraph;
pub use dynamic_linker::DynamicLinker;
pub use extension::{
    AddExtension, ExtensionAdder, ExtensionLister, ExtensionRemover, ListExtensions,
    RemoveExtension,
};
pub use function_registry::FunctionRegistry;
pub use iface::IfaceDetector;
pub use manifest::{Load, LoadError, Manifest, ManifestHandle, Store};
pub use spec::CommandSpec;
