# Core Extraction Project

## Purpose

Split icp-cli into separate core library and CLI components to improve maintainability, testability, and potential reusability.

## Status

In Progress (45% Complete)

## Tasks

- [Complete] Setup and Initial Structure Task

  - Create icp-core crate, update workspace configuration, and set up initial module structure
  - Located in: `tasks/setup/`

- [In Progress] Core Interface Types Migration Task

  - Move core interface types to icp-core while maintaining all functionality
  - Current progress: 97%
  - Located in: `tasks/interface-migration/`

- [Not Started] Component and Extension Logic Migration Task

  - Move core component and extension handling to icp-core
  - Located in: `tasks/component-migration/`

- [Not Started] CLI Simplification Task

  - Refactor icp-cli to be a thin wrapper around icp-core
  - Located in: `tasks/cli-simplification/`

- [In Progress] Testing Infrastructure Task
  - Improve overall test coverage and testing tools
  - Current progress: 5%
  - Located in: `tasks/testing-infrastructure/`

## Project Overview

The Core Extraction project aims to split icp-cli into separate core library and CLI components to improve maintainability, testability, and potential reusability. The project is organized into five main tasks, each focusing on a specific aspect of the extraction process.

## Current Focus

The current focus is on completing the Core Interface Types Migration task, which is 97% complete, and advancing the Testing Infrastructure task, which is being worked on in parallel to ensure the quality of the migrated code.

## Next Steps

1. Complete the Core Interface Types Migration task:

   - Fix the identified issues with the WebAssembly Component Model templates:
     - Fix DUPLICATE_INTERFACE_TEMPLATE (invalid extern name issue)
     - Fix NESTED_INSTANCES_TEMPLATE (type mismatch issue)
     - Review and fix other potential issues in templates
   - Complete the comprehensive tests for interface detection
   - Finalize error handling for interface detection

2. Continue work on the Testing Infrastructure task:

   - Implement the WAT validation testing utility
   - Create test cases for various WAT scenarios
   - Add integration tests for the validation utility

3. Begin planning for the Component and Extension Logic Migration task:
   - Identify which components and extension logic need to be moved
   - Plan the migration strategy to minimize disruption
   - Create a detailed task list for implementation

## Dependencies

This project builds upon the completed Extension Inter-Communication project, leveraging the functionality implemented there for extension communication and dependency management.

## Known Issues

- WebAssembly Component Model template issues:
  - Identified specific issues in DUPLICATE_INTERFACE_TEMPLATE and NESTED_INSTANCES_TEMPLATE
  - Some functions have mismatches between their signatures and the canonical ABI requirements
  - See HANDOFF-005.md in the interface-migration task for detailed analysis

## Timeline

Each task is expected to take 1-2 working sessions, with the entire project completing in 5-10 sessions.
