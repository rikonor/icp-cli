# Quick Install Task

## Overview

Create a curl-based installation method for icp that works across all supported platforms, similar to popular tools that offer `curl -sL https://get.example.com | sh` installation.

## Scope

- Shell script for Unix-like systems
- PowerShell variant for Windows
- Platform/architecture detection
- Binary selection and verification
- Domain setup (get.icp-cli.com)
- Security measures implementation

## Status

- Current Phase: In Progress
- Progress: 90%
- Last Updated: 2025-03-20

## Implementation Details

### Script Development

- ✅ Cross-platform shell script (install.sh)
- ✅ PowerShell script for Windows (install.ps1)
- ✅ Platform detection logic
- ✅ Binary selection and verification
- ✅ Error handling and feedback

### Infrastructure

- Domain registration and setup (get.icp-cli.com)
- ✅ GitHub Pages configuration for static file hosting
  - Using .nojekyll for direct file serving
  - No Jekyll processing required
  - Secure, efficient distribution of binaries and scripts
- ✅ GitHub Actions workflow for deployment
  - Generates installation scripts and landing page from templates
  - Processes and verifies binary artifacts
  - Creates checksums for security
  - Deploys to gh-pages branch
  - Validates all generated files
- SSL certificate configuration (via GitHub Pages)

### File Organization

- ✅ Templates in icp-distribution crate
  - install.sh.tmpl and install.ps1.tmpl for installation scripts
  - Landing page template for documentation
- ✅ Script generation system
  - Centralized in generate_scripts.rs
  - Handles template rendering
  - Creates all necessary deployment files
- ✅ Deployment structure
  - All files generated during deployment
  - No static copies in documentation
  - Single source of truth in templates

### Distribution Crate Development

- ✅ Create new `icp-distribution` crate
- ✅ Implement template system for installation scripts
- ✅ Convert existing scripts to templates
- ✅ Setup build integration for script generation

## Dependencies

- GitHub release artifacts
- Domain registration
- SSL certificates

## Technical Challenges

- Cross-platform compatibility
- Security verification
- Error handling
- Platform detection reliability

## Success Criteria

- Installation works on all platforms
- Security measures implemented
- Clear error handling
- User-friendly experience

## Notes

Focus on security and reliability while keeping the installation process simple and user-friendly.
