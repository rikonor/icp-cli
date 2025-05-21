use std::path::Path;

use dashmap::DashMap;
use icp_component_invoke::Val as IcpVal;

use crate::{
    CanisterManifest, LazyRef,
    bindings::icp::cli::{component::invoke, misc::print},
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
    fn build(&self, canister_dir: &str) -> Result<String, BuildError>;
}

pub struct Builder {
    read_file: Box<dyn Fn(&str) -> Result<Vec<u8>, String>>,
    builders: LazyRef<DashMap<String, (String, String)>>,
}

impl Builder {
    pub fn new(
        read_file: Box<dyn Fn(&str) -> Result<Vec<u8>, String>>,
        builders: LazyRef<DashMap<String, (String, String)>>,
    ) -> Self {
        Builder {
            read_file,
            builders,
        }
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

        let (interface_name, function_name) = match self.builders.get(&cm.canister.canister_type) {
            // Ok
            Some(b) => b.value().clone(),

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

        let params = vec![
            IcpVal::String(canister_dir.to_string()), // canister_dir
        ];

        // TODO(or.ricon): The following is probably going to have to be repeated in other facades,
        // We should look into creating a common function for this under icp-component-invoke

        // Serialize parameters
        let params = serde_json::to_vec(&params).map_err(|err| {
            BuildError::BuildFailed(format!("failed to serialize parameters: {:?}", err))
        })?;

        // Invoke the builder
        let out = invoke(&interface_name, &function_name, &params).map_err(|err| {
            BuildError::BuildFailed(format!("failed to invoke builder: {:?}", err))
        })?;

        // Deserialize results
        let out: Vec<IcpVal> = serde_json::from_slice(&out).map_err(|err| {
            BuildError::BuildFailed(format!("failed to deserialize results: {:?}", err))
        })?;

        // Check the number of results
        if out.len() != 1 {
            return Err(BuildError::BuildFailed(format!(
                "results were expected to have one element: {:?}",
                out
            )));
        }

        // Get the first result
        let out = out.get(0).ok_or_else(|| {
            BuildError::BuildFailed(format!("invalid result from canister builder: {:?}", out))
        })?;

        // Check the type of the result
        let out = match out {
            // Correct
            IcpVal::Result(out) => out,

            // Wrong
            _ => {
                return Err(BuildError::BuildFailed(format!(
                    "expected result, but got something else: {:?}",
                    out
                )));
            }
        };

        // Check the result and error
        let out = match out {
            // Correct
            Ok(Some(out)) => Ok(out),

            // Correct
            Err(Some(err)) => Err(err),

            // Wrong
            _ => {
                return Err(BuildError::BuildFailed(format!(
                    "invalid result from canister builder: {:?}",
                    out
                )));
            }
        };

        let out = out.map_err(|err| {
            BuildError::BuildFailed(format!("canister builder failed: {:?}", err))
        })?;

        // Extract the result
        let out = match out.as_ref() {
            // Correct
            IcpVal::String(out) => out,

            // Wrong
            _ => {
                return Err(BuildError::BuildFailed(format!(
                    "invalid result from canister builder: {:?}",
                    out
                )));
            }
        };

        Ok(out.to_owned())
    }
}
