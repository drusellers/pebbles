# Pebbles

Task tracking for AI-assisted development.

Pebbles is a command-line bug/task tracker designed to integrate with [opencode](https://opencode.ai) and both Git and [Jujutsu](https://martinvonz.github.io/jj/) version control. It manages work items through their lifecycle while automatically handling workspaces.

## Installation

### From source

```bash
git clone https://github.com/drusellers/pebbles
cd pebbles
cargo build --release
# Binary is at target/release/pebbles
```

## Quick Start

```bash
# Initialize pebbles in your project
pebbles init

# Create a new change
pebbles new "Fix login validation"

# View all changes
pebbles list

# Show change details
pebbles show <id>

# Start working (auto-runs /implement)
pebbles start <id>

# Or create an isolated workspace
pebbles start <id> --isolate

# Open TUI without auto-running /implement
pebbles start <id> --wait

# Create changes from text file (intake)
pebbles intake features.txt

# Plan and break down a change into actionable steps
pebbles plan <id>

# Delete a change
pebbles delete <id>

# When done, mark complete
pebbles done <id>
```

## Workflow

Pebbles follows a simple status flow:

```
Draft → Approved → InProgress → Review → Done
                      ↓           ↓
                   Blocked    (reject back to InProgress)
                      ↓
                   Paused
```

1. **Draft**: New changes start here. Use for ideas and rough specs.
2. **Approved**: Change is ready to be worked on. (`pebbles approve <id>`)
3. **InProgress**: Work has started with `pebbles start <id>`:
   - Default: Works in current directory, auto-runs `/implement`
   - `--isolate`: Creates an isolated workspace (recommended for large changes)
   - `--wait`: Opens opencode TUI without auto-running `/implement`
4. **Done**: Work is complete. (`pebbles done <id>`)

## Commands

| Command       | Description                                                      |
|---------------|------------------------------------------------------------------|
| `doctor`      | Check for required dependencies (jj, git, EDITOR, opencode)      |
| `init`        | Initialize a new pebbles repository                              |
| `new`         | Create a new change                                              |
| `list`        | List all changes (use `--all` to include done)                   |
| `show`        | Show details of a change                                         |
| `update`      | Update a change's title, body, or priority                       |
| `approve`     | Mark a change as approved for work                               |
| `start`       | Start working on a change (alias: `work`)                        |
| `done`        | Mark a change as done                                            |
| `log`         | Show event history for a change                                  |
| `cleanup`     | Clean up a workspace after work is complete                      |
| `intake`      | Intake text from file or STDIN to create changes                 |
| `completions` | Generate shell completions                                       |
| `current`     | Show the current change (when in a workspace)                    |
| `status`      | Show workspace status including change details                   |
| `edit`        | Edit a change in your editor                                     |
| `delete`      | Delete a change (aliases: `rm`, `del`)                           |
| `block`       | Add a blocking dependency to a change                            |
| `unblock`     | Remove a blocking dependency from a change                       |
| `plan`        | Plan and break down a change into actionable steps               |

## Configuration

Create `.pebbles/config.toml` to customize behavior:

```toml
[work]
skip_permissions = true  # Skip opencode permission prompts
auto_implement = true    # Auto-run implement command

[vcs]
prefer = "auto"  # auto, git, jujutsu

[editor]
# Uses $EDITOR environment variable by default
# command = "vim"
```

## Environment Variables

- `EDITOR` - Editor to use when editing changes (overrides config)
- `PEBBLES_CHANGE` - Set when running from a workspace
- `PEBBLES_VCS` - The detected version control system

## Storage

Changes are stored in `.pebbles/db.json` as a human-readable JSON database.

## AI Integration

When you run `pebbles start <id>`, it:
1. Creates a workspace (`ws-<id>/`) if `--isolate` is specified
2. Sets `PEBBLES_CHANGE=<id>` environment variable
3. Launches opencode with the change context
4. Auto-runs `/implement` (unless `--wait` is specified)

The `.opencode/commands/` directory contains:
- `implement.md` - Guide for AI implementation
- `describe.md` - Commit message generator

## Intake Workflow

The `intake` command allows you to create multiple related changes from a single text input. This is useful for importing feature requests, bug reports, or planning documents.

```bash
# Read from a file
pebbles intake features.txt

# Read from STDIN
cat features.txt | pebbles intake
```

The text is passed to opencode, which will:
1. Parse the content to identify a parent/top-level issue
2. Identify all child/sub-tasks
3. Create the parent change
4. Create all child changes linked to the parent

This creates a hierarchical structure of related work items.

## Dependencies and Blocking

Use the `block` command to mark one change as dependent on another:

```bash
# Mark change-456 as blocked by change-123
pebbles block change-456 change-123
```

Use `unblock` to remove a dependency:

```bash
pebbles unblock change-456 change-123
```

## Planning Workflow

The `plan` command uses AI to break down a change into actionable steps:

```bash
# Plan the current change or specify an ID
pebbles plan
pebbles plan <id>

# Open in TUI without auto-running /plan
pebbles plan --wait
```

This creates a detailed implementation plan as a child change.

## Current Workspace Commands

When working inside a workspace:

```bash
# Show the current change
pebbles current

# Show workspace status
pebbles status

# Edit the current change in your editor
pebbles edit
```

## Command Options

### List Filtering and Sorting

```bash
# Filter by status
pebbles list --status inprogress

# Filter by priority
pebbles list --priority high

# Filter by changelog type
pebbles list --changelog feature

# Sort by different fields
pebbles list --sort priority
pebbles list --sort created --reverse

# Flat list instead of tree view
pebbles list --flat
```

### Start Options

```bash
# Skip permission prompts
pebbles start <id> --skip-permissions

# Verbose output
pebbles start <id> --verbose
```

### Done Options

```bash
# Verify all acceptance criteria are checked
pebbles done <id> --auto

# Force mark done even if criteria not met
pebbles done <id> --force
```

## License

MIT
