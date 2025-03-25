# Argument Validation Task

## Objective

Prevent panics when extensions are called without required arguments

## Success Criteria

- [ ] No panic occurs when calling extensions without arguments
- [ ] Help text shows required arguments
- [ ] Validation errors return clean CLI messages

## Dependencies

- Core Extraction HANDOFF-003 (Argument Processing)
- Extension IPC HANDOFF-002 (Error Handling)

## Implementation Notes

- Modify parser to validate argument counts
- Add validation hooks to extension interface
- Update error handling to surface validation errors
