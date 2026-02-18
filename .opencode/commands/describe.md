# Generate Commit Description

Generate a commit message for the current changes following Google's CL description best practices.

## Instructions

1. Run the appropriate VCS command to see changes:
   - Git: `git diff --staged` or `git diff HEAD~1`
   - Jujutsu: `jj show --git`

2. Run `pebbles show $PEBBLES_CHANGE` for context about the change.

## Commit Message Format

**First line**: Short imperative summary of WHAT changed
- Use imperative mood: "Add", "Fix", "Remove" (not "Added", "Fixing")
- Should be searchable and stand alone
- Keep it concise but descriptive
- Use changelog type to inform the verb:
  - feature → "Add ..."
  - fix → "Fix ..."
  - change → "Update ...", "Improve ..."
  - removed → "Remove ..."

**Blank line**

**Body**: Explain WHY this change was made
- The problem being solved
- Why this approach was chosen
- Reference the change ID naturally

## Guidelines

- The body explains reasoning, not just restates the diff
- Future developers should understand whether they can safely modify this code
- This will be a permanent part of version control history

## Output

Wrap your commit message in `<commit>` tags:

```
<commit>
Your commit message here
</commit>
```

Output ONLY the commit message inside the tags. No preamble, no explanation.
