# Persona Guidelines

This directory contains role-specific guidelines for different personas in the icp project. Each persona file captures the essential traits, responsibilities, and concrete verification steps for that role.

## Core Principles

1. **Concrete Over Abstract**

   - Every requirement must be explicitly verifiable
   - Use "MUST" to indicate required actions
   - Avoid vague directives

2. **Role-Specific Structure**

   - Let structure emerge from role's needs
   - Include only relevant sections
   - Adapt format to workflow

3. **Verifiable Actions**
   - Break down into atomic steps
   - Each step must be independently checkable
   - Clear success criteria

## Common Patterns

From our existing personas, these patterns have emerged as useful:

### Personality Traits

Define 3-5 key traits that characterize the role's approach:

```markdown
## Personality Traits

- [Trait]: [How it manifests in the role]
```

### Core Responsibilities

List primary areas of focus:

```markdown
## Core Responsibilities

You [role description]. Your key areas include:

- [Area 1]
- [Area 2]
```

### Session Requirements

Break down into concrete steps:

```markdown
### [Phase Name]

You MUST verify:

- [ ] [Concrete action]
```

## Creating New Personas

1. Study existing personas for inspiration
2. Focus on role-specific needs
3. Use concrete, verifiable language
4. Include only necessary sections
5. Evolve structure as needed

## Examples

See QA_TESTER.md for a working example of these principles in practice. Key patterns demonstrated include:

### Personality Traits Example

```markdown
- Diligent: Thorough and careful in work, leaving no stone unturned
- Skeptical: Questions assumptions and looks for edge cases
```

### Session Requirements Example

```markdown
### Test Planning

For each testable claim, you MUST:

- [ ] List happy path test cases
- [ ] List edge case test cases
```

These examples show how to make guidelines concrete and actionable while maintaining flexibility for different roles.
