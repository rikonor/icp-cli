use crate::error::{DistributionError, Result};

/// Handles URL construction and validation for distribution assets
pub struct UrlBuilder {
    domain: String,
    base_path: String,
    repo_url: String,
}

impl UrlBuilder {
    /// Creates a new UrlBuilder with the given domain and repository URL
    pub fn new(domain: impl Into<String>, repo_url: impl Into<String>) -> Self {
        let domain = domain.into();
        // Strip any trailing slashes
        let domain = domain.trim_end_matches('/').to_string();

        Self {
            domain,
            base_path: "binaries/icp".to_string(),
            repo_url: repo_url.into(),
        }
    }

    /// Sets a custom base path for binary locations
    pub fn with_base_path(mut self, path: impl Into<String>) -> Self {
        self.base_path = path.into().trim_matches('/').to_string();
        self
    }

    /// Constructs the base URL with proper protocol
    fn base_url(&self) -> String {
        if self.domain.starts_with("http://") || self.domain.starts_with("https://") {
            self.domain.clone()
        } else {
            format!("https://{}", self.domain)
        }
    }

    /// Gets the URL for binary downloads
    pub fn binary_url(&self) -> Result<String> {
        if self.domain.is_empty() {
            return Err(DistributionError::UrlError(
                "Domain cannot be empty".to_string(),
            ));
        }
        Ok(format!("{}/{}", self.base_url(), self.base_path))
    }

    /// Gets the URL for binary checksums
    pub fn checksum_url(&self) -> Result<String> {
        self.binary_url()
    }

    /// Gets the base URL for GitHub Pages without the binary path
    pub fn pages_url(&self) -> Result<String> {
        if self.domain.is_empty() {
            return Err(DistributionError::UrlError(
                "Domain cannot be empty".to_string(),
            ));
        }
        Ok(self.base_url())
    }

    /// Gets the repository URL
    pub fn repo_url(&self) -> Result<String> {
        Ok(self.repo_url.clone())
    }

    /// Gets the URL for a specific binary file
    pub fn binary_file_url(&self, filename: &str) -> Result<String> {
        Ok(format!("{}/{}", self.binary_url()?, filename))
    }

    /// Gets the URL for the checksums file
    pub fn checksums_file_url(&self) -> Result<String> {
        Ok(format!("{}/checksums.txt", self.binary_url()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_builder_with_domain() {
        let builder = UrlBuilder::new("example.com", "https://github.com/example/repo");
        assert_eq!(
            builder.binary_url().unwrap(),
            "https://example.com/binaries/icp"
        );
    }

    #[test]
    fn test_url_builder_with_https() {
        let builder = UrlBuilder::new("https://example.com", "https://github.com/example/repo");
        assert_eq!(
            builder.binary_url().unwrap(),
            "https://example.com/binaries/icp"
        );
    }

    #[test]
    fn test_url_builder_with_trailing_slash() {
        let builder = UrlBuilder::new("example.com/", "https://github.com/example/repo");
        assert_eq!(
            builder.binary_url().unwrap(),
            "https://example.com/binaries/icp"
        );
    }

    #[test]
    fn test_url_builder_with_custom_path() {
        let builder = UrlBuilder::new("example.com", "https://github.com/example/repo")
            .with_base_path("custom/path");
        assert_eq!(
            builder.binary_url().unwrap(),
            "https://example.com/custom/path"
        );
    }

    #[test]
    fn test_url_builder_empty_domain() {
        let builder = UrlBuilder::new("", "https://github.com/example/repo");
        assert!(matches!(
            builder.binary_url(),
            Err(DistributionError::UrlError(_))
        ));
    }

    #[test]
    fn test_binary_file_url() {
        let builder = UrlBuilder::new("example.com", "https://github.com/example/repo");
        assert_eq!(
            builder.binary_file_url("test-binary").unwrap(),
            "https://example.com/binaries/icp/test-binary"
        );
    }

    #[test]
    fn test_checksums_file_url() {
        let builder = UrlBuilder::new("example.com", "https://github.com/example/repo");
        assert_eq!(
            builder.checksums_file_url().unwrap(),
            "https://example.com/binaries/icp/checksums.txt"
        );
    }
}
