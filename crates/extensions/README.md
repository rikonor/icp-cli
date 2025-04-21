# ICP CLI Extensions

This directory contains extensions for the ICP CLI. Extensions are self-contained crates that provide additional functionality, often interacting with external systems or implementing specific logic.

## Structure

Each extension resides in its own subdirectory within `crates/extensions/`. The name of the subdirectory should match the crate name.

```
crates/extensions/
├── <extension_name_1>/
│   ├── Cargo.toml
│   ├── src/
│   └── wit/
├── <extension_name_2>/
│   ├── Cargo.toml
│   ├── src/
│   └── wit/
└── ...
```

## Convention

- **Directory Name:** The directory name must match the crate name defined in the extension's `Cargo.toml`.
- **Workspace Inclusion:** This directory (`crates/extensions/`) is included in the main workspace `Cargo.toml` using a wildcard (`crates/extensions/*`). Ensure your new extension crate builds correctly as part of the workspace.
- **Interface:** Extensions typically define their interface using WIT in a `wit/` subdirectory and generate bindings accordingly. Refer to existing extensions (e.g., `multiply`, `square`) for the current pattern.

## Adding a New Extension

1.  Create a new directory: `crates/extensions/<your_extension_name>/`.
2.  Initialize a new Rust crate within that directory (e.g., using `cargo init --lib`).
3.  Define your extension's logic and WIT interface (if applicable).
4.  Ensure it builds correctly within the workspace (`cargo build -p <your_extension_name>`).
5.  Refer to the `extension-conventions` project guideline for more details (`pm guideline show extension-conventions`).
