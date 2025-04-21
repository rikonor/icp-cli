---
metadata:
  description: Explains the purpose and key features of the ICP CLI codebase.
---

# ICP CLI Codebase Purpose

**Purpose:** To provide a clear understanding of what the ICP CLI codebase is and its key features.

## Codebase Overview

The ICP CLI (Internet Computer Protocol Command Line Interface) is a command-line utility for the Internet Computer platform that leverages the WebAssembly Component Model for its extension system, enabling modular and extensible CLI functionality.

## Key Features

- **WebAssembly Component-Based Extension System**: The CLI uses WebAssembly components as extensions, allowing for isolated, secure, and dynamically loadable functionality.
- **Clear Interface Definitions**: Extensions interact with the CLI through well-defined interfaces using the WebAssembly Interface Type (WIT) system.
- **Async Operation Support**: The CLI supports asynchronous operations, enabling non-blocking interactions with the Internet Computer and other services.
- **Dependency Management**: Extensions can depend on other extensions, with automatic dependency resolution and validation.

## Architecture

The codebase is organized into several crates:

- `icp-cli`: The main CLI application
- `icp-core`: Core functionality shared across the codebase
- `icp-distribution`: Tools for distributing the CLI
- `crates/extensions/`: Directory containing individual extensions
- `crates/examples/`: Example implementations and usage patterns

## Extension System

Extensions are self-contained WebAssembly components that can be dynamically loaded and executed by the CLI. Each extension:

- Resides in its own directory within `crates/extensions/`
- Defines its interface using WIT
- Can import functionality from other extensions
- Can export functionality for use by other extensions or the CLI itself
