//! Manifest handling for icp extensions.

mod model;

pub use model::{
    ExportedInterface, Extension, ImportedInterface, Load, LoadError, Manifest, ManifestHandle,
    Store, StoreError,
};
