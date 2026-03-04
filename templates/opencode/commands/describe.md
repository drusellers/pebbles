# Generate Commit Description

Generate a commit message for the current changes following the OpenCommits standard.

## Instructions

1. Run the appropriate VCS command to see changes:
   - Git: `git diff --staged` or `git diff HEAD~1`
   - Jujutsu: `jj show --git`

2. Run `pebbles show $PEBBLES_CHANGE` for context about the change.

## Commit Message Format

Following [OpenCommits](https://github.com/opencommits-org/opencommits/blob/main/opencommits.md):

```
Type[!] [scope] description

[optional body explaining WHY]
```

**Type** — 3-letter capitalized identifier (required):
- `Add` — New features or capabilities
- `Fix` — Bug fixes or incorrect behavior
- `Ref` — Internal refactoring and cleanup
- `Opt` — Performance optimizations
- `Rmv` — Removal of code, features, or files
- `Doc` — Documentation and comments
- `Tst` — Tests and test-related changes
- `Sty` — Style and formatting
- `Chr` — Maintenance and tooling changes
- `Rev` — Commit reversals

**!** — Optional modifier for breaking changes (appended directly to Type)

**scope** — Optional lowercase domain identifier (e.g., `api`, `ui`, `db`, `auth`)

**description** — Lowercase, concise, specific, no trailing period

**body** — Optional explanation of WHY the change was made (the body is optional)

## Examples

**Single line (no body):**

```
Fix api pagination offset calculation
```

**Single line (breaking change):**

```
Add! new api pagination offset calculation
```

**With body:**

```
Ref core error handling

Centralize error handling into a single middleware to reduce
duplication across controllers. This approach was chosen to
ensure consistent error responses and simplify future changes
to error formatting.
```

## Guidelines

- Use imperative mood in descriptions
- Keep the first line under 72 characters when possible
- The body explains reasoning, not just restates the diff
- When using `!` for breaking changes, describe the impact in the body
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
