use std::path::Path;

use crate::{
    CanisterManifest, ProjectManifest, bindings::exports::icp::project::lib::CanisterInfo,
};

#[derive(Debug, thiserror::Error)]
pub enum ListError {
    #[error("Failed to process project or canister manifest: {0}")]
    ManifestProcessing(String),

    #[error("No canisters found in the project based on icp.toml members")]
    NoCanistersFound,

    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl From<ListError> for String {
    fn from(e: ListError) -> Self {
        match e {
            ListError::ManifestProcessing(msg) => {
                format!("Error processing manifest file: {}", msg)
            }
            ListError::NoCanistersFound => e.to_string(),
            ListError::Unexpected(err) => {
                format!("An unexpected error occurred: {}", err)
            }
        }
    }
}

impl From<ListError> for u8 {
    fn from(e: ListError) -> Self {
        match e {
            ListError::ManifestProcessing(_) => 1,
            ListError::Unexpected(_) => 2,
            ListError::NoCanistersFound => 3,
        }
    }
}

pub trait List {
    fn list(&self) -> Result<Vec<CanisterInfo>, ListError>;
}

pub struct Lister {
    read_file: Box<dyn Fn(&str) -> Result<Vec<u8>, String>>,
}

impl Lister {
    pub fn new(read_file: Box<dyn Fn(&str) -> Result<Vec<u8>, String>>) -> Self {
        Lister { read_file }
    }
}

impl List for Lister {
    fn list(&self) -> Result<Vec<CanisterInfo>, ListError> {
        // 1. Read icp.toml
        let icp_toml_content_bytes = (self.read_file)("icp.toml").map_err(|e| {
            ListError::ManifestProcessing(format!("Failed to read icp.toml: {}", e))
        })?;

        let icp_toml_content_str = String::from_utf8(icp_toml_content_bytes).map_err(|e| {
            ListError::ManifestProcessing(format!("Failed to decode icp.toml: {}", e))
        })?;

        // 2. Parse icp.toml
        let project_manifest: ProjectManifest =
            toml::from_str(&icp_toml_content_str).map_err(|e| {
                ListError::ManifestProcessing(format!("Failed to parse icp.toml: {}", e))
            })?;

        let mut canister_infos = Vec::new();

        // 3. Iterate through members
        for member_path_str in &project_manifest.workspace.members {
            let canister_toml_path = Path::new(member_path_str).join("canister.toml");
            let canister_toml_path_str = canister_toml_path.to_string_lossy();

            // 4. Read and parse canister.toml for each member
            match self.read_and_parse_canister_toml(&canister_toml_path_str) {
                Ok(canister_manifest) => {
                    canister_infos.push(CanisterInfo {
                        name: canister_manifest.canister.name,
                        canister_type: canister_manifest.canister.canister_type,
                        path: member_path_str.clone(),
                    });
                }
                Err(e) => {
                    // Propagate the first error encountered during canister manifest processing.
                    return Err(e);
                }
            }
        }

        if canister_infos.is_empty() {
            return Err(ListError::NoCanistersFound);
        }

        Ok(canister_infos)
    }
}

// Add helper method to Lister to use injected read_file
impl Lister {
    fn read_and_parse_canister_toml(&self, path_str: &str) -> Result<CanisterManifest, ListError> {
        let content_bytes = (self.read_file)(path_str).map_err(|e| {
            ListError::ManifestProcessing(format!("Failed to read {}: {}", path_str, e))
        })?;

        let content_str = String::from_utf8(content_bytes).map_err(|e| {
            ListError::ManifestProcessing(format!("Failed to decode {}: {}", path_str, e))
        })?;

        toml::from_str(&content_str).map_err(|e| {
            ListError::ManifestProcessing(format!("Failed to parse {}: {}", path_str, e))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Error};

    #[test]
    fn test_list_no_canisters_found() {
        let mock_read_file = |path: &str| -> Result<Vec<u8>, String> {
            match path {
                "icp.toml" => Ok(b"[workspace]\nmembers = []\n".to_vec()),
                _ => Err(format!("Mock: Unexpected file read '{}'", path)),
            }
        };

        let result = Lister::new(Box::new(mock_read_file)).list();

        assert!(
            matches!(result, Err(ListError::NoCanistersFound)),
            "Expected NoCanistersFound error, got {:?}",
            result
        );
    }

    #[test]
    fn test_list_finds_valid_canisters() -> Result<(), Error> {
        let mock_read_file = |path: &str| -> Result<Vec<u8>, String> {
            match path {
                "icp.toml" => Ok(
                    b"[workspace]\nmembers = [\"canisters/canister-1\", \"canisters/canister-2\"]\n".to_vec(),
                ),
                "canisters/canister-1/canister.toml" => {
                    Ok(b"[canister]\nname = \"canister-1\"\ntype = \"rust\"\n".to_vec())
                }
                "canisters/canister-2/canister.toml" => {
                    Ok(b"[canister]\nname = \"canister-2\"\ntype = \"motoko\"\n".to_vec())
                }
                _ => Err(format!("Mock: Unexpected file read '{}'", path)),
            }
        };

        let out = Lister::new(Box::new(mock_read_file))
            .list()
            .context("failed to list canisters")?;

        assert_eq!(out.len(), 2, "Expected to find 2 canisters");

        assert_eq!(out[0].name, "canister-1");
        assert_eq!(out[0].canister_type, "rust");
        assert_eq!(out[0].path, "canisters/canister-1");

        assert_eq!(out[1].name, "canister-2");
        assert_eq!(out[1].canister_type, "motoko");
        assert_eq!(out[1].path, "canisters/canister-2");

        Ok(())
    }
}
