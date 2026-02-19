# Plan Command

You are helping plan and break down a pebbles change into actionable steps.

## Current Change

Run `pebbles show $PEBBLES_CHANGE` to see the current change details.

## Planning Process

### 1. Review Acceptance Criteria

Check if the change has acceptance criteria in its body:
- Look for sections like "Acceptance Criteria", "Checklist", "TODO", or similar
- If no acceptance criteria exist, ask the user: "This change doesn't have acceptance criteria yet. Would you like me to help you add some?"
- If the user agrees, work with them to define clear, testable criteria

### 2. Scan the Codebase

Scan the codebase to understand the context:
- Look at the project structure and relevant files
- Identify where changes need to be made
- Find existing patterns or similar implementations
- Note any configuration files, dependencies, or architecture constraints

Add findings to the change body under a "## Context" or "## Implementation Notes" section.

### 3. Create a Checklist

Based on the acceptance criteria and codebase analysis, create a checklist of implementation steps:

```markdown
## Implementation Checklist

- [ ] Step 1: Description
- [ ] Step 2: Description
- [ ] Step 3: Description
```

Each step should be:
- Small and actionable (can be completed in one sitting)
- Specific about what needs to be done
- Ordered by dependency (what needs to happen first)
- Testable when completed

### 4. Identify Dependencies

If the change depends on other changes:
- Run `pebbles list` to see existing changes
- Identify any blockers or prerequisites
- Use `pebbles block <change_id> <dependency_id>` to add dependencies if needed

### 5. Update the Change

Use `pebbles update` to save all the planning work:
- Add acceptance criteria (if missing)
- Add context from codebase scan
- Add the implementation checklist
- Add any notes or questions for clarification

## Example Output

After planning, the change body should look like:

```markdown
## Goal
Brief description of what needs to be done

## Acceptance Criteria
- [ ] Criterion 1
- [ ] Criterion 2
- [ ] Criterion 3

## Context
- Uses React 18 with TypeScript
- Related files: src/components/Button.tsx, src/styles/button.css
- Similar implementation exists in src/components/Input.tsx

## Implementation Checklist
- [ ] Create Button component skeleton
- [ ] Add TypeScript types and props
- [ ] Implement basic styling
- [ ] Add unit tests
- [ ] Update documentation

## Questions
- Should the button support both light and dark themes?
- What size variants are needed?
```

## Notes

- Keep the plan realistic and achievable
- Break down large tasks into smaller steps
- If the change seems too large, suggest splitting it into child issues
- Use `pebbles new --parent $PEBBLES_CHANGE` to create child issues if needed
