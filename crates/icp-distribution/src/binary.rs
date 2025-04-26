use crate::error::{DistributionError, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;

#[derive(Debug, Serialize, Clone)]
pub struct BinaryInfo {
    pub name: String,
    pub target: String,
    pub variant: String,
    pub checksum: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ExtensionInfo {
    pub name: String,
    pub version: String, // Added version field
    pub file: String,
    pub checksum: String,
}

pub struct BinaryProcessor {
    binaries_dir: PathBuf,
    extensions_dir: PathBuf,
    checksums: HashMap<String, String>,
}

impl BinaryProcessor {
    /// Creates a new BinaryProcessor for the given directory
    pub fn new(
        binaries_dir: PathBuf,
        extensions_dir: PathBuf,
        checksums_path: PathBuf,
    ) -> Result<Self> {
        if !binaries_dir.exists() {
            return Err(DistributionError::BinaryNotFound(binaries_dir));
        }

        if !extensions_dir.exists() {
            return Err(DistributionError::ExtensionNotFound(extensions_dir));
        }

        if !checksums_path.exists() {
            return Err(DistributionError::MissingFile(checksums_path));
        }

        let checksums = fs::read_to_string(&checksums_path)?
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    Some((parts[1].to_string(), parts[0].to_string()))
                } else {
                    None
                }
            })
            .collect();

        Ok(Self {
            binaries_dir,
            extensions_dir,
            checksums,
        })
    }

    /// Validates a binary file's format and checksum
    pub fn validate_binary(&self, filename: &str) -> Result<()> {
        let file_path = self.binaries_dir.join(filename);
        if !file_path.exists() {
            return Err(DistributionError::BinaryNotFound(file_path));
        }

        // Validate filename format
        let parts: Vec<&str> = filename.split('-').collect();
        if parts.len() < 4 {
            return Err(DistributionError::InvalidFormat(format!(
                "Invalid filename format: {}. Expected format: icp-<arch>-<os>-<variant>",
                filename
            )));
        }

        // Verify checksum if available
        if let Some(expected_checksum) = self.checksums.get(filename) {
            let mut file = File::open(&file_path)?;
            let mut hasher = Sha256::new();
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            hasher.update(&buffer);
            let hash = format!("{:x}", hasher.finalize());

            if hash != *expected_checksum {
                return Err(DistributionError::ChecksumMismatch(filename.to_string()));
            }
        }

        Ok(())
    }

    /// Parses and validates all binary files in the directory
    pub fn parse_binary_info(&self) -> Result<Vec<BinaryInfo>> {
        self.parse_binaries()
    }

    /// Parse just the binaries, excluding extensions
    fn parse_binaries(&self) -> Result<Vec<BinaryInfo>> {
        let mut binaries = Vec::new();

        for entry in fs::read_dir(&self.binaries_dir)? {
            let entry = entry?;
            let filename = entry.file_name().to_string_lossy().to_string();
            if filename == "checksums.txt" {
                continue;
            }

            // Parse filename like: icp-x86_64-apple-darwin-standard
            let parts: Vec<&str> = filename.split('-').collect();
            if parts.len() >= 4 {
                // Validate the binary before including it
                self.validate_binary(&filename)?;

                binaries.push(BinaryInfo {
                    name: filename.clone(),
                    target: parts[1..parts.len() - 1].join("-"),
                    variant: parts.last().unwrap().to_string(),
                    checksum: self.checksums.get(&filename).cloned().unwrap_or_default(),
                });
            }
        }

        if binaries.is_empty() {
            return Err(DistributionError::BinaryNotFound(self.binaries_dir.clone()));
        }

        Ok(binaries)
    }

    /// Parse WebAssembly component extensions
    pub fn parse_extensions(&self) -> Result<Vec<ExtensionInfo>> {
        let mut extensions = Vec::new();
        for entry in fs::read_dir(&self.extensions_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Only process .wasm files
            if path.extension() == Some(OsStr::new("wasm")) {
                let filename = path.file_name().unwrap().to_string_lossy().to_string();

                // Extract name by removing .component.wasm or .wasm extension
                let name = filename
                    .strip_suffix(".component.wasm")
                    .or_else(|| filename.strip_suffix(".wasm"))
                    .unwrap_or(&filename)
                    .to_string();

                // Get checksum if available
                let checksum = self.checksums.get(&filename).cloned().unwrap_or_default();

                extensions.push(ExtensionInfo {
                    name,
                    version: "unknown".to_string(), // Placeholder version
                    file: filename,
                    checksum,
                });
            }
        }

        Ok(extensions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    fn setup_test_dirs() -> (TempDir, PathBuf, PathBuf, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let binaries_dir = temp_dir.path().join("binaries");
        let extensions_dir = temp_dir.path().join("extensions");
        let checksums_path = temp_dir.path().join("checksums.txt");

        fs::create_dir(&binaries_dir).unwrap();
        fs::create_dir(&extensions_dir).unwrap();

        (temp_dir, binaries_dir, extensions_dir, checksums_path)
    }

    fn create_test_binary(dir: PathBuf, name: &str, content: &[u8]) -> PathBuf {
        let path = dir.join(name);
        let mut file = File::create(&path).unwrap();
        file.write_all(content).unwrap();
        path
    }

    #[test]
    fn test_binary_validation() {
        let (_temp_dir, binaries_dir, extensions_dir, checksums_path) = setup_test_dirs();

        // Create test binary
        let binary_name = "icp-x86_64-apple-darwin-standard";
        let binary_content = b"test binary content";
        create_test_binary(binaries_dir.clone(), binary_name, binary_content);

        // Create checksums file
        let mut hasher = Sha256::new();
        hasher.update(binary_content);
        let checksum = format!("{:x}", hasher.finalize());

        let checksums_content = format!("{} {}", checksum, binary_name);
        fs::write(&checksums_path, checksums_content).unwrap();

        // Test validation
        let processor = BinaryProcessor::new(binaries_dir, extensions_dir, checksums_path).unwrap();
        assert!(processor.validate_binary(binary_name).is_ok());
    }

    #[test]
    fn test_invalid_binary_format() {
        let (_temp_dir, binaries_dir, extensions_dir, checksums_path) = setup_test_dirs();

        // Create invalid binary name
        let binary_name = "invalid-name";
        create_test_binary(binaries_dir.clone(), binary_name, b"test content");

        // Create empty checksums file
        fs::write(&checksums_path, "").unwrap();

        let processor = BinaryProcessor::new(binaries_dir, extensions_dir, checksums_path).unwrap();
        assert!(matches!(
            processor.validate_binary(binary_name),
            Err(DistributionError::InvalidFormat(_))
        ));
    }

    #[test]
    fn test_checksum_mismatch() {
        let (_temp_dir, binaries_dir, extensions_dir, checksums_path) = setup_test_dirs();

        // Create test binary
        let binary_name = "icp-x86_64-apple-darwin-standard";
        create_test_binary(binaries_dir.clone(), binary_name, b"test content");

        // Create checksums file with wrong checksum
        fs::write(&checksums_path, format!("wrong_checksum {}", binary_name)).unwrap();

        let processor = BinaryProcessor::new(binaries_dir, extensions_dir, checksums_path).unwrap();
        assert!(matches!(
            processor.validate_binary(binary_name),
            Err(DistributionError::ChecksumMismatch(_))
        ));
    }
}
