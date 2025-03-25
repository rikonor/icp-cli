# Force Install Task

## Objective

Allow overwriting existing extensions via --force flag

## Success Criteria

- [ ] --force flag added to extension add command
- [ ] Existing extensions can be overwritten when --force specified
- [ ] Default behavior remains non-destructive

## Dependencies

- Package Manager HANDOFF-004 (Force Flag Pattern)
- Core Extraction HANDOFF-005 (Component Protection)

## Implementation Notes

- Add force: bool parameter to extension installation
- Update CLI parser to accept --force flag
- Modify installation logic to respect force flag
