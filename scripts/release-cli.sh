#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

# --- Configuration ---
GIT_REMOTE="origin" # Or configure as needed
PYTHON_HELPER="scripts/set_workspace_version.py" # Path to the Python helper

# --- Helper Functions ---
error_exit() {
    echo "Error: $1" >&2
    exit 1
}

# --- Argument Parsing ---
VERSION=$1
if [ -z "$VERSION" ]; then
    error_exit "Usage: $0 <VERSION>"
fi

# Basic SemVer check (can be made more robust)
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.-]+)?(\+[a-zA-Z0-9.-]+)?$ ]]; then
    error_exit "Version '$VERSION' does not look like Semantic Versioning (X.Y.Z)."
fi

TAG_NAME="v$VERSION"
COMMIT_MSG="chore(release): bump core version to $TAG_NAME"

echo "--- Starting Core CLI Release: $TAG_NAME ---"

# --- Pre-flight Checks ---
echo "1. Checking git status..."
# Check if git status --porcelain produces any output. If it does, the directory is not clean.
if [ -n "$(git status --porcelain)" ]; then
    error_exit "Working directory is not clean (uncommitted changes or untracked files). Please commit, stash, or remove changes."
fi
echo "   Git status clean."

echo "2. Checking if tag '$TAG_NAME' already exists..."
if git rev-parse "$TAG_NAME" >/dev/null 2>&1; then
    error_exit "Tag '$TAG_NAME' already exists."
fi
echo "   Tag does not exist."

echo "3. Checking for Python 3..."
if ! python3 --version > /dev/null 2>&1; then
    error_exit "'python3' command not found. Please ensure Python 3 is installed and in PATH."
fi
echo "   Python 3 found."

echo "4. Checking for Python helper script '$PYTHON_HELPER'..."
if [ ! -f "$PYTHON_HELPER" ]; then
    error_exit "Helper script '$PYTHON_HELPER' not found."
fi
if [ ! -x "$PYTHON_HELPER" ]; then
    echo "   Warning: Helper script '$PYTHON_HELPER' is not executable. Attempting to run with python3..."
    PYTHON_CMD="python3 $PYTHON_HELPER"
else
    PYTHON_CMD="$PYTHON_HELPER"
fi
echo "   Helper script found."

# --- Update Version ---
echo "5. Updating workspace version using $PYTHON_HELPER to $VERSION..."
if $PYTHON_CMD "$VERSION"; then
    echo "   Successfully updated workspace version via Python script."
else
    # Python script should output specific errors to stderr
    error_exit "Failed to update workspace version using '$PYTHON_HELPER'."
fi

# --- Git Operations ---
echo "6. Committing version bump..."
# The Python script only modifies Cargo.toml. Cargo.lock might need updating separately if desired/needed.
# For now, only committing Cargo.toml based on the script's action.
git add Cargo.toml
git commit -m "$COMMIT_MSG"
echo "   Committed."

echo "7. Creating tag '$TAG_NAME'..."
git tag "$TAG_NAME"
echo "   Tagged."

echo "8. Pushing commit and tag to $GIT_REMOTE..."
# Use --atomic for safer push if supported and desired
git push "$GIT_REMOTE" HEAD
git push "$GIT_REMOTE" "$TAG_NAME"
echo "   Pushed."

echo "--- Core CLI Release $TAG_NAME Complete ---"
exit 0