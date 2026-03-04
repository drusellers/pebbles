# Answer Command

You are answering !agent directives from a pebbles change document.

## Current Change

Run `pebbles show $PEBBLES_CHANGE` to see the full change details including title and body.

## Directives to Answer

The environment variable `PEBBLES_DIRECTIVES` contains the numbered list of directives:

```
$PEBBLES_DIRECTIVES
```

Total directives: $PEBBLES_DIRECTIVE_COUNT

## Your Task

Answer each directive in order. Return your responses as a numbered list matching the input format:

```
1. Answer to first directive
2. Answer to second directive
3. Answer to third directive
```

### Guidelines

1. **One answer per directive** - Each number must have exactly one response
2. **Replace in place** - The user will replace each `!agent:` line with your answer
3. **Be specific and actionable** - Provide clear, concrete information
4. **Codebase scan** - If asked to scan the codebase, identify relevant files and patterns
5. **Questions** - If a directive asks a question, provide a thorough answer
6. **Research** - If a directive asks for research, summarize your findings

### Response Format

Your output must be a numbered list starting from 1. Each answer should be concise but complete.

Example:

Input directives:
```
1. Scan codebase for database migration files
2. What ORM does this project use?
```

Your response:
```
1. Found migration files in `src/db/migrations/`. Key files: 001_initial.sql, 002_add_users.sql. Uses SQLx for migrations.
2. This project uses SQLx (https://github.com/launchbadge/sqlx) as its ORM. See `Cargo.toml` for dependency and `src/db/` for usage examples.
```

## Notes

- The response will replace the `!agent:` line inline, so don't include markdown formatting unless the answer should have it
- Be thorough but concise
- If you're unsure about something, state your uncertainty clearly
- The answers should be ready to use without further editing
