# Intake Command

You are processing a large intake item and breaking it down into actionable changes.

## Input Processing

The user has provided a large text (from a file or pasted content) that needs to be analyzed and converted into tracked changes.

## Instructions

### 1. Analyze the Content

Read through the entire input and identify:
- **Main goals or objectives** - What is the user trying to achieve?
- **Individual tasks or features** - What specific work needs to be done?
- **Dependencies** - Does anything need to happen in a specific order?
- **Scope boundaries** - What is explicitly NOT included?

### 2. Identify Change Boundaries

Break the content into logical, independently completable changes:
- Each change should represent one coherent piece of work
- Changes should be small enough to complete in one session or less
- Group related tasks together where it makes sense
- Identify if any changes are prerequisites for others

### 3. Create Changes

For each identified change:

```
pebbles new "Brief, actionable title"
```

Then edit the change body to include:

- **Goal**: Clear statement of what this change accomplishes
- **Acceptance Criteria**: Checklist of what "done" looks like
- **Context**: Relevant context from the intake (copy relevant sections)
- **Dependencies**: Any prerequisite changes
- **Priority**: Set based on the original content's urgency

### 4. Set Up Relationships

If changes depend on each other:

```
pebbles block <change_id> <dependency_id>
```

### 5. Summarize

After creating all changes, provide a summary:
- List all created changes with their IDs
- Highlight any dependencies or sequencing requirements
- Note any questions or ambiguities that need clarification

## Best Practices

- **Be conservative**: When in doubt, create smaller, more focused changes
- **Preserve context**: Copy relevant excerpts from the original intake into each change's body
- **Ask questions**: If the intake is ambiguous, create the changes but note what needs clarification
- **Link back**: Include a reference to the original intake in your summary

## Output Format

Provide:
1. A brief analysis of what you found
2. The list of changes created with IDs
3. Any relationships or dependencies established
4. Next steps or questions for the user
