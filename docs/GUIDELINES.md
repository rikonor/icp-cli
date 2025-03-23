# Development Guidelines

Welcome to the icp project! This document provides essential guidance for working on the project effectively.

## Project Overview

icp is a command-line utility for the Internet Computer platform with WebAssembly extension support. Our development process is organized around discrete tasks within projects, enabling clear ownership and effective collaboration.

## Project Organization

All development work is organized in the `docs/projects/` directory using the following structure:

```
docs/projects/
└── project-name/
    ├── PROJECT.md         # Project overview and task listing
    └── tasks/            # Task-specific documentation
        └── task-name/
            ├── TASK.md   # Task details and requirements
            └── HANDOFF/  # Task-specific handoff notes
```

This structure enables:

- Clear task ownership and tracking
- Discrete units of work
- Sequential handoff documentation
- Progress monitoring through completed subtasks

### Task Structure

Tasks are organized as sequences of dependent subtasks that must be completed in order. This structure reflects how AI assistants process and execute work:

1. Each subtask builds on previous subtasks' completion
2. Scope expansion happens by adding subtasks in the appropriate sequence
3. Progress is tracked by completed subtasks rather than percentages
4. Dependencies between subtasks are explicitly documented

Example Task Structure:

```markdown
## Subtasks

1. [ ] Initial Setup

   - Success: Repository structure ready
   - Next: Component Implementation

2. [ ] Component Implementation

   - Dependencies: Initial Setup
   - Success: Core functionality working
   - Next: Integration

3. [ ] Integration

   - Dependencies: Component Implementation
   - Success: System fully integrated
   - Next: Testing

4. [ ] Testing
   - Dependencies: Integration
   - Success: All tests passing
   - Next: Documentation
```

Each subtask includes:

- Clear success criteria
- Dependencies on previous subtasks
- Next steps or handoff points
- Checkbox for completion tracking

## Working Process

### Beginning of Session

1. **Task Assignment**: Unless already provided, ask the operator for your task assignment.
2. **Review Task**: Examine task's `TASK.md` file to understand requirements and current status.
3. **Review Handoffs**: Read the latest task handoff document to understand current progress.
4. **Review Context**: Examine any code or documentation changes from previous work.
5. **Set Goals**: Establish specific objectives for the current session.

### During Session

1. **Incremental Changes**: Make changes in small, logical increments.
2. **Documentation**: Update documentation as work progresses.
3. **Testing**: Ensure new code is properly tested.
4. **Commit Regularly**: Commit changes after each significant milestone.

### End of Session

1. **Update Task Status**: Update TASK.md with current progress and status.
2. **Create Handoff**: Create numbered handoff document (HANDOFF-XXX.md) with current state and next steps.
3. **Update Project Status**: Update PROJECT.md with task progress if needed.
4. **Final Commit**: Commit all changes with a descriptive message. **This is a hard requirement**.
5. **Next Steps**: Document requirements for continuing work in the next session.

## Task Lifecycle

1. **Creation**

   - Create task directory under project's tasks/
   - Write initial TASK.md with subtask sequence
   - Create HANDOFF directory
   - Define initial success criteria for each subtask

2. **Development**

   - Complete subtasks in sequence
   - Document decisions and changes
   - Create numbered handoff documents
   - Update subtask completion status
   - Add new subtasks as scope expands

3. **Completion**
   - Verify all subtasks are complete
   - Create final handoff document
   - Update PROJECT.md task status

## Documentation Standards

### Handoff Document Template

Create sequentially numbered handoff documents (HANDOFF-XXX.md) with:

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

## Guidelines Audit

Guidelines followed in this session:

- [Guideline Category] Specific guideline followed
  - How it was applied
  - Impact on the work

Guidelines that could have been better applied:

- [Guideline Category] Specific guideline
  - Why it was challenging
  - Suggested improvements

New guideline suggestions:

- Any patterns or practices that emerged that could become guidelines
```

### Code Documentation

1. **Code Comments**: Explain why code exists, not just what it does.
2. **Function Documentation**: Include for all public functions:
   - Purpose
   - Parameters
   - Return values
   - Examples (where appropriate)
3. **Module Documentation**: Add top-level doc comments explaining module purpose.

## Code Quality Standards

1. **Error Handling**: Use proper error handling with meaningful messages.
2. **Performance**: Consider performance implications of code.
3. **Naming**: Use clear, descriptive names.
4. **Code Structure**: Keep functions small and focused.
5. **Magic Numbers/Strings**: Use named constants.
6. **Consistency**: Follow project code style.
7. **Readability**: Prioritize clarity over cleverness.

## Testing Requirements

1. **Unit Tests**: Write for all new functions and methods.
2. **Integration Tests**: Test interaction between extensions.
3. **Error Handling**: Verify proper error case handling.
4. **Backward Compatibility**: Test with existing extensions.

## Best Practices

### Code Modification

1. **Prefer Targeted Changes**: Use `replace_in_file` with carefully crafted SEARCH/REPLACE blocks.
2. **Retry Diff Operations**: Adjust SEARCH blocks before falling back to full rewrites.
3. **Write New Files**: Use `write_to_file` for new files or major rewrites.
4. **Incremental Changes**: Make changes in small, reviewable increments.

### Commit Messages

Follow this format:

```
[Phase X][Component]: Brief description

More detailed explanation if necessary

- Specific change 1
- Specific change 2

Task: task-name
```

### Technical Tools

- Use `wasm-tools` for examining WebAssembly components
- Example: `wasm-tools component wit <PATH>` for WIT definitions

## Quality Assurance

### Verification Steps

1. **External APIs**: Verify assumptions about external dependencies.
2. **Documentation**: Ensure accuracy of documentation.
3. **Previous Work**: Review existing implementations.
4. **Test Failures**: Investigate root causes thoroughly.

### Improvement Process

At the end of each session:

1. Note what worked well
2. Identify areas for improvement
3. Suggest workflow enhancements
4. Update guidelines as needed

Remember: The goal is to maintain high quality while enabling efficient collaboration through clear documentation and communication.

## Project Creation Best Practices

When creating a new project:

- Review 1-2 existing projects in docs/projects/ to understand the expected level of detail and structure
- PROJECT.md should focus on high-level objectives, task breakdown, and dependencies
- TASK.md files should be concise, focusing on clear objectives, scope, and success criteria
- Track progress through ordered subtasks with clear success criteria
- Avoid excessive detail in initial task documentation - let the implementation details emerge through handoffs
