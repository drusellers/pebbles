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

## Adding a New Command

1. Create `src/commands/<name>.rs` with `pub async fn run(...) -> Result<()>`
2. Add to `src/commands/mod.rs`
3. Add variant to `Commands` enum in `src/cli.rs`
4. Route in `src/main.rs`
