# Checksum Validation Task

## Objective

Add checksum verification for extension installation

## Success Criteria

- [ ] --checksum parameter added to extension add
- [ ] SHA256 validation performed during installation
- [ ] Invalid checksum fails installation with clear error

## Dependencies

- Homebrew HANDOFF-004 (Checksum Implementation)
- Quick-Install HANDOFF-008 (Security Requirements)

## Implementation Notes

- Extend manifest format to include optional checksum
- Add checksum validation to download process
- Update CLI help text
