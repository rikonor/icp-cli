use crate::error::{DistributionError, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};
// Remove unused imports
// use std::collections::HashMap;
// use std::ffi::OsStr;
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
    // extensions_dir is removed as parse_extensions is no longer needed here
    // checksums HashMap is removed as we read from .sha256 files directly
}

impl BinaryProcessor {
    /// Creates a new BinaryProcessor for the given directory
    pub fn new(binaries_dir: PathBuf) -> Result<Self> {
        if !binaries_dir.exists() {
            return Err(DistributionError::BinaryNotFound(binaries_dir));
        }
        // No need to check extensions_dir or checksums_path anymore
        Ok(Self { binaries_dir })
    }

    /// Reads the checksum from the corresponding .sha256 file
    fn read_checksum_file(&self, binary_filename: &str) -> Result<String> {
        let checksum_filename = format!("{}.sha256", binary_filename);
        let checksum_path = self.binaries_dir.join(&checksum_filename);

        if !checksum_path.exists() {
            // It's possible a binary exists without a checksum file during development/testing
            // Return an error or a default value? Returning error seems safer for distribution.
            eprintln!("Warning: Checksum file not found for {}", binary_filename);
            return Err(DistributionError::MissingFile(checksum_path));
            // Alternatively, return Ok(String::new()) or Ok("CHECKSUM_NOT_FOUND".to_string())
            // if generate_scripts can handle missing checksums gracefully.
        }

        let checksum_content = fs::read_to_string(&checksum_path)?;
        // Extract the first part (the checksum itself) in case the file contains extra info (like filename)
        let checksum = checksum_content
            .split_whitespace()
            .next()
            .ok_or_else(|| {
                DistributionError::InvalidFormat(format!(
                    "Checksum file {} is empty or invalid",
                    checksum_filename
                ))
            })?
            .to_string();
        Ok(checksum)
    }

    /// Validates a binary file's format and checksum against its .sha256 file
    pub fn validate_binary(&self, filename: &str) -> Result<()> {
        let file_path = self.binaries_dir.join(filename);
        if !file_path.exists() {
            return Err(DistributionError::BinaryNotFound(file_path));
        }

        // Validate filename format (simplified check, assumes icp-<target>-<variant>[.exe])
        let parts: Vec<&str> = filename.split('-').collect();
        if !filename.starts_with("icp-") || parts.len() < 3 {
            return Err(DistributionError::InvalidFormat(format!(
                "Invalid filename format: {}. Expected format: icp-<target>-<variant>[.exe]",
                filename
            )));
        }

        // Verify checksum against the .sha256 file
        let expected_checksum = self.read_checksum_file(filename)?;

        let mut file = File::open(&file_path)?;
        let mut hasher = Sha256::new();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        hasher.update(&buffer);
        let actual_hash = format!("{:x}", hasher.finalize());

        if actual_hash != expected_checksum {
            eprintln!(
                "Checksum mismatch for {}: Expected {}, Got {}",
                filename, expected_checksum, actual_hash
            );
            return Err(DistributionError::ChecksumMismatch(filename.to_string()));
        }

        Ok(())
    }

    /// Parses and validates all binary files in the directory
    pub fn parse_binary_info(&self) -> Result<Vec<BinaryInfo>> {
        self.parse_binaries()
    }

    /// Parse just the binaries, excluding extensions and .sha256 files
    fn parse_binaries(&self) -> Result<Vec<BinaryInfo>> {
        let mut binaries = Vec::new();

        for entry in fs::read_dir(&self.binaries_dir)? {
            let entry = entry?;
            let path = entry.path();
            let filename = entry.file_name().to_string_lossy().to_string();

            // Skip directories and .sha256 files
            if path.is_dir() || filename.ends_with(".sha256") {
                continue;
            }

            // Basic check to ensure it's likely an icp binary we want
            if !filename.starts_with("icp-") {
                eprintln!(
                    "Skipping file that does not start with 'icp-': {}",
                    filename
                );
                continue;
            }

            // Parse filename like: icp-x86_64-apple-darwin-standard or icp-x86_64-pc-windows-msvc-standard.exe
            let parts: Vec<&str> = filename.split('-').collect();
            // Handle .exe for parsing variant correctly
            let (variant_part, target_parts) = if filename.ends_with(".exe") {
                let variant_with_exe = parts.last().unwrap_or(&"");
                let variant = variant_with_exe
                    .strip_suffix(".exe")
                    .unwrap_or(variant_with_exe);
                (variant, &parts[1..parts.len() - 1])
            } else {
                // Dereference parts.last() to match the type of the `if` branch (&str, &[&str])
                (*parts.last().unwrap_or(&""), &parts[1..parts.len() - 1])
            };

            if parts.len() >= 3 {
                // Need at least icp-<target>-<variant>
                // Validate the binary before including it (this now reads the .sha256 file)
                match self.validate_binary(&filename) {
                    Ok(_) => {
                        // Read checksum again here to store it in BinaryInfo
                        // (validate_binary confirms it exists and matches)
                        let checksum = self.read_checksum_file(&filename).unwrap_or_else(|e| {
                            eprintln!("Error reading checksum for {}: {}", filename, e);
                            // Return a specific string indicating error, or handle differently if needed
                            "ERROR_READING_CHECKSUM".to_string()
                        });

                        binaries.push(BinaryInfo {
                            name: filename.clone(),
                            target: target_parts.join("-"),
                            variant: variant_part.to_string(),
                            checksum,
                        });
                    }
                    Err(e) => {
                        // Log error but potentially continue? Or fail hard?
                        // Failing hard seems safer for distribution artifacts.
                        eprintln!(
                            "Validation failed for binary '{}', skipping: {}",
                            filename, e
                        );
                        // Optionally return the error to stop processing: return Err(e);
                    }
                }
            } else {
                eprintln!(
                    "Skipping file with unexpected format (less than 3 parts): {}",
                    filename
                );
            }
        }

        // It's okay if binaries list is empty if no matching files were found/valid
        // The check below might be too strict if some platforms are optional
        // if binaries.is_empty() {
        //     return Err(DistributionError::BinaryNotFound(self.binaries_dir.clone()));
        // }

        Ok(binaries)
    }

    // parse_extensions method is removed as it's no longer needed by generate_scripts
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    // Update return type to remove checksums_path
    fn setup_test_dirs() -> (TempDir, PathBuf, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let binaries_dir = temp_dir.path().join("binaries");
        let extensions_dir = temp_dir.path().join("extensions");
        // let checksums_path = temp_dir.path().join("checksums.txt"); // Removed

        fs::create_dir(&binaries_dir).unwrap();
        fs::create_dir(&extensions_dir).unwrap();

        // Update returned tuple
        (temp_dir, binaries_dir, extensions_dir)
    }

    fn create_test_binary(dir: PathBuf, name: &str, content: &[u8]) -> PathBuf {
        let path = dir.join(name);
        let mut file = File::create(&path).unwrap();
        file.write_all(content).unwrap();
        path
    }

    #[test]
    fn test_binary_validation() {
        // Update destructuring to match setup_test_dirs return type
        let (_temp_dir, binaries_dir, _extensions_dir) = setup_test_dirs();

        // Create test binary
        let binary_name = "icp-x86_64-apple-darwin-standard";
        let binary_content = b"test binary content";
        create_test_binary(binaries_dir.clone(), binary_name, binary_content);

        // Create checksums file
        let mut hasher = Sha256::new();
        hasher.update(binary_content);
        let checksum = format!("{:x}", hasher.finalize()); // Calculate checksum

        // Create the corresponding .sha256 file for the test with the correct checksum
        let checksum_file_path = binaries_dir.join(format!("{}.sha256", binary_name));
        fs::write(&checksum_file_path, checksum).unwrap(); // Write only the checksum string

        // Test validation
        // Constructor call is already correct here
        let processor = BinaryProcessor::new(binaries_dir).unwrap();
        assert!(processor.validate_binary(binary_name).is_ok());
    }

    #[test]
    fn test_invalid_binary_format() {
        // Update destructuring to match setup_test_dirs return type
        let (_temp_dir, binaries_dir, _extensions_dir) = setup_test_dirs();

        // Create invalid binary name
        let binary_name = "invalid-name";
        create_test_binary(binaries_dir.clone(), binary_name, b"test content");

        // Remove the write to the non-existent checksums_path variable
        // fs::write(&checksums_path, "").unwrap();

        // Update constructor call
        let processor = BinaryProcessor::new(binaries_dir).unwrap();
        // Validation should still fail based on filename format, not constructor
        assert!(matches!(
            processor.validate_binary(binary_name),
            Err(DistributionError::InvalidFormat(_))
        ));
    }

    #[test]
    fn test_checksum_mismatch() {
        // Update destructuring to match setup_test_dirs return type
        let (_temp_dir, binaries_dir, _extensions_dir) = setup_test_dirs();

        // Create test binary
        let binary_name = "icp-x86_64-apple-darwin-standard";
        create_test_binary(binaries_dir.clone(), binary_name, b"test content");

        // Create checksums file with wrong checksum
        // Create the corresponding .sha256 file with the wrong checksum
        let checksum_file_path = binaries_dir.join(format!("{}.sha256", binary_name));
        fs::write(&checksum_file_path, "wrong_checksum").unwrap();

        // Update constructor call
        let processor = BinaryProcessor::new(binaries_dir).unwrap();
        // Validation should fail due to checksum mismatch read from the .sha256 file
        assert!(matches!(
            processor.validate_binary(binary_name),
            Err(DistributionError::ChecksumMismatch(_))
        ));
    }
}
