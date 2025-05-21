---
metadata:
  description: Availability of local WIT specification documentation
---

# Guideline: WIT Specification Documentation Availability

**Purpose:** To inform AI assistants working on this project that the official WebAssembly Interface Type (WIT) specification documentation is available locally within the project's `docs` directory.

**Guideline:**

- The official WIT specification documentation, sourced from `https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md`, is located in the `docs/wit-spec/` directory.
- This documentation is organized into smaller markdown files with a main [README.md](docs/wit-spec/README.md) serving as a table of contents.
- AI assistants can access and read these files using the `read_file` tool (e.g., `<read_file><path>docs/wit-spec/README.md</path></read_file>`) to gain detailed understanding of the WIT format, its structure, types, and concepts when needed for tasks.
- Referencing this local documentation is preferred over searching external sources for the WIT specification.

**Goal:** Ensure AI assistants are aware of and utilize the readily available, official WIT documentation for accurate and efficient task completion related to WIT.