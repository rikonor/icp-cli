# Development Guidelines

## Project Documentation

All project documentation is maintained in the `docs/` directory to keep the project organized. This includes:

- [Summary](SUMMARY.md)
- [Guidelines](GUIDELINES.md)
- [Workflow](WORKFLOW.md)

This document guidelines for work. Among other things, it defines how we'll organize our work across multiple projects and sessions and ensure continuity.

## Session Workflow

### Beginning of Session

1. **Task Assignment**: Unless one was already provided to you, start by asking the operator for your task assignment.
2. **Review Guidelines**: Review the [guidelines](GUIDELINES.md) and [summary](SUMMARY.md).
3. **Review Task**: Examine task's `TASK.md` file to understand requirements and current status.
4. **Review Handoffs**: Read the latest task handoff document to understand current progress.
5. **Verify Alignment**: Ensure work aligns with task requirements and project goals.
6. **Review Context**: Examine any code or documentation changes from previous work.
7. **Set Goals**: Establish specific objectives for the current session.

### During Session

1. **Incremental Changes**: Make changes in small, logical increments.
2. **Documentation**: Update documentation as work progresses.
3. **Testing**: Ensure new code is properly tested.
4. **Commit Regularly**: Commit changes after each significant milestone.

### End of Session

1. **Update Task Status**: Update TASK.md with current progress and status.
2. **Create Handoff**: Create numbered handoff document (HANDOFF-XXX.md) with current state and next steps.
3. **Update Project Status**: Update PROJECT.md with task progress if needed.
4. **Final Commit**: Commit all changes with a descriptive message. **This is a hard requirement** - no session should end without committing all changes.
5. **Next Steps**: Document requirements for continuing work in the next session.

## Documentation Standards

1. **Code Comments**: Use meaningful comments that explain why code exists, not just what it does.
2. **Function Documentation**: All public functions should have documentation comments explaining:
   - Purpose
   - Parameters
   - Return values
   - Examples (where appropriate)
3. **Module Documentation**: Each module should have a top-level doc comment explaining its purpose.
4. **Status Updates**: Keep PROJECT_STATUS.md up to date with accurate completion percentages.

## Testing Requirements

1. **Unit Tests**: Write unit tests for all new functions and methods.
2. **Integration Tests**: Test the interaction between extensions.
3. **Error Handling Tests**: Verify that error cases are handled properly.
4. **Backward Compatibility**: Test with existing extensions to ensure they continue to work.

## Commit Message Conventions

Follow this format for commit messages:

```
[Phase X][Component]: Brief description of the change

More detailed explanation if necessary

- Specific change 1
- Specific change 2

Related issues or tasks: #123
```

Examples:

- `[Phase 1][WIT]: Add library interface to world.wit`
- `[Phase 2][Manifest]: Update manifest model to track dependencies`

## Handoff Document Template

Create a file named `HANDOFF-XXX.md` (using sequential numbering) with the following format:

```markdown
# Task Handoff - HANDOFF-XXX

## Current State

Brief description of task progress and status.

## Completed Work

- Item 1: Description and outcome
- Item 2: Description and outcome

## Technical Details

- Decision 1: Rationale and implementation notes
- Decision 2: Rationale and implementation notes

## Challenges

- Challenge 1: How it was addressed
- Challenge 2: How it was addressed

## Next Steps

- Immediate next actions
- Remaining work items

## Notes

Additional context or observations for the next session.
```

## Code Modification Best Practices

1. **Prefer Targeted Changes**: When modifying existing files, use the `replace_in_file` tool with carefully crafted SEARCH/REPLACE blocks rather than overwriting the entire file with `write_to_file`.

2. **Retry Diff Operations**: If you encounter issues with the `replace_in_file` tool, it's preferable to retry with adjusted SEARCH blocks rather than immediately falling back to `write_to_file`. This is because diffs are much more efficient than full file rewrites.

3. **Reserve `write_to_file` for New Files**: Only use `write_to_file` when creating entirely new files or when the changes are so extensive that using `replace_in_file` would be impractical.

4. **Incremental Changes**: Make changes in small, logical increments that can be easily reviewed and understood.

5. **Use WebAssembly Tools**: The `wasm-tools` binary is installed and available for examining WebAssembly components. For example, use `wasm-tools component wit <PATH>` to examine the WIT definitions of components.

## Code Quality Expectations

1. **Error Handling**: All operations should have proper error handling with meaningful error messages.
2. **Performance**: Consider the performance implications of code, especially during extension loading.
3. **Naming**: Use clear, descriptive names for variables, functions, and modules.
4. **Code Structure**: Keep functions small and focused on a single responsibility.
5. **Magic Numbers/Strings**: Avoid hardcoding values; use constants with descriptive names.
6. **Consistency**: Follow the existing code style and patterns of the project.
7. **Readability**: Prioritize readability over clever or overly concise code.

## Improvement Process

At the end of each session, consider:

1. What worked well?
2. What didn't work well?
3. What could be improved in the workflow?
4. Are there any new guidelines that should be added based on experience?

Update this document with those improvements for the next session.

## Verification of Assumptions

1. **External APIs**: Verify assumptions about external APIs and libraries before documenting limitations or implementing workarounds.
2. **Documentation Accuracy**: Ensure that documentation accurately reflects the current state of the code and external dependencies.
3. **Previous Implementations**: When replacing or refactoring code, carefully examine previous implementations to understand their approach and any lessons learned.
4. **Test Failures**: When tests fail, investigate thoroughly to understand the root cause rather than making assumptions.
