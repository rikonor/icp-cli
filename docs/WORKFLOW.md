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
├── GUIDELINES.md    # Foundation for all development sessions
├── WORKFLOW.md      # This file - explains project organization
└── projects/       # All project-specific documentation
    └── project-name/
        ├── PROJECT.md         # Project overview and task listing
        └── tasks/            # Task-specific documentation
            └── task-name/
                ├── TASK.md   # Task details and requirements
                └── HANDOFF/  # Task-specific handoff notes
```

## Session Structure

1. Each session begins by reviewing:

   - GUIDELINES.md for development principles
   - Current task's documentation
   - Latest task handoff document

2. Task Assignment:

   - Each session focuses on specific tasks
   - Tasks are identified at session start
   - Progress tracked in PROJECT.md and TASK.md

3. Documentation Updates:
   - Update TASK.md with current progress
   - Create sequentially numbered handoff document
   - Update PROJECT.md as needed

## Task Lifecycle

1. **Creation**

   - Create task directory under project's tasks/
   - Write initial TASK.md with requirements
   - Create HANDOFF directory

2. **Development**

   - Track progress in TASK.md
   - Document decisions and changes
   - Create numbered handoff documents
   - Update PROJECT.md status

3. **Completion**
   - Update final status in TASK.md
   - Create final handoff document
   - Update PROJECT.md task status

## Current Projects

1. **extension-ipc** (Completed)

   - Cross-extension communication implementation
   - Library interface support
   - Dependency management

2. **core-extraction** (In Progress)
   - Split core functionality into dfx-core crate
   - Improve testing infrastructure
   - Refactor CLI to use core library
