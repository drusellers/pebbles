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

# Start working (creates workspace + launches opencode)
pebbles work <id>

# Or work without creating a workspace
pebbles build <id>

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
3. **InProgress**: Work has started. Use either:
   - `pebbles work <id>` - Creates an isolated workspace (recommended for large changes)
   - `pebbles build <id>` - Works in current directory (for quick fixes)
4. **Done**: Work is complete. (`pebbles done <id>`)

## Commands

| Command       | Description                                                 |
|---------------|-------------------------------------------------------------|
| `doctor`      | Check for required dependencies (jj, git, EDITOR, opencode) |
| `init`        | Initialize a new pebbles repository                         |
| `new`         | Create a new change                                         |
| `list`        | List all changes (use `--all` to include done)              |
| `show`        | Show details of a change                                    |
| `update`      | Update a change's title, body, or priority                  |
| `approve`     | Mark a change as approved for work                          |
| `done`        | Mark a change as done                                       |
| `work`        | Start working on a change (creates workspace + opencode)    |
| `build`       | Start working on a change without creating workspace        |
| `log`         | Show event history for a change                             |
| `cleanup`     | Clean up a workspace after work is complete                 |
| `completions` | Generate shell completions                                  |

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

When you run `pebbles work <id>`, it:
1. Creates a workspace (`ws-<id>/`)
2. Sets `PEBBLES_CHANGE=<id>` environment variable
3. Launches opencode with the change context

The `.opencode/commands/` directory contains:
- `implement.md` - Guide for AI implementation
- `describe.md` - Commit message generator

## License

MIT
