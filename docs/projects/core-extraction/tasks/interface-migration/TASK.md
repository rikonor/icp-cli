# Core Interface Types Migration Task

## Overview

Move core interface types to dfx-core while maintaining all functionality, ensuring that the interface detection and handling capabilities are properly extracted from dfx-cli.

## Scope

- Move Interface/ComponentInterfaces to dfx-core
- Move IfaceDetector trait and implementation
- Update dfx-cli to use these from dfx-core
- Update integration tests for interface detection
- Implement actual interface detection in dfx-core
- Split the Explainer.md into smaller more focused files
- Fix WAT templates in test-utils with proper component model syntax
- Add comprehensive tests for interface detection
- Refine error handling for interface detection

## Status

- Current Phase: In Progress
- Progress: 97%
- Last Updated: 2025-03-17

## Implementation Details

### Interface Types Migration

- Moved Interface/ComponentInterfaces to dfx-core
- Moved IfaceDetector trait and implementation
- Updated dfx-cli to use these from dfx-core
- Updated integration tests for interface detection
- Implemented actual interface detection in dfx-core

### Documentation Improvements

- Split the Explainer.md into smaller more focused files for easier reference
- Added comprehensive documentation to the interface module
- Enhanced error handling with specific error types for interface detection

### Testing Infrastructure

- Fixed WAT templates in test-utils with proper component model syntax
- Added more test templates for edge cases:
  - Empty component
  - Many interfaces
  - Nested instances
  - Duplicate interfaces
- Fixed WebAssembly Component Model template issues:
  - Fixed parameter type mismatch in string handling functions
  - Fixed return type mismatch in string return functions
  - Fixed boolean conversion for boolean return values
  - Fixed duplicate interface naming issues
  - Fixed log function return type in nested instances template
- Added detailed comments to all WAT templates explaining:
  - Purpose of each template
  - Structure and key components
  - Key WebAssembly Component Model features demonstrated
  - Common issues and pitfalls
  - Test expectations

## Dependencies

- Setup and Initial Structure Task: Required for the dfx-core crate to exist

## Technical Challenges and Solutions

### WebAssembly Component Model Template Issues

Encountered and fixed various issues with the WebAssembly Component Model templates:

- Parameter type mismatches in string handling functions
- Return type mismatches in string return functions
- Boolean conversion issues for boolean return values
- Duplicate interface naming issues
- Log function return type issues in nested instances template

Some functions still have mismatches between their signatures and the canonical ABI requirements, which need to be identified and fixed.

### Documentation Organization

Split the large Explainer.md into smaller, more focused files for easier reference, improving the overall documentation structure.

## Success Criteria

- All existing interface detection functionality works
- New integration tests pass
- No duplication of interface types
- Extension system still works with interface detection

## Remaining Work

- Fix the remaining issues with the WebAssembly Component Model templates
- Complete the comprehensive tests for interface detection
- Finalize error handling for interface detection

## Notes

This task is part of the Core Extraction project, which aims to split dfx-cli into separate core library and CLI components to improve maintainability, testability, and potential reusability.
