use crate::error::{DistributionError, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
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

pub struct BinaryProcessor {
    path: PathBuf,
    checksums: HashMap<String, String>,
}

impl BinaryProcessor {
    /// Creates a new BinaryProcessor for the given directory
    pub fn new(path: PathBuf) -> Result<Self> {
        if !path.exists() {
            return Err(DistributionError::BinaryNotFound(path));
        }

        let checksums_path = path.join("checksums.txt");
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

        Ok(Self { path, checksums })
    }

    /// Validates a binary file's format and checksum
    pub fn validate_binary(&self, filename: &str) -> Result<()> {
        let file_path = self.path.join(filename);
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
        let mut binaries = Vec::new();

        for entry in fs::read_dir(&self.path)? {
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
            return Err(DistributionError::BinaryNotFound(self.path.clone()));
        }

        Ok(binaries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_binary(dir: &TempDir, name: &str, content: &[u8]) -> PathBuf {
        let path = dir.path().join(name);
        let mut file = File::create(&path).unwrap();
        file.write_all(content).unwrap();
        path
    }

    #[test]
    fn test_binary_validation() {
        let temp_dir = TempDir::new().unwrap();

        // Create test binary
        let binary_name = "icp-x86_64-apple-darwin-standard";
        let binary_content = b"test binary content";
        create_test_binary(&temp_dir, binary_name, binary_content);

        // Create checksums file
        let mut hasher = Sha256::new();
        hasher.update(binary_content);
        let checksum = format!("{:x}", hasher.finalize());

        let checksums_content = format!("{} {}", checksum, binary_name);
        let checksums_path = temp_dir.path().join("checksums.txt");
        fs::write(&checksums_path, checksums_content).unwrap();

        // Test validation
        let processor = BinaryProcessor::new(temp_dir.path().to_path_buf()).unwrap();
        assert!(processor.validate_binary(binary_name).is_ok());
    }

    #[test]
    fn test_invalid_binary_format() {
        let temp_dir = TempDir::new().unwrap();

        // Create invalid binary name
        let binary_name = "invalid-name";
        create_test_binary(&temp_dir, binary_name, b"test content");

        // Create empty checksums file
        fs::write(temp_dir.path().join("checksums.txt"), "").unwrap();

        let processor = BinaryProcessor::new(temp_dir.path().to_path_buf()).unwrap();
        assert!(matches!(
            processor.validate_binary(binary_name),
            Err(DistributionError::InvalidFormat(_))
        ));
    }

    #[test]
    fn test_checksum_mismatch() {
        let temp_dir = TempDir::new().unwrap();

        // Create test binary
        let binary_name = "icp-x86_64-apple-darwin-standard";
        create_test_binary(&temp_dir, binary_name, b"test content");

        // Create checksums file with wrong checksum
        fs::write(
            temp_dir.path().join("checksums.txt"),
            format!("wrong_checksum {}", binary_name),
        )
        .unwrap();

        let processor = BinaryProcessor::new(temp_dir.path().to_path_buf()).unwrap();
        assert!(matches!(
            processor.validate_binary(binary_name),
            Err(DistributionError::ChecksumMismatch(_))
        ));
    }
}
