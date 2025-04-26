---
metadata:
  description: Procedure for releasing ICP CLI and extensions
---

# Releasing ICP CLI and Extensions

**Purpose:** To document the standard procedure for creating new releases for the main `icp-cli` tool and its individual extensions.

## Release Process Overview

Releases are triggered by pushing Git tags that follow semantic versioning patterns. Dedicated Make targets simplify the process of updating versions, creating tags, and pushing them.

## Releasing the Main ICP CLI Tool

1.  **Determine the new version:** Decide on the next semantic version (e.g., `vX.Y.Z`) based on the changes included.
2.  **Run the Make target:** Execute the following command from the repository root, replacing `<NEW_VERSION>` with the chosen version (e.g., `v0.2.0`):
    ```bash
    make release-cli VERSION=<NEW_VERSION>
    ```
3.  **What it does:**
    - Updates the version in relevant `Cargo.toml` files.
    - Commits the version changes.
    - Creates a Git tag with the specified version.
    - Pushes the commit and the tag to the remote repository (`origin`).
    - The pushed tag triggers the `release.yml` GitHub Actions workflow, which builds binaries and creates the GitHub Release.

## Releasing an Individual Extension

1.  **Determine the new version:** Decide on the next semantic version (e.g., `vA.B.C`) for the specific extension.
2.  **Identify the extension name:** Use the directory name under `crates/extensions/` (e.g., `identity`, `build`).
3.  **Run the Make target:** Execute the following command from the repository root, replacing `<EXTENSION_NAME>` and `<NEW_VERSION>`:
    ```bash
    make release-extension EXTENSION_NAME=<EXTENSION_NAME> VERSION=<NEW_VERSION>
    ```
    _Example:_ `make release-extension EXTENSION_NAME=identity VERSION=v0.1.1`
4.  **What it does:**
    - Updates the version in the extension's `Cargo.toml`.
    - Commits the version change.
    - Creates a Git tag formatted as `<EXTENSION_NAME>-v<NEW_VERSION>` (e.g., `identity-v0.1.1`).
    - Pushes the commit and the tag to the remote repository (`origin`).
    - The pushed tag triggers the `release-extension.yml` GitHub Actions workflow (assuming it exists), which builds the extension WASM and creates a corresponding GitHub Release.

**Important:** Ensure you have the necessary permissions to push tags to the repository.