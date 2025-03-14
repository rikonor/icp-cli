# Development Workflow

## Project Organization

The development is organized into discrete projects, each in its own directory under `docs/projects/`. This structure allows us to:

- Track each project's progress independently
- Maintain project-specific documentation
- Handle project handoffs separately
- Better track completion and dependencies between projects

### Directory Structure

```
docs/
├── GUIDELINES.md     # Foundation for all development sessions
├── WORKFLOW.md       # This file - explains project organization
└── projects/        # All project-specific documentation
    ├── project-1/
    │   ├── PLAN.md             # Project goals and implementation plan
    │   ├── PROJECT_STATUS.md   # Current status and progress
    │   ├── COMPLETION.md       # Added when project is finished
    │   └── HANDOFF/           # Project-specific handoff notes
    └── project-2/
        └── ...
```

## Session Structure

1. Each session begins by reviewing:

   - GUIDELINES.md for development principles
   - Current project's documentation

2. Project Assignment:

   - Each session focuses on a specific project
   - Project is identified at session start
   - Progress tracked in project's PROJECT_STATUS.md

3. Documentation Updates:
   - Update PROJECT_STATUS.md during session
   - Create handoff notes in project's HANDOFF directory
   - Update COMPLETION.md when project finished

## Project Lifecycle

1. **Planning**

   - Create project directory under docs/projects/
   - Write initial PLAN.md
   - Create PROJECT_STATUS.md

2. **Development**

   - Track progress in PROJECT_STATUS.md
   - Document decisions and changes
   - Maintain handoff notes

3. **Completion**
   - Create COMPLETION.md
   - Document final state
   - Note any items deferred to future projects

## Current Projects

1. **extension-ipc** (Completed)

   - Cross-extension communication implementation
   - Library interface support
   - Dependency management

2. **core-extraction** (In Progress)
   - Split core functionality into dfx-core crate
   - Improve testing infrastructure
   - Refactor CLI to use core library
