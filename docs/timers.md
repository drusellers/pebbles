# Time Tracking

Pebbles includes built-in time tracking to help you understand how long you spend on each change.

## Overview

Time tracking in Pebbles is designed to be:
- **Automatic**: Timers start when you begin work and stop when you finish
- **Session-based**: Tracks individual work sessions with accumulated totals
- **Non-intrusive**: Records time in the background without disrupting your workflow

## Data Model

Each change tracks:

```yaml
---
timer_start: 2026-03-04T10:00:00Z              # When current session started (null if stopped)
accumulated_duration_secs: 3600                # Total seconds from previous sessions
---
```

### Key Concepts

- **Accumulated Time**: Total time from all completed sessions
- **Active Session**: Current running timer (if any)
- **Total Time**: Accumulated + active session duration

## Commands

### Manual Timer Control

```bash
# Start the timer for a change
pebbles timer start [CHANGE_ID]

# Stop the timer for a change  
pebbles timer stop [CHANGE_ID]

# Check timer status and accumulated time
pebbles timer status [CHANGE_ID]
```

If no `CHANGE_ID` is provided, the command uses the current workspace change.

### Automatic Timer Management

Timers are managed automatically when you use Pebbles workflow commands:

- **`pebbles start`**: Automatically starts the timer when you begin work
- **`pebbles done`**: Automatically stops the timer and reports session time

### Viewing Time

Time information appears in:

- **`pebbles show`**: Displays total time (with "RUNNING" indicator if active)
- **`pebbles timer status`**: Detailed breakdown including:
  - Current status (RUNNING/STOPPED)
  - Session start time (if running)
  - Elapsed time in current session
  - Total accumulated time

## Output Format

Time is displayed in human-readable format:
- Hours and minutes: `2h 15m`
- Minutes and seconds: `45m 30s`
- Seconds only: `45s`

## Events

Timer activity is recorded in the change's event log:

```markdown
## Events

1. 2026-03-04T10:00:00Z [abc1] timer_started
   {}

2. 2026-03-04T11:30:00Z [def2] timer_stopped
   {"duration_secs": 5400}
```

This provides an audit trail of when you worked on each change.

## Use Cases

### Understanding Time Investment

Track how long changes actually take to complete:

```bash
$ pebbles show my-change

Time:
  3h 45m
```

### Session-Based Work

Timer automatically handles interruptions. If you:
1. Start work at 9am (timer auto-starts)
2. Stop for lunch at 12pm (timer still running, tracking time)
3. Run `pebbles done` at 2pm (timer stops, reports 5 hours)

Or manually control sessions:
```bash
pebbles timer start my-change
# ... work ...
pebbles timer stop my-change
# ... break ...
pebbles timer start my-change  # Resumes tracking
```

### Tracking Multiple Sessions

Accumulated time persists across sessions:
- Monday: Work 2 hours → accumulated: 2h
- Tuesday: Work 1.5 hours → accumulated: 3.5h  
- Wednesday: Work 30 min → accumulated: 4h

## Migration

When migrating from legacy JSON storage, timer fields default to:
- `timer_start: null`
- `accumulated_duration_secs: 0`

Existing changes will show no time until you start tracking.

## Implementation Notes

- Timer state is stored in the change's markdown frontmatter
- Timer events are stored in the change's event log
- The timer uses UTC timestamps for consistency
- Duration is stored in seconds for precision
- Timer methods on the `Change` struct:
  - `is_timer_running()` - Check if timer is active
  - `total_duration_secs()` - Get total time (accumulated + current session)
  - `timer_start()` - Start the timer (idempotent)
  - `timer_stop()` - Stop the timer and add to accumulated
