# Implement Change

You are helping implement a change tracked by pebbles.

## Instructions

1. **Get the change context** by running `pebbles show $PEBBLES_CHANGE` to understand what needs to be implemented.

2. **Understand the requirements**:
   - Read the **Goal** section
   - Review the **Acceptance Criteria** 
   - Check the **Context** section for constraints

3. **Plan and implement**:
   - Create a todo list to track progress
   - Work through each acceptance criterion
   - Follow best practices for the codebase

4. **Run checks before finishing**:
   - Run `cargo check` to verify code compiles
   - Run `cargo clippy -- -D warnings` for lints
   - Run `cargo test` if tests exist

5. **Update progress**:
   - Use `pebbles update $PEBBLES_CHANGE` to update the body with progress
   - Add entries to the Log section: `- YYYY-MM-DD: Description`
   - Check off acceptance criteria with `- [x]`

6. **Set changelog type** when complete:
   - Ask: "What type of change is this for the changelog?"
   - Options: feature, fix, change, deprecated, removed, security, internal
   - Set with: `pebbles update $PEBBLES_CHANGE --changelog TYPE`

7. **Mark as done**:
   - Use `pebbles done $PEBBLES_CHANGE --auto` to verify all criteria are met
   - Use `--force` if needed to override

## Getting Started

Begin by running `pebbles show $PEBBLES_CHANGE` and present a summary.
