# Distribution Project

## Purpose

Make icp easily accessible to users across all platforms through multiple distribution channels, including package managers and a quick installation method.

## Status

In Progress (15% Complete)

## Tasks

- [Not Started] Documentation Task

  - Create comprehensive installation guides
  - Current progress: 0%
  - Located in: `tasks/documentation/`

- [In Progress] Quick Install Task

  - Implement curl-based installation method
  - Current progress: 50%
  - Located in: `tasks/quick-install/`

- [Not Started] Package Manager Tasks

  - Homebrew package for macOS/Linux
  - Current progress: 0%
  - Located in: `tasks/package-managers/homebrew/`

  - Cargo package for Rust users
  - Current progress: 0%
  - Located in: `tasks/package-managers/cargo/`

  - Chocolatey package for Windows
  - Current progress: 0%
  - Located in: `tasks/package-managers/chocolatey/`

  - Linux packages (APT/RPM)
  - Current progress: 0%
  - Located in: `tasks/package-managers/linux-packages/`

## Project Overview

The Distribution project aims to make icp easily accessible to users through familiar installation methods on each platform. This includes a quick curl-based installation, native package managers, and comprehensive documentation.

## Current Focus

Initial implementation of quick installation scripts is complete. The next phase involves creating a dedicated `icp-distribution` crate to manage installation scripts and prepare for integration with multiple package managers. This approach allows for consistent version management across all distribution channels.

## Next Steps

1. Documentation preparation:

   - Platform compatibility matrix
   - Installation guide templates
   - Security documentation

2. Quick Install implementation:

   - ✅ Shell script development
   - ✅ PowerShell script development
   - Create `icp-distribution` crate
   - Domain setup
   - Security measures implementation

3. Package manager setup:
   - Start with Homebrew
   - Expand to other platforms
   - Automate release process

## Dependencies

This project builds on the existing CI workflow that produces cross-platform releases.

## Known Issues

1. Domain acquisition for get.icp-cli.com needed
2. Package signing requirements vary by platform
3. Version synchronization across package managers

## Timeline

Quick Install and initial documentation: 2-3 weeks
Package manager integration: 4-6 weeks (parallel development)
