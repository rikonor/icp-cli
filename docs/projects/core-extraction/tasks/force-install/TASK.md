# Force Install Task

## Purpose

Add --force flag to overwrite existing extensions during installation

## Subtasks

1. [x] CLI Argument Handling

   - Added --force flag to command parser
   - Success: Flag appears in help output and is parseable
   - Next: Core Logic Implementation

2. [x] Core Logic Implementation

   - Modified extension installation to overwrite existing when --force specified
   - Dependencies: CLI Argument Handling
   - Success: Force install verified via manual testing
   - Next: Testing Infrastructure

3. [ ] Testing Infrastructure

   - Add tests for force install scenarios
   - Dependencies: Core Logic Implementation
   - Success: All tests pass in CI
   - Next: Documentation

4. [ ] Documentation
   - Update man pages and help text
   - Add to migration guide
   - Dependencies: Testing Infrastructure
   - Success: Docs updated and reviewed

## Technical Approach

- Modify `clap` configuration in CLI entrypoint
- Update `icp-core` installation logic to handle overwrites
- Add integration tests verifying force behavior
- Document flag usage in appropriate help sections

## Dependencies

- Requires completed CLI simplification task (ARG-004)
- Relies on extension registry from component-migration

## Success Criteria

- Users can overwrite existing extensions with --force
- Fails gracefully when force not specified and extension exists
- Full test coverage of force scenarios
- Documentation reflects new functionality
