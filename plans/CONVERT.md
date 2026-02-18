# CONVERT Plan: Pebbles (formerly Docket Clone) in Rust

## Overview

Build a task tracking CLI tool for AI-assisted development, inspired by [steveklabnik/docket](https://github.com/steveklabnik/docket).

**Key Differences:**
- Uses **opencode** instead of Claude Code
- Supports both **Jujutsu (jj)** and **Git** version control
- Similar feature set with some enhancements

---

## Architecture

### Tech Stack

- **Language**: Rust (async with tokio)
- **CLI Framework**: `clap` with derive macros
- **Database**: `deeb` - ACID-compliant JSON database
- **Serialization**: `serde` + `serde_json`
- **Error Handling**: `anyhow`
- **Terminal UI**: `colored`, `dialoguer`
- **VCS Support**: 
  - Jujutsu: Shell out to `jj` CLI
  - Git: Shell out to `git` CLI
- **Time**: `chrono`
- **IDs**: `ulid` (via deeb) or custom short IDs
- **Config**: `toml`

### Project Structure

```
.
├── Cargo.toml
├── README.md
├── .opencode/
│   └── commands/
│       ├── implement.md      # AI implementation guide
│       └── describe.md       # Commit message generator
├── src/
│   ├── main.rs               # CLI entry point (async)
│   ├── lib.rs                # Module exports
│   ├── cli.rs                # CLI definitions (clap)
│   ├── models.rs             # Change, Event, Status, Priority types
│   ├── db.rs                 # Deeb database initialization and queries
│   ├── config.rs             # Configuration
│   ├── vcs/                  # Version control abstraction
│   │   ├── mod.rs
│   │   ├── git.rs
│   │   └── jujutsu.rs
│   └── commands/             # CLI commands (all async)
│       ├── mod.rs
│       ├── init.rs
│       ├── new.rs
│       ├── list.rs
│       ├── show.rs
│       ├── update.rs
│       ├── approve.rs
│       ├── done.rs
│       ├── work.rs
│       ├── cleanup.rs
│       ├── log.rs
│       └── completions.rs
├── tests/
│   └── integration_tests.rs
└── docs/
    └── schema-versioning.md
```

---

## Core Concepts

### Deeb Database Storage

Uses **deeb** - an ACID-compliant JSON database:

**Collections:**
- `changes` - Main change/task documents
- `events` - Optional audit log of changes (for history)
- `tags` - Tag definitions and associations
- `releases` - Release management (optional)

**Storage:**
- Database stored in `.pebbles/db.json`
- Human-readable JSON format
- Transactions for multi-document operations
- Indexes on frequently queried fields (status, priority, parent)

### Status Flow

```
Draft → Approved → InProgress → Review → Done
                      ↓           ↓
                   Blocked    (reject back to InProgress)
                      ↓
                   Paused
```

### Data Model

Using deeb's `Collection` derive macro:

```rust
use deeb::Collection;
use serde::{Serialize, Deserialize};

#[derive(Collection, Serialize, Deserialize, Clone)]
#[deeb(name = "changes", primary_key = "id")]
pub struct Change {
    pub id: String,           // 4-char alphanumeric
    pub title: String,
    pub body: String,
    pub status: Status,
    pub priority: Priority,
    pub parent: Option<String>,
    pub children: Vec<String>,
    pub dependencies: Vec<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Collection, Serialize, Deserialize, Clone)]
#[deeb(name = "events", primary_key = "id")]
pub struct Event {
    pub id: String,
    pub change_id: String,
    pub event_type: EventType,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Status {
    Draft,
    Approved,
    InProgress,
    Review,
    Done,
    Blocked,
    Paused,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}
```
```

---

## Commands

| Command | Description |
|---------|-------------|
| `init` | Initialize docket repository |
| `new <title>` | Create new change |
| `list` | List changes (filter by status, priority) |
| `show <id>` | Show change details |
| `update <id>` | Update title, body, priority |
| `approve <id>` | Approve change for work |
| `work <id>` | Start work (creates workspace + launches opencode) |
| `done <id>` | Mark as complete |
| `cleanup <id>` | Clean up workspace |
| `log <id>` | Show event history |
| `current` | Show current change (when in workspace) |
| `completions <shell>` | Generate shell completions |

---

## Database Layer (deeb)

### Initialization

```rust
// src/db.rs
use deeb::{Deeb, Collection, Query};
use anyhow::Result;

pub struct Database {
    deeb: Deeb,
}

impl Database {
    pub async fn open(path: &Path) -> Result<Self> {
        let deeb = Deeb::new();
        
        // Add instances for each collection
        deeb.add_instance(
            "docket",
            path.join("db.json"),
            vec![
                Change::entity(),
                Event::entity(),
            ]
        ).await?;
        
        // Add indexes
        Change::add_index(&deeb, "status_idx", vec!["status"], None).await?;
        Change::add_index(&deeb, "priority_idx", vec!["priority"], None).await?;
        Change::add_index(&deeb, "parent_idx", vec!["parent"], None).await?;
        
        Ok(Self { deeb })
    }
    
    pub fn instance(&self) -> &Deeb {
        &self.deeb
    }
}
```

### Repository Pattern

```rust
// src/repository.rs
use anyhow::Result;

pub struct ChangeRepository {
    db: Database,
}

impl ChangeRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
    
    pub async fn create(&self, change: Change) -> Result<Change> {
        Change::insert_one(
            self.db.instance(),
            change,
            None
        ).await
    }
    
    pub async fn find_by_id(&self, id: &str
    ) -> Result<Option<Change>> {
        Change::find_one(
            self.db.instance(),
            Query::eq("id", id),
            None
        ).await
    }
    
    pub async fn find_by_status(
        &self,
        status: Status
    ) -> Result<Vec<Change>> {
        Change::find_many(
            self.db.instance(),
            Query::eq("status", status.to_string()),
            None
        ).await
    }
    
    pub async fn update(
        &self,
        id: &str,
        change: &Change
    ) -> Result<()> {
        Change::update_one(
            self.db.instance(),
            Query::eq("id", id),
            change,
            None
        ).await
    }
}
```

### Short ID Generation

Since deeb uses ULIDs by default, we'll implement custom short IDs:

```rust
// src/id.rs
use rand::{thread_rng, Rng};

const ALPHANUMERIC: &str = "abcdefghijklmnopqrstuvwxyz0123456789";

pub fn generate_id() -> String {
    let mut rng = thread_rng();
    (0..4)
        .map(|_| {
            let idx = rng.gen_range(0..ALPHANUMERIC.len());
            ALPHANUMERIC.chars().nth(idx).unwrap()
        })
        .collect()
}

pub async fn generate_unique_id(db: &Database) -> Result<String> {
    loop {
        let id = generate_id();
        let existing = Change::find_one(
            db.instance(),
            Query::eq("id", &id),
            None
        ).await?;
        
        if existing.is_none() {
            return Ok(id);
        }
    }
}
```

---

## VCS Abstraction

```rust
pub trait Vcs {
    fn name(&self) -> &'static str;
    fn is_repo(&self) -> bool;
    fn create_workspace(&self, name: &str) -> Result<PathBuf>;
    fn cleanup_workspace(&self, name: &str) -> Result<()>;
    fn generate_commit_msg(&self, change: &Change) -> Result<String>;
    fn current_change_id(&self) -> Option<String>;
}

pub struct VcsDetector;
impl VcsDetector {
    pub fn detect() -> Option<Box<dyn Vcs>>;
}
```

### Git Implementation

- Workspaces: Use `git worktree`
- Commit messages: `git commit` with generated message
- Detect: Check for `.git/` directory

### Jujutsu Implementation

- Workspaces: Use `jj workspace`
- Commit messages: `jj describe` with generated message
- Detect: Check for `.jj/` directory

---

## AI Integration (opencode)

### Commands Directory

`.opencode/commands/implement.md` - AI implementation guide

`.opencode/commands/describe.md` - Commit message generator

### Environment Variables

When launching opencode:
- `DOCKET_CHANGE=<id>` - Current change ID
- `DOCKET_VCS=<git|jujutsu>` - Detected VCS

---

## Configuration

`.pebbles/config.toml`:

```toml
[work]
skip_permissions = false  # Skip opencode permission prompts
auto_implement = false    # Auto-run implement command

[vcs]
prefer = "auto"  # auto, git, jujutsu

[output]
colors = true

[editor]
# Uses $EDITOR environment variable by default
# Can override here: command = "vim"
```

### Editor Support

When editing issues (e.g., `docket new`, `docket edit`), the tool respects the `EDITOR` environment variable:

```bash
# Set your preferred editor
export EDITOR="vim"
export EDITOR="code --wait"
export EDITOR="nano"

# Or use the config file to override
```

If `EDITOR` is not set and no config override exists, falls back to a sensible default (vim on Unix, notepad on Windows).

---

## Dependencies

```toml
[dependencies]
# Core
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"

# CLI
clap = { version = "4", features = ["derive"] }
colored = "2"
dialoguer = "0.11"

# Database
deeb = "0.0"
deeb_core = "0.0"
deeb_macros = "0.0"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Time
chrono = { version = "0.4", features = ["serde"] }

# IDs
rand = "0.8"
ulid = "1.0"

# Config
toml = "0.8"

# Other
uuid = { version = "1.0", features = ["v4"] }
dirs = "5.0"
```

---

## Implementation Phases

### Phase 1: Core Infrastructure ✅ COMPLETED

1. **Project Setup**
   - ✅ Create Cargo.toml with dependencies
   - ✅ Set up async module structure
   - ✅ Add basic CLI structure with clap

2. **Database Layer**
   - ✅ Implement JSON-based database (`db.rs`)
   - ✅ Define `Change` and `Event` models
   - ✅ Set up `.pebbles/db.json` storage
   - Repository pattern for data access

3. **ID Generation**
   - ✅ 4-character alphanumeric short IDs

### Phase 2: Basic Commands ✅ COMPLETED

1. ✅ **init** - Initialize repository
2. ✅ **new** - Create changes (with editor support via $EDITOR)
3. ✅ **list** - List changes (with filtering and sorting)
4. ✅ **show** - Display change details
5. ✅ **update** - Modify changes
6. ✅ **edit** - Edit change body in editor
7. ✅ **log** - Show event history

### Phase 3: Status Management ✅ COMPLETED

1. ✅ **approve** - Approve changes
2. ✅ Status transitions validation
3. ✅ Status flow enforcement

### Phase 4: VCS Integration ✅ COMPLETED

1. ✅ **VCS Detection** - Auto-detect git/jj
2. ✅ **work** - Create workspace + launch opencode
3. ✅ **done** - Mark complete (with acceptance criteria check)
4. ✅ **cleanup** - Remove workspace
5. ✅ **current** - Show current change ID

### Phase 5: Advanced Features ⚠️ PARTIAL

1. ⏸️ **Dependencies** - Block/unblock changes (data model ready)
2. ⏸️ **Parent/Child** - Epic/sub-task relationships (data model ready)
3. ⏸️ **Scratchpad** - Quick notes (data model ready)
4. ✅ **Shell Completions** - Tab completion

### Phase 6: Polish ⏸️ NOT STARTED

1. ⏸️ Comprehensive tests
2. ⏸️ Documentation
3. ⏸️ Release workflow

---

## Current Status: MVP COMPLETE ✅

The core functionality is implemented and the project compiles successfully. Key features:

- ✅ All basic CRUD operations
- ✅ Status workflow management
- ✅ VCS integration (Git + Jujutsu)
- ✅ Workspace creation and cleanup
- ✅ Editor integration via $EDITOR
- ✅ AI integration via opencode
- ✅ Shell completions

## Next Steps

1. **Testing**: Add unit and integration tests
2. **Advanced Features**: Implement dependency management
3. **Documentation**: Expand docs and examples
4. **Release**: Set up CI/CD and releases
2. Documentation
3. Release workflow

---

## README Structure

1. Project name and tagline
2. Installation instructions (cargo-binstall, from source)
3. Quick start guide
4. Workflow explanation (status flow)
5. Command reference table
6. Configuration options
7. AI integration details
8. Storage format explanation
9. License

---

## Testing Strategy

- **Unit Tests**: Database queries, model validation, VCS abstractions
- **Integration Tests**: Full command workflows with temporary databases
- **Fixture Tests**: Pre-populated database states for testing specific scenarios

## Deeb Query Examples

```rust
// Find all changes with status InProgress
let in_progress = Change::find_many(
    &db,
    Query::eq("status", "InProgress"),
    None
).await?;

// Find high priority changes
let high_priority = Change::find_many(
    &db,
    Query::eq("priority", "High"),
    None
).await?;

// Complex query: InProgress OR Review
let active = Change::find_many(
    &db,
    Query::or(
        Query::eq("status", "InProgress"),
        Query::eq("status", "Review")
    ),
    None
).await?;

// Transaction example
let mut txn = db.begin_transaction().await;
let change = Change::find_one(&db, Query::eq("id", "abc1"), Some(&mut txn)).await?;
// ... modify change ...
Change::update_one(&db, Query::eq("id", "abc1"), &change, Some(&mut txn)).await?;
db.commit(&mut txn).await?;
```

---

## Open Questions

1. Should we support GitHub/GitLab PR integration?
2. Should we include release management features?
3. Should changes support custom fields/tags?
4. Should there be a web UI component?

---

## Migration Path

For users coming from docket:

- Command interface is similar
- `.pebbles/` directory structure preserved (now uses `db.json`)
- opencode commands similar to Claude commands
- Migration script can convert docket JSONL files to deeb JSON format
