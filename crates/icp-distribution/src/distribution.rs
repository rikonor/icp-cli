use crate::error::{DistributionError, Result};
use std::str::FromStr;

/// Represents the distribution method for the ICP CLI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Distribution {
    /// Standard installation (e.g., cargo install, manual download)
    Standard,
    /// Homebrew package manager (macOS and Linux)
    Homebrew,
    /// NuGet package manager (Windows)
    NuGet,
    /// APT package manager (Debian/Ubuntu)
    Apt,
}

impl FromStr for Distribution {
    type Err = DistributionError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "standard" => Ok(Self::Standard),
            "homebrew" | "brew" => Ok(Self::Homebrew),
            "nuget" => Ok(Self::NuGet),
            "apt" | "aptitude" => Ok(Self::Apt),
            _ => Err(DistributionError::InvalidDistribution(s.to_string())),
        }
    }
}

impl TryFrom<&str> for Distribution {
    type Error = DistributionError;

    fn try_from(s: &str) -> Result<Self> {
        Self::from_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_distribution_from_str() {
        assert_eq!(
            Distribution::from_str("standard").unwrap(),
            Distribution::Standard
        );
        assert_eq!(
            Distribution::from_str("homebrew").unwrap(),
            Distribution::Homebrew
        );
        assert_eq!(
            Distribution::from_str("brew").unwrap(),
            Distribution::Homebrew
        );
        assert_eq!(
            Distribution::from_str("nuget").unwrap(),
            Distribution::NuGet
        );
        assert_eq!(Distribution::from_str("apt").unwrap(), Distribution::Apt);
        assert_eq!(
            Distribution::from_str("aptitude").unwrap(),
            Distribution::Apt
        );

        // Test case insensitivity
        assert_eq!(
            Distribution::from_str("HOMEBREW").unwrap(),
            Distribution::Homebrew
        );
        assert_eq!(
            Distribution::from_str("Standard").unwrap(),
            Distribution::Standard
        );

        // Test invalid distribution
        assert!(matches!(
            Distribution::from_str("invalid"),
            Err(DistributionError::InvalidDistribution(_))
        ));
    }
}
