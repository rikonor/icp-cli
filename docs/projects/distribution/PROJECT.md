# Distribution Project

## Purpose

Make icp easily accessible to users across all platforms through multiple distribution channels, including package managers and a quick installation method.

## Project Overview

The Distribution project aims to make icp easily accessible to users through familiar installation methods on each platform. This includes a quick curl-based installation, native package managers, and comprehensive documentation.

## Current Focus

The project has established a robust distribution infrastructure through the `icp-distribution` crate, which now supports:

- Template-based script generation
- Distribution-aware builds
- Automated deployment processes
- Extension management

We are transitioning from the quick-install implementation to package manager integration, starting with Homebrew.

## Tasks

### Documentation Task [Not Started]

- Create comprehensive installation guides
- Located in: `tasks/documentation/`
- Subtasks to be defined

### Quick Install Task [Near Completion]

- Implement curl-based installation method
- Located in: `tasks/quick-install/`
- Remaining work:
  - Domain setup
  - Final security measures

### Package Manager Tasks

#### Homebrew [In Progress]

- Located in: `tasks/package-managers/homebrew/`
- Current subtask: Distribution Framework Implementation
- Next: Path Configuration
- See task file for detailed subtask sequence
- Security Enhancements:
  - [ ] Add --force flag for extension overwrites
  - [ ] Implement checksum validation for installed extensions

#### Cargo [Not Started]

- Located in: `tasks/package-managers/cargo/`
- Subtasks to be defined

#### Chocolatey [Not Started]

- Located in: `tasks/package-managers/chocolatey/`
- Subtasks to be defined

#### Linux Packages [Not Started]

- Located in: `tasks/package-managers/linux-packages/`
- Subtasks to be defined

## Infrastructure

### icp-distribution Crate

- Template-based script generation
- Distribution variant support
- Build-time configuration
- Extension management

### CI/CD Integration

- GitHub Actions workflows
- Automated deployments
- Cross-platform builds
- Release synchronization

## Dependencies

- Existing CI workflow for cross-platform releases
- homebrew-icp-cli tap repository
- GitHub Pages infrastructure

## Known Issues

1. Domain acquisition for get.icp-cli.com needed
2. Package signing requirements vary by platform
3. Version synchronization across package managers

## Next Steps

1. Complete remaining quick-install security measures
2. Implement Homebrew distribution variant
3. Setup automated formula updates
4. Expand to additional package managers
