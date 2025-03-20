# Crate Organization Task

## Purpose

Establish a clear and organized directory structure for extensions and examples in the icp codebase, creating a foundation for future extension development and integration.

## Status

In Progress (0%)

## Requirements

1. **Design a Directory Structure**

   - Create a logical organization for extensions and examples
   - Ensure scalability for future additions
   - Maintain clear separation of concerns

2. **Implement Basic Structure**

   - Set up the directory skeleton
   - Create necessary configuration files
   - Update workspace configuration

3. **Document Conventions**

   - Create README files explaining directory purposes
   - Document extension structure and conventions
   - Provide guidelines for adding new extensions

4. **Migration Plan**
   - Outline how to migrate existing code (like hello-world) to the new structure
   - Ensure backward compatibility

## Acceptance Criteria

1. Directory structure is created and documented
2. Workspace configuration is updated to include new directories
3. README files are created for each major directory
4. Documentation clearly explains how to add new extensions
5. Migration path for existing code is documented

## Technical Details

### Proposed Directory Structure

```
crates/
├── icp-cli/           # Main CLI application (existing)
├── icp-core/          # Core library functionality (existing)
├── test-utils/        # Testing utilities (existing)
├── extensions/        # New directory for extensions
│   ├── common/        # Common utilities shared across extensions
│   ├── built-in/      # Extensions that ship with icp by default
│   │   ├── extension1/
│   │   ├── extension2/
│   │   └── ...
│   └── registry.rs    # Registry of built-in extensions
├── examples/          # New directory for example extensions
│   ├── minimal/       # Minimal example extension
│   ├── advanced/      # More complex example extension
│   ├── template/      # Template for creating new extensions
│   └── README.md      # Documentation for examples
└── tools/             # New directory for development tools
    ├── extension-dev/ # Tools for extension development
    └── templates/     # Templates for creating new extensions
```

### Implementation Considerations

1. **Extension Registration**: Consider how extensions will be registered with icp-cli. The `extensions/registry.rs` file could handle this.

2. **Dependency Management**: Ensure that extensions can depend on common utilities without creating circular dependencies.

3. **Build System Integration**: Update the build system to handle the new directory structure.

4. **Documentation**: Create clear documentation on how to add new extensions to the structure.

5. **Migration Strategy**: The existing `hello-world` crate is temporary and should eventually be migrated to the new structure, likely as `examples/minimal`.

## Next Steps

1. Create the directory structure
2. Set up basic Cargo.toml files for each new directory
3. Update workspace configuration
4. Create README files
5. Document extension structure and conventions
