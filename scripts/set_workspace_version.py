#!/usr/bin/env python3

import re
import sys
import os

# --- Configuration ---
CARGO_TOML_PATH = "Cargo.toml"

# --- Check for TOML library ---
try:
    import toml
except ImportError:
    print("Error: 'toml' library not found. Please install it: pip install toml", file=sys.stderr)
    sys.exit(1)

# --- Argument Parsing ---
if len(sys.argv) != 2:
    print(f"Usage: {sys.argv[0]} <NEW_VERSION>", file=sys.stderr)
    sys.exit(1)

new_version = sys.argv[1]

# Basic SemVer check (can be made more robust)
if not re.match(r"^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.-]+)?(\+[a-zA-Z0-9.-]+)?$", new_version):
    print(
        f"Error: Version '{new_version}' does not look like Semantic Versioning (X.Y.Z).", file=sys.stderr)
    sys.exit(1)


# --- Read and Update TOML ---
try:
    if not os.path.exists(CARGO_TOML_PATH):
        print(
            f"Error: '{CARGO_TOML_PATH}' not found in the current directory.", file=sys.stderr)
        sys.exit(1)

    with open(CARGO_TOML_PATH, 'r') as f:
        data = toml.load(f)

    if 'workspace' not in data or 'package' not in data['workspace']:
        print(
            f"Error: '[workspace.package]' section not found in '{CARGO_TOML_PATH}'.", file=sys.stderr)
        sys.exit(1)

    if 'version' not in data['workspace']['package']:
        print(
            f"Error: 'version' key not found under '[workspace.package]' in '{CARGO_TOML_PATH}'.", file=sys.stderr)
        sys.exit(1)

    current_version = data['workspace']['package']['version']
    print(f"Current workspace version: {current_version}")

    data['workspace']['package']['version'] = new_version
    print(f"Updating workspace version to: {new_version}")

    with open(CARGO_TOML_PATH, 'w') as f:
        toml.dump(data, f)

    print(f"Successfully updated '{CARGO_TOML_PATH}'.")
    sys.exit(0)

except Exception as e:
    print(f"An error occurred: {e}", file=sys.stderr)
    sys.exit(1)
