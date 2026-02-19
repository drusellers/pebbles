# Architecture

This document describes the architecture of Pebbles, a CLI task tracker for AI-assisted development.

## Overview

Pebbles is a Rust CLI application that manages work items (changes) through their lifecycle while integrating with version control systems and the opencode AI assistant.

```
┌─────────────────────────────────────────────────────────────────┐
│                          CLI Layer                               │
│  clap-based argument parsing (cli.rs) → Command routing (main.rs)│
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                       Commands Layer                             │
│   Individual command implementations (commands/*.rs)             │
│   init, new, list, show, update, approve, start, done, etc.     │
└─────────────────────────────────────────────────────────────────┘
                                │
            ┌───────────────────┼───────────────────┐
            ▼                   ▼                   ▼
┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐
│   Repository     │  │      VCS         │  │     Config       │
│ (repository.rs)  │  │   (vcs/*.rs)     │  │   (config.rs)    │
│  ChangeRepository│  │ Git, Jujutsu     │  │ TOML-based       │
└──────────────────┘  └──────────────────┘  └──────────────────┘
            │
            ▼
┌──────────────────────────────────────────────────────────────────┐
│                         Data Layer                                │
│    Db (db.rs) → JSON storage (.pebbles/db.json)                   │
│    Models: Change, Event, Status, Priority (models.rs)            │
└──────────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. CLI Layer (`src/cli.rs`)

- Uses `clap` for argument parsing with derive macros
- Defines all commands and their arguments
- Maps to `Commands` enum for routing

### 2. Commands Layer (`src/commands/`)

Each command is a separate module:

| File | Command | Purpose |
|------|---------|---------|
| `init.rs` | `init` | Initialize `.pebbles/` directory |
| `new.rs` | `new` | Create a new change |
| `list.rs` | `list` | List changes with filtering/sorting |
| `show.rs` | `show` | Display change details |
| `update.rs` | `update` | Modify change properties |
| `approve.rs` | `approve` | Mark change as approved |
| `start.rs` | `start` | Start working on a change (alias: `work`) |
| `done.rs` | `done` | Mark change complete |
| `cleanup.rs` | `cleanup` | Remove workspace |
| `log.rs` | `log` | Show event history |
| `current.rs` | `current` | Show current workspace change |
| `edit.rs` | `edit` | Open change in editor |
| `delete.rs` | `delete` | Remove a change |
| `doctor.rs` | `doctor` | Check dependencies |
| `completions.rs` | `completions` | Generate shell completions |

### 3. Data Models (`src/models.rs`)

#### Change
Core entity representing a work item:

```rust
struct Change {
    id: String,           // Unique identifier (e.g., "abc12")
    title: String,        // Brief description
    body: String,         // Detailed spec/markdown
    status: Status,       // Current state
    priority: Priority,   // Importance level
    parent: Option<String>,     // For subtasks
    children: Vec<String>,      // Child change IDs
    dependencies: Vec<String>,  // Blocked by these changes
    tags: Vec<String>,          // Labels
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
```

#### Status Flow
```
Draft → Approved → InProgress → Review → Done
                      ↓           ↓
                   Blocked    (reject back to InProgress)
                      ↓
                   Paused
```

#### Event
Tracks all changes to a change for audit trail:

```rust
struct Event {
    id: String,
    change_id: String,
    event_type: EventType,  // Created, StatusChanged, Updated, etc.
    data: Value,            // JSON payload
    created_at: DateTime<Utc>,
}
```

### 4. Database Layer (`src/db.rs`, `src/repository.rs`)

#### Db
Low-level JSON database operations:

- `open()` - Load database from `.pebbles/db.json`
- `save()` - Persist to disk
- CRUD operations on changes
- Case-insensitive ID lookup with prefix matching

#### ChangeRepository
Higher-level abstraction that:
- Wraps Db with domain operations
- Automatically creates events on changes
- Handles status transitions

### 5. ID Resolution (`src/idish.rs`)

`IDish` type provides flexible ID input:

- Case-insensitive exact match (fastest)
- Case-insensitive prefix matching
- Returns error if ambiguous (multiple matches)

Example: `abc1`, `ABC1`, `ab` all resolve to change `abc1` if unique.

### 6. VCS Abstraction (`src/vcs/`)

Trait-based version control integration:

```rust
trait Vcs {
    fn name(&self) -> &'static str;
    fn is_repo(&self, path: &Path) -> bool;
    fn detect(&self) -> bool;
    fn create_workspace(&self, name: &str) -> Result<PathBuf>;
    fn cleanup_workspace(&self, name: &str) -> Result<()>;
    fn current_workspace_id(&self) -> Option<String>;
    fn commit(&self, message: &str) -> Result<()>;
}
```

Implementations:
- **Git** (`git.rs`) - Uses `git worktree` for isolated workspaces
- **Jujutsu** (`jujutsu.rs`) - Uses `jj workspace` commands

### 7. Configuration (`src/config.rs`)

TOML-based configuration in `.pebbles/config.toml`:

```toml
[work]
skip_permissions = true
auto_implement = true

[vcs]
prefer = "auto"  # auto, git, jujutsu

[editor]
command = "vim"  # Falls back to $EDITOR
```

### 8. Output Formatting (`src/table.rs`)

Simple table renderer with:
- Unicode box-drawing characters
- ANSI color support
- Automatic column width calculation

## Directory Structure

```
project/
├── .pebbles/
│   ├── db.json        # Change database (human-readable JSON)
│   └── config.toml    # User configuration
├── .opencode/
│   └── commands/
│       ├── implement.md  # AI implementation guide
│       └── describe.md   # Commit message generator
├── ws-abc12/          # Workspace for change "abc12"
│   └── (working copy)
└── (main repo)
```

## AI Integration

When `pebbles start <id>` is executed:

1. Optionally creates workspace directory `ws-<id>/` (with `--isolate`)
2. VCS creates isolated working copy (if `--isolate`)
3. Sets environment variables:
   - `PEBBLES_CHANGE=<id>`
   - `PEBBLES_VCS=<git|jujutsu>`
4. Launches `opencode` in working directory
5. Auto-runs `/implement` (unless `--wait` is specified)

The `.opencode/commands/` directory provides AI guidance:
- `implement.md` - How to implement a change
- `describe.md` - How to write commit messages

## Key Design Decisions

### JSON Storage
- Human-readable for debugging
- No external database dependency
- Simple to back up with git

### Case-Insensitive IDs
- More forgiving user input
- Prefix matching for quick reference

### Workspace Pattern
- Isolated working directories per change
- Leverages VCS native branching/worktree features
- Easy context switching between changes

### Event Sourcing
- Complete audit trail of all changes
- Enables `pebbles log` command
- Future: could enable undo/redo

## Dependencies

| Crate | Purpose |
|-------|---------|
| `tokio` | Async runtime |
| `clap` | CLI argument parsing |
| `serde` | JSON/TOML serialization |
| `anyhow` | Error handling |
| `chrono` | Timestamps |
| `colored` | Terminal colors |
| `dialoguer` | Interactive prompts |
| `toml` | Config file parsing |
| `uuid` | Unique event IDs |
