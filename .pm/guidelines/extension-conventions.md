---
metadata:
  description: Summarizes conventions for creating ICP CLI extensions and points to the detailed README.
---

# ICP CLI Extension Conventions (Summary)

**Purpose:** To provide a brief overview of the standard conventions for creating new ICP CLI extensions, ensuring consistency across the project.

This guideline summarizes the basic convention for creating new ICP CLI extensions.

## Core Convention

- Extensions are self-contained Rust crates implementing a specific interface defined by WIT.
- Each extension resides in its own directory within `crates/extensions/`.
- The directory name **must** match the crate name (e.g., `crates/extensions/my-extension/` for a crate named `my-extension`).
- The `crates/extensions/*` path is included in the workspace `members` list in the root `Cargo.toml`.

## Detailed Instructions

For detailed instructions on structure (including WIT usage), patterns, required files (`wit/world.wit`, `src/lib.rs`, etc.), and step-by-step guidance on adding a new extension, please refer to the primary documentation located at:

**[`crates/extensions/README.md`](./crates/extensions/README.md)**

Refer to the `minimal` example (`crates/examples/minimal/`) as a minimal working example of an extension's structure.
