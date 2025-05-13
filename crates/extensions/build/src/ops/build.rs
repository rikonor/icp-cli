use std::path::Path;

use dashmap::DashMap;
use serde::Serialize;

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
    fn build(&self, canister_dir: &str) -> Result<(), BuildError>;
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
    fn build(&self, canister_dir: &str) -> Result<(), BuildError> {
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

        let params = serde_json::to_vec(&[
            Val::String(canister_dir.to_string()), // canister-dir
        ])
        .expect("failed to serialize params");

        invoke(
            &interface_name, // interface_name
            &function_name,  // function_name
            &params,
        )
        .map_err(|err| BuildError::BuildFailed(err))?;

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum Val {
    // Primitive types
    Bool(bool),

    // Signed integers
    S8(i8),
    S32(i32),
    S64(i64),
    S16(i16),

    // Unsigned integers
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),

    // Floating point numbers
    Float32(f32),
    Float64(f64),

    // Text
    Char(char),
    String(String),

    // Containers
    Enum(String),
    List(Vec<Val>),
    Option(Option<Box<Val>>),
    Record(Vec<(String, Val)>),
    Result(Result<Option<Box<Val>>, Option<Box<Val>>>),
    Tuple(Vec<Val>),
    Variant(String, Option<Box<Val>>),

    // Other
    Flags(Vec<String>),
    // TODO: Figure out how to represent this
    // Resource(ResourceAny),
}
