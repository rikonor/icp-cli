#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

# Function to print error messages and exit
error_exit() {
    echo "Error: $1" >&2
    exit 1
}

# --- Argument Parsing ---
if [ -z "$1" ]; then
    error_exit "Extension NAME is required as the first argument."
fi
NAME=$1

if [ -z "$2" ]; then
    error_exit "VERSION is required as the second argument (e.g., 0.1.2)."
fi
VERSION=$2
TAG_NAME="${NAME}-v${VERSION}"
EXTENSION_CRATE_PATH="crates/extensions/${NAME}"
EXTENSION_CARGO_TOML="${EXTENSION_CRATE_PATH}/Cargo.toml"

echo "--- Preparing release for extension '${NAME}' version '${VERSION}' ---"

# --- Pre-release Checks ---
echo "1. Checking git status..."
if ! git diff --quiet HEAD --; then
    error_exit "Working directory is not clean. Please commit or stash changes."
fi
# Check for untracked files as well
if [ -n "$(git status --porcelain)" ]; then
    error_exit "Untracked files detected. Please commit or remove them."
fi
echo "   Git status is clean."

echo "2. Checking if tag '${TAG_NAME}' already exists..."
if git rev-parse "$TAG_NAME" >/dev/null 2>&1; then
    error_exit "Tag '${TAG_NAME}' already exists."
fi
echo "   Tag '${TAG_NAME}' does not exist."

echo "3. Checking if extension crate path '${EXTENSION_CRATE_PATH}' exists..."
if [ ! -d "$EXTENSION_CRATE_PATH" ]; then
    error_exit "Extension crate path '${EXTENSION_CRATE_PATH}' not found."
fi
echo "   Extension crate path found."

echo "4. Checking if extension Cargo.toml '${EXTENSION_CARGO_TOML}' exists..."
if [ ! -f "$EXTENSION_CARGO_TOML" ]; then
    error_exit "Extension Cargo.toml '${EXTENSION_CARGO_TOML}' not found."
fi
echo "   Extension Cargo.toml found."

# --- Version Update ---
echo "5. Updating version in '${EXTENSION_CARGO_TOML}' to '${VERSION}'..."
# Use sed for robustness as cargo-edit might not be installed.
# This assumes the version line looks like `version = "..."` at the start of a line.
if ! sed -i.bak "s/^version = \".*\"/version = \"${VERSION}\"/" "$EXTENSION_CARGO_TOML"; then
    error_exit "Failed to update version in ${EXTENSION_CARGO_TOML} using sed."
fi
rm "${EXTENSION_CARGO_TOML}.bak" # Remove backup file on success
echo "   Version updated successfully."

echo "6. Updating Cargo.lock..."
# Run cargo check to update the lock file based on Cargo.toml changes
# We check the specific package to ensure only relevant parts of Cargo.lock might change
cargo check --package ${NAME} --quiet || error_exit "cargo check failed for package ${NAME}, could not update Cargo.lock"
echo "   Cargo.lock updated."

# --- Git Operations ---
COMMIT_MSG="chore(release): bump ${NAME} extension to v${VERSION}"
echo "7. Committing changes..."
# Add both the extension's Cargo.toml and the potentially updated Cargo.lock
git add "$EXTENSION_CARGO_TOML" Cargo.lock
git commit -m "$COMMIT_MSG"
echo "   Committed with message: '${COMMIT_MSG}'"

echo "8. Creating tag '${TAG_NAME}'..."
git tag "$TAG_NAME"
echo "   Tag '${TAG_NAME}' created."

echo "9. Pushing commit and tag to remote 'origin'..."
# Push commit first
if ! git push origin HEAD; then
    error_exit "Failed to push commit. Please check remote connection and permissions."
fi
# Push tag
if ! git push origin "$TAG_NAME"; then
    # Attempt to clean up local tag if push fails
    git tag -d "$TAG_NAME" >/dev/null 2>&1
    error_exit "Failed to push tag '${TAG_NAME}'. Local tag removed. Please check remote connection and permissions."
fi
echo "   Commit and tag pushed successfully."

echo "--- Release process for extension '${NAME}' version '${VERSION}' completed. ---"
echo "Triggering the release-extension CI workflow..."

exit 0