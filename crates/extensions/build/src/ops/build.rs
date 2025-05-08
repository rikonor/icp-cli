use std::path::Path;

use dashmap::DashMap;

use crate::{
    CanisterManifest, LazyRef,
    bindings::icp::{build::types, cli::misc::print},
};

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("failed to process canister manifest: {0}")]
    ManifestProcessing(String),

    #[error("failed to build canister: {0}")]
    BuildFailed(String),

    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl From<BuildError> for String {
    fn from(e: BuildError) -> Self {
        match e {
            BuildError::ManifestProcessing(err) => err,
            BuildError::BuildFailed(err) => err,
            BuildError::Unexpected(err) => {
                format!("An unexpected error occurred: {}", err)
            }
        }
    }
}

impl From<BuildError> for u8 {
    fn from(e: BuildError) -> Self {
        match e {
            BuildError::ManifestProcessing(_) => 3,
            BuildError::BuildFailed(_) => 4,
            BuildError::Unexpected(_) => 2,
        }
    }
}

pub trait Build {
    fn build(&self, canister_dir: &str) -> Result<(), BuildError>;
}

pub struct Builder {
    read_file: Box<dyn Fn(&str) -> Result<Vec<u8>, String>>,
    builders: LazyRef<DashMap<String, types::Builder>>,
}

impl Builder {
    pub fn new(
        read_file: Box<dyn Fn(&str) -> Result<Vec<u8>, String>>,
        builders: LazyRef<DashMap<String, types::Builder>>,
    ) -> Self {
        Builder {
            read_file,
            builders,
        }
    }
}

impl Build for Builder {
    fn build(&self, canister_dir: &str) -> Result<(), BuildError> {
        print("[build] building");

        let manifest_path = Path::new(canister_dir).join("canister.toml");
        let manifest_path = manifest_path.to_string_lossy();

        let bs = (self.read_file)(&manifest_path).map_err(|err| {
            BuildError::ManifestProcessing(format!("failed to read {}: {}", manifest_path, err))
        })?;

        let s = String::from_utf8(bs).map_err(|err| {
            BuildError::ManifestProcessing(format!("failed to read {}: {}", manifest_path, err))
        })?;

        let cm: CanisterManifest = toml::from_str(&s).map_err(|err| {
            BuildError::ManifestProcessing(format!(
                "failed to decode toml {}: {}",
                manifest_path, err
            ))
        })?;

        print(&format!(
            "[build] building {} with type {}",
            cm.canister.name, cm.canister.canister_type
        ));

        let b = match self.builders.get(&cm.canister.canister_type) {
            // Ok
            Some(b) => b,

            // No such builder
            None => {
                print("Available builders:");
                self.builders.iter().for_each(|v| {
                    print(&format!("  - {}", v.key()));
                });

                return Err(BuildError::BuildFailed(format!(
                    "Canister builder for '{}' not available",
                    cm.canister.canister_type
                )));
            }
        };

        b.value()
            .build_canister(canister_dir)
            .map_err(|err| BuildError::BuildFailed(err))
    }
}
