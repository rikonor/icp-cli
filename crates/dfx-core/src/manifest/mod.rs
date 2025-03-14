//! Manifest handling for dfx extensions.

mod model;

pub use model::{
    ExportedInterface, Extension, ImportedInterface, Load, LoadError, Manifest, ManifestHandle,
    Store, StoreError,
};
