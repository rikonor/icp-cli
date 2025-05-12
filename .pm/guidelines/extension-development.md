---
metadata:
  description: Comprehensive guide for creating, building, and testing ICP CLI extensions.
---

# ICP CLI Extension Development

**Purpose:** To provide a complete guide for developing extensions for the ICP CLI, covering conventions, building, and testing.

## Extension Conventions

### Core Structure

- Extensions are self-contained Rust crates implementing a specific interface defined by WIT.
- Each extension resides in its own directory within `crates/extensions/`.
- The directory name **must** match the crate name (e.g., `crates/extensions/my-extension/` for a crate named `my-extension`).
- The `crates/extensions/*` path is included in the workspace `members` list in the root `Cargo.toml`.

### Required Files

- `Cargo.toml`: Defines the crate and its dependencies
- `src/lib.rs`: Contains the extension's implementation
- WIT definition file in the root `wit/extensions/` directory (e.g., `wit/extensions/<extension_name>.wit`)

## Building Extensions

Extensions can be built using the Makefile in the root directory of the codebase. The Makefile provides a convenient way to compile an extension into a WebAssembly component that can be loaded by the CLI.

### Build Process

1. Navigate to the root directory of the codebase.
2. Use the `make` command with the `EXTENSION_NAME` parameter to build a specific extension:

```bash
make EXTENSION_NAME=<extension_name>
```

For example, to build the "multiply" extension:

```bash
make EXTENSION_NAME=multiply
```

3. The command will output the path to the compiled WebAssembly component file (`.component.wasm`).

## Testing Extensions

After building an extension, you can test it by adding it to the CLI and then using it.

### Testing Process

1. Add the extension to the CLI using the `icp-cli extension add` command:

```bash
icp-cli extension add --name <extension_name> <path_to_component_wasm>
```

For example, using the output path from the build step:

```bash
icp-cli extension add --name multiply /path/to/target/wasm32-unknown-unknown/debug/multiply.component.wasm
```

2. Verify that the extension was added successfully:

```bash
icp-cli extension ls
```

3. Test the extension by using its commands:

```bash
icp-cli <extension_name> <subcommand> [arguments]
```

For example:

```bash
icp-cli multiply 5 10
```

## Removing Test Extensions

After testing, you may want to remove the extension:

```bash
icp-cli extension rm <extension_name>
```

## Debugging Extensions

If an extension doesn't work as expected:

1. Check the extension's implementation in `crates/extensions/<extension_name>/src/`.
2. Verify that the WIT interface in `wit/extensions/<extension_name>.wit` is correctly defined.
3. Ensure that any dependencies are correctly specified and available.
4. Rebuild the extension and try again.

## Example Workflow

```bash
# Build the extension
make EXTENSION_NAME=multiply

# Add the extension to the CLI (using the path from the build output)
icp-cli extension add --name multiply /path/to/target/wasm32-unknown-unknown/debug/multiply.component.wasm

# Test the extension
icp-cli multiply 5 10

# Remove the extension when done
icp-cli extension rm multiply
```

## Detailed Documentation

For more detailed instructions on extension development, including advanced patterns and examples, please refer to the primary documentation located at:

**[`crates/extensions/README.md`](./crates/extensions/README.md)**

You can also refer to the `minimal` example (`crates/examples/minimal/`) as a minimal working example of an extension's structure.
