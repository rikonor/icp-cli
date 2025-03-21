# Core Interface Types Migration Task

## Overview

Move core interface types to icp-core while maintaining all functionality, ensuring that the interface detection and handling capabilities are properly extracted from icp-cli.

## Scope

- Move Interface/ComponentInterfaces to icp-core
- Move IfaceDetector trait and implementation
- Update icp-cli to use these from icp-core
- Update integration tests for interface detection
- Implement actual interface detection in icp-core
- Split the Explainer.md into smaller more focused files
- Fix WAT templates in test-utils with proper component model syntax
- Add comprehensive tests for interface detection
- Refine error handling for interface detection

## Status

- Current Phase: In Progress
- Progress: 97%
- Last Updated: 2025-03-18

## Implementation Details

### Interface Types Migration

- Moved Interface/ComponentInterfaces to icp-core
- Moved IfaceDetector trait and implementation
- Updated icp-cli to use these from icp-core
- Updated integration tests for interface detection
- Implemented actual interface detection in icp-core

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

- Setup and Initial Structure Task: Required for the icp-core crate to exist

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

- Fix the identified issues with the WebAssembly Component Model templates:
  - Fix DUPLICATE_INTERFACE_TEMPLATE (invalid extern name issue)
  - Fix NESTED_INSTANCES_TEMPLATE (type mismatch issue)
  - Review and fix other potential issues in templates
- Complete the comprehensive tests for interface detection
- Finalize error handling for interface detection

See HANDOFF-005.md for detailed analysis and implementation plan.

## Notes

This task is part of the Core Extraction project, which aims to split icp-cli into separate core library and CLI components to improve maintainability, testability, and potential reusability.
