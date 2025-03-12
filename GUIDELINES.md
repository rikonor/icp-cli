# Development Guidelines

This document outlines the working process for implementing the extension inter-communication feature for the dfx-2 project. It defines how we'll organize our work across multiple sessions and ensure continuity.

## Session Workflow

### Beginning of Session

1. **Review Guidelines**: Start by reviewing this GUIDELINES.md file to understand the process.
2. **Review Plan**: Examine PLAN.md to understand the current requirements of the project.
3. **Review Project Status**: Examine PROJECT_STATUS.md to understand the current state of the project.
4. **Review Handoff Document**: Read the latest handoff document from the previous session.
5. **Review Alignemt with Goal**: Ensure our work thus far has not strayed too far from our plan.
6. **Review Code Changes**: Examine any code changes made in previous sessions to get context.
7. **Establish Session Goals**: Identify specific tasks to be completed during the current session.

### During Session

1. **Incremental Changes**: Make changes in small, logical increments.
2. **Documentation**: Update documentation as code is changed.
3. **Testing**: Ensure new code is properly tested.
4. **Commit Regularly**: Commit changes after each significant milestone.

### End of Session

1. **Update PROJECT_STATUS.md**: Update the status of tasks and overall project progress.
2. **Update GUIDELINES.md**: Make suggestions to improve workflow and update accordingly.
3. **Create Handoff Document**: Document the current state, what was accomplished, and next steps.
4. **Final Commit**: Commit all changes with a descriptive message. **This is a hard requirement** - no session should end without committing all changes to ensure continuity between sessions.
5. **Provide Next Session Instructions**: Clearly communicate what the user should provide at the beginning of the next session to help resume work effectively (e.g., "Please provide X at the beginning of the next session so I can resume work").

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

Create a file named `HANDOFF-SESSION-X.md` at the end of each session with the following format:

```markdown
# Session X Handoff Document

## Completed in this Session

- Task 1: Brief description and outcome
- Task 2: Brief description and outcome

## Current State

Description of the current state of the project and any work in progress.

## Technical Decisions Made

- Decision 1: Rationale
- Decision 2: Rationale

## Challenges and Solutions

- Challenge 1: How it was addressed or workaround
- Challenge 2: How it was addressed or workaround

## Next Steps

- Immediate next task
- Following tasks

## Additional Notes

Any other relevant information or observations that might be helpful for the next session.
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
