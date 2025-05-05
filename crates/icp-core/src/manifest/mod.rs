//! Manifest handling for icp extensions.

mod model;

pub use model::{
    Extension, Interface, Load, LoadError, Manifest, ManifestHandle, Store, StoreError,
};
