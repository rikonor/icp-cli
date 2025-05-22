use std::path::Path;

use crate::{CanisterManifest, bindings::icp::cli::command::CommandOutput};

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
    fn build(&self, canister_dir: &str) -> Result<String, BuildError>;
}

pub struct Builder {
    read_file: Box<dyn Fn(&str) -> Result<Vec<u8>, String>>,
    execute: Box<dyn Fn(&str, &[String]) -> Result<CommandOutput, String>>,
}

impl Builder {
    pub fn new(
        read_file: Box<dyn Fn(&str) -> Result<Vec<u8>, String>>,
        execute: Box<dyn Fn(&str, &[String]) -> Result<CommandOutput, String>>,
    ) -> Self {
        Builder { read_file, execute }
    }
}

impl Build for Builder {
    fn build(&self, canister_dir: &str) -> Result<String, BuildError> {
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

        let mut input_path = cm.canister.motoko.input;

        // Check if the input path is relative
        if input_path.is_relative() {
            input_path = Path::new(canister_dir).join(input_path);
        }

        // Convert the path to a string
        let input_path = input_path.to_string_lossy().into_owned();

        // Specify output path
        let output_path = Path::new(canister_dir)
            .join("main.wasm")
            .to_string_lossy()
            .into_owned();

        #[rustfmt::skip]
        let args = [
            &input_path,

            // output path
            "-o", &output_path,
        ]
        .map(ToString::to_string);

        // Invoke the `moc` command
        let out = (self.execute)("moc", &args)
            .map_err(|err| BuildError::BuildFailed(format!("failed to build canister: {}", err)))?;

        // Check the exit code
        if out.exit_code != 0 {
            return Err(BuildError::BuildFailed(format!(
                "moc failed with exit code {}",
                out.exit_code
            )));
        }

        Ok(output_path)
    }
}
