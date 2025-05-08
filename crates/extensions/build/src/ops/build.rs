use std::path::Path;

use crate::{CanisterManifest, bindings::icp::cli::misc::print};

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("Failed to process canister manifest: {0}")]
    ManifestProcessing(String),

    #[error("One or more canisters failed to build.")]
    BuildFailed,

    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl From<BuildError> for String {
    fn from(e: BuildError) -> Self {
        match e {
            BuildError::ManifestProcessing(e) => e,
            BuildError::BuildFailed => e.to_string(),
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
            BuildError::BuildFailed => 4,
            BuildError::Unexpected(_) => 2,
        }
    }
}

pub trait Build {
    fn build(&self, canister_dir: &str) -> Result<(), BuildError>;
}

pub struct Builder {
    read_file: Box<dyn Fn(&str) -> Result<Vec<u8>, String>>,
}

impl Builder {
    pub fn new(read_file: Box<dyn Fn(&str) -> Result<Vec<u8>, String>>) -> Self {
        Builder { read_file }
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

        Ok(())
    }
}
