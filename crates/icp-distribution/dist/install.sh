#!/bin/bash
# ICP CLI Installation Script for Unix-like systems (macOS, Linux)
# This script downloads and installs the ICP CLI binary.
# Version: 0.1.0

set -euo pipefail

# Configuration
INSTALL_DIR="/usr/local/bin"
BINARY_NAME="icp"
PLATFORM="$(uname | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"
BINARY_URL="https://get.icp-cli.com/binaries/icp/${PLATFORM}-${ARCH}"
CHECKSUM_URL="https://get.icp-cli.com/binaries/icp/${PLATFORM}-${ARCH}.sha256"

# Helper Functions
status() {
    echo "==> $1" >&2
}

error() {
    echo "ERROR: $1" >&2
    exit 1
}

# Check for required commands
for cmd in curl shasum chmod; do
    if ! command -v $cmd >/dev/null 2>&1; then
        error "Required command '$cmd' not found"
    fi
done

# Main Installation Process
status "Starting ICP CLI 0.1.0 installation..."

# Check if running as root
if [ "$EUID" -eq 0 ]; then
    error "Please do not run this script as root or with sudo"
fi

# Check write permissions
if [ ! -w "$INSTALL_DIR" ]; then
    error "Cannot write to $INSTALL_DIR. Please ensure you have write permissions"
fi

# Download binary
status "Downloading ICP CLI binary from $BINARY_URL"
if ! curl -sSL "$BINARY_URL" -o "$INSTALL_DIR/$BINARY_NAME"; then
    error "Failed to download binary"
fi

# Verify checksum
status "Verifying binary checksum"
EXPECTED_CHECKSUM=$(curl -sSL "$CHECKSUM_URL" || error "Failed to download checksum")
ACTUAL_CHECKSUM=$(shasum -a 256 "$INSTALL_DIR/$BINARY_NAME" | awk '{print $1}')

if [ "$ACTUAL_CHECKSUM" != "$EXPECTED_CHECKSUM" ]; then
    rm -f "$INSTALL_DIR/$BINARY_NAME"
    error "Checksum verification failed! Expected: $EXPECTED_CHECKSUM, Got: $ACTUAL_CHECKSUM"
fi

# Make binary executable
status "Making binary executable"
chmod +x "$INSTALL_DIR/$BINARY_NAME"

# Update PATH in shell configuration files
status "Updating PATH configuration"
for shell_config in ~/.bashrc ~/.zshrc; do
    if [ -f "$shell_config" ]; then
        if ! grep -q "$INSTALL_DIR" "$shell_config"; then
            echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$shell_config"
            status "Updated $shell_config"
        else
            status "$shell_config already contains $INSTALL_DIR in PATH"
        fi
    fi
done

status "Installation complete! You may need to restart your terminal for PATH changes to take effect."
status "Try running 'icp --version' to verify the installation."
