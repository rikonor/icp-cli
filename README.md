# icp

A command-line utility for Internet Computer that uses WebAssembly component model for extension support.

## Overview

icp is a CLI tool for the Internet Computer platform that leverages the WebAssembly Component Model for its extension system, enabling modular and extensible CLI functionality. It focuses on providing:

- WebAssembly component-based extension system
- Extension system with clear interfaces
- Async operation support

## Getting Started

### Quick Installation

#### Unix-like Systems (macOS, Linux)

```bash
curl -sL https://rikonor.github.io/icp-cli/install.sh | bash
```

#### Windows

```powershell
iwr -useb https://rikonor.github.io/icp-cli/install.ps1 | iex
```

Note: The installation URL will be automatically updated with the correct GitHub Pages URL in the CI process.

### Prerequisites for Building from Source

- Rust toolchain
- WebAssembly target support

### Building from Source

```bash
git clone [repository-url]
cargo build
```

## Extension System

icp supports WebAssembly components through a defined interface system (WIT). Extensions are isolated components that can be dynamically loaded and executed.

For detailed development guidelines and process documentation, see [Development Guidelines](docs/GUIDELINES.md).

## Development

Build the project:

```bash
cargo build
```

Run tests:

```bash
cargo test
```
