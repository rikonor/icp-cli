# Core Extraction Project

## Overview

Split dfx-cli into separate core library and CLI components to improve maintainability, testability, and potential reusability.

## Project Stages

### Stage 1: Setup and Initial Structure

**Goal**: Create new dfx-core crate with minimal structure without breaking existing functionality.

**Tasks**:

1. Create new dfx-core crate
2. Update workspace Cargo.toml
3. Create initial module structure
4. Set up minimal public API

**Success Criteria**:

- `cargo build` succeeds for all crates
- `cargo test` passes for existing tests
- dfx-core crate has version and documentation
- dfx-cli binary still functions normally

### Stage 2: Core Interface Types Migration

**Goal**: Move core interface types to dfx-core while maintaining all functionality.

**Tasks**:

1. Move Interface/ComponentInterfaces to dfx-core
2. Move IfaceDetector trait and implementation
3. Update dfx-cli to use these from dfx-core
4. Add integration tests for interface detection

**Success Criteria**:

- All existing interface detection functionality works
- New integration tests pass
- No duplication of interface types
- Extension system still works with interface detection

### Stage 3: Component and Extension Logic Migration

**Goal**: Move core component and extension handling to dfx-core.

**Tasks**:

1. Move component handling logic
2. Move extension management
3. Move dependency graph logic
4. Create proper abstraction layers

**Success Criteria**:

- Extension management commands work
- Dependency resolution works
- Component instantiation works
- Added unit tests for each migrated module
- Integration tests verify extension lifecycle

### Stage 4: CLI Simplification

**Goal**: Refactor dfx-cli to be a thin wrapper around dfx-core.

**Tasks**:

1. Refactor command handling
2. Update main.rs to use dfx-core APIs
3. Clean up CLI-specific code
4. Improve error handling

**Success Criteria**:

- All CLI commands work as before
- Reduced code size in dfx-cli
- Clear separation between CLI and core logic
- End-to-end tests pass

### Stage 5: Testing Infrastructure

**Goal**: Improve overall test coverage and testing tools.

**Tasks**:

1. Update test-utils to use dfx-core
2. Add more comprehensive integration tests
3. Add benchmarks for core operations
4. Add documentation tests

**Success Criteria**:

- Test coverage above 80% for dfx-core
- All test utilities use dfx-core APIs
- Documentation includes examples
- Performance benchmarks established

## Risk Mitigation

- Each stage has clear rollback points
- Stages are designed to preserve functionality
- Tests must pass before moving to next stage
- Documentation is updated with each change

## Timeline

Each stage is expected to take 1-2 working sessions, with the entire project completing in 5-10 sessions.
