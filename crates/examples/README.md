# ICP CLI Examples

This directory contains example crates demonstrating various features and usage patterns of the ICP CLI and its extensions.

## Purpose

Examples serve several purposes:

- Illustrate how to use core ICP CLI features.
- Showcase how to integrate and use specific extensions.
- Provide starting points for users building their own integrations or workflows.
- Serve as integration tests for the core components and extensions.

## Structure

Each example resides in its own subdirectory within `crates/examples/`. The name of the subdirectory should reflect the purpose or feature being demonstrated.

```
crates/examples/
├── <example_name_1>/
│   ├── Cargo.toml
│   └── src/
├── <example_name_2>/
│   ├── Cargo.toml
│   └── src/
└── ...
```

## Convention

- **Directory Name:** Choose a descriptive name for the example's directory (e.g., `minimal`, `using-multiply-extension`).
- **Workspace Inclusion:** This directory (`crates/examples/`) is included in the main workspace `Cargo.toml` using a wildcard (`crates/examples/*`). Ensure your new example crate builds correctly as part of the workspace.
- **Simplicity:** Examples should be as simple as possible while effectively demonstrating the intended concept.
- **Documentation:** Include comments in the example code and consider adding a small README within the example's directory if the setup or concept requires further explanation beyond the code itself.

## Adding a New Example

1.  Create a new directory: `crates/examples/<your_example_name>/`.
2.  Initialize a new Rust crate within that directory (e.g., using `cargo init`).
3.  Implement the example code.
4.  Ensure it builds correctly within the workspace (`cargo build -p <your_example_name>`).
