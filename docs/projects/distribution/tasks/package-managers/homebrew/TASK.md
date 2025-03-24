# Homebrew Package Task

## Overview

Create and maintain a Homebrew formula for icp, enabling macOS and Linux users to install via Homebrew. The formula will handle both the CLI installation and extension management through a distribution-aware build process.

## Scope

- Distribution-aware build system
- Homebrew formula creation
- Multi-architecture support
- Tap repository integration
- Automated updates

## Implementation Details

The implementation centers around a distribution-aware build system that allows icp-cli to adapt its paths based on the installation method. This enables proper extension management within Homebrew's constraints.

### Distribution Framework

The `icp-distribution` crate will provide a Distribution enum:

```rust
pub enum Distribution {
    Standard,
    Homebrew,
    NuGet,
    Apt,
}

impl TryFrom<&str> for Distribution {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "standard" => Ok(Distribution::Standard),
            "homebrew" | "brew" => Ok(Distribution::Homebrew),
            "nuget" => Ok(Distribution::NuGet),
            "aptitutde" | "apt" => Ok(Distribution::Apt),
            _ => Err("Invalid DISTRIBUTION value"),
        }
    }
}
```

Build-time validation ensures distribution values are correct:

```rust
// build.rs
use std::env;

fn main() {
    if let Some(distribution) = env::var("DISTRIBUTION").ok() {
        if let Err(e) = Distribution::try_from(distribution.as_str()) {
            panic!(
                "❌ ERROR: Invalid DISTRIBUTION value '{}': {}",
                distribution, e
            );
        }
        println!("cargo:rustc-env=DISTRIBUTION={}", distribution);
    } else {
        println!("cargo:rustc-env=DISTRIBUTION=standard");
    }
}
```

CLI integration uses this to determine appropriate paths:

```rust
static DISTRIBUTION: Lazy<Distribution> = Lazy::new(|| {
    match option_env!("DISTRIBUTION")
        .map(Distribution::try_from)
        .transpose()
    {
        Ok(Some(distribution)) => distribution,
        Err(e) => {
            eprintln!("⚠️ Warning: {}. Falling back to Standard.", e);
            Distribution::Standard
        }
        _ => Distribution::Standard,
    }
});

static DEFAULT_PATH_MANIFEST: Lazy<PathBuf> = Lazy::new(|| match *DISTRIBUTION {
    Distribution::Standard => dirs::home_dir()
        .expect("no home dir found")
        .join(format!(".{SERVICE_NAME}/manifest.json")),
    Distribution::Homebrew => unimplemented!("homebrew"),
    Distribution::NuGet => unimplemented!("nuget"),
    Distribution::Apt => unimplemented!("apt"),
});
```

## Subtasks

1. [x] Distribution Framework Implementation

   - Add Distribution enum to icp-distribution crate
   - Implement TryFrom for distribution parsing
   - Add build.rs for distribution validation
   - Add tests for distribution parsing
   - Success: Distribution framework compiles and validates input
   - Next: Path Configuration

2. [x] Path Configuration

   - Dependencies: Distribution Framework
   - Implement Homebrew path resolution using `brew --prefix`
   - Update DEFAULT_PATH_MANIFEST for Homebrew variant
   - Add tests for path resolution
   - Success: Paths correctly resolve in Homebrew context
   - Next: Build Integration

3. [x] Build Integration

   - Dependencies: Path Configuration
   - Add DISTRIBUTION environment variable support to build process
   - Update Makefile/build scripts
   - Add distribution-specific build tests
   - Success: CLI builds with correct distribution settings
   - Next: Formula Creation

4. [x] Formula Creation

   - Dependencies: Build Integration
   - Create initial Homebrew formula template ✓
   - Configure multi-architecture support ✓
   - Add extension installation commands ✓
   - Implement post_install hook for extension management ✓
   - Test installation flow ✓
   - Success: Formula installs CLI and extensions ✓
   - Next: CI Integration

5. [ ] CI Integration

   - Dependencies: Formula Creation
   - Setup workflow to update homebrew-icp-cli repository
   - Add version synchronization
   - Configure automated testing
   - Test release process
   - Success: Formula auto-updates on releases
   - Next: Documentation

6. [ ] Documentation

   - Dependencies: CI Integration
   - Update installation guides
   - Add troubleshooting section
   - Document extension handling
   - Success: Documentation complete and accurate

## Success Criteria

Primary success criterion: Users can install icp-cli and its extensions via:

```bash
brew tap rikonor/icp-cli
brew install icp-cli
```

Additional criteria:

- All extensions are properly installed and configured
- Paths are correctly set for Homebrew environment
- Updates work properly through brew
- CI/CD pipeline maintains formula automatically

## Technical Challenges

- Path management in Homebrew context
- Extension installation during formula execution
- Version synchronization across repositories
- Multi-architecture support

## Dependencies

- GitHub release artifacts
- homebrew-icp-cli tap repository
- icp-distribution crate
- CI/CD infrastructure

## Notes

The distribution-aware build system provides a foundation for supporting multiple package managers beyond Homebrew, while ensuring each variant can properly handle extension management within its environment's constraints.
