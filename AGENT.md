# AGENT.md

## Project Overview

Pebbles is a Rust CLI task tracker for AI-assisted development. It integrates with opencode and version control systems (Git, Jujutsu).

## Build & Run

**Building:**
```bash
cargo build            # Debug build
cargo build --release  # Release build
```

**Running (self-bootstrapping):**
```bash
cargo run -- <pebbles args>   # e.g., cargo run -- list
```

> **Important:** Always use `cargo run --` instead of `pebbles` when working in this project, since we use pebbles to bootstrap itself.

## Quality Checks

Run these before marking work complete:
```bash
cargo check                    # Verify compilation
cargo clippy -- -D warnings    # Lint (warnings as errors)
cargo test                     # Run tests
```

## Architecture

See ARCHITECTURE.md for full details. Key components:
- `src/cli.rs` - clap-based argument parsing
- `src/commands/` - Individual command implementations
- `src/models.rs` - Change, Event, Status, Priority
- `src/db.rs` / `src/repository.rs` - JSON storage layer
- `src/vcs/` - Git and Jujutsu abstractions

## Code Conventions

- Rust 2024 edition
- `anyhow` for error handling
- `clap` with derive macros for CLI
- Case-insensitive ID matching with prefix support
- Each command in its own file under `src/commands/`
- **Strong ID Types**: We use a strong `ID` type (defined in `src/idish.rs`) instead of `String` or `&str` for all identifiers. This provides type safety and prevents accidental mixing of IDs with other string values.
  - Use `ID::new(...)` to create a new ID
  - Use `id.as_str()` or `id.as_ref()` to get the string representation
  - The `ID` type implements `Deref`, `AsRef<str>`, `Display`, `From<String>`, `From<&str>`, and `FromStr`
  - Collections of IDs should be `Vec<ID>` or `HashMap<ID, ...>`, not `Vec<String>`
  - CLI arguments that accept IDs use `ID` directly (via clap's `FromStr` implementation)
  - The `ID` type provides `resolve()` method for prefix matching against the database

## Adding a New Command

1. Create `src/commands/<name>.rs` with `pub async fn run(...) -> Result<()>`
2. Add to `src/commands/mod.rs`
3. Add variant to `Commands` enum in `src/cli.rs`
4. Route in `src/main.rs`

## ID Type Usage Examples

```rust
use crate::idish::ID;

// Creating IDs
let id = ID::new("abc123");
let id: ID = "abc123".parse().unwrap();
let id = ID::from("abc123");

// Using IDs in structs
pub struct Change {
    pub id: ID,
    pub parent: Option<ID>,
    pub children: Vec<ID>,
}

// Passing IDs to functions
fn get_change(id: &ID) -> Option<&Change> {
    // ...
}

// Display/formatting
println!("Change: {}", id);  // Uses Display trait

// Resolving partial IDs
let partial: ID = "ab".parse().unwrap();
let full_id = partial.resolve(&db)?;  // Resolves to full ID
```
