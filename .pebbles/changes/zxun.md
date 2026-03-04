---
id: zxun
status: draft
priority: medium
changelog_type: null
parent: null
blocked_by: []
tags: []
created_at: 2026-02-19T18:51:24.791182Z
updated_at: 2026-02-19T18:51:24.791182Z
timer_start: null
accumulated_duration_secs: 0
---

# Implement Releases feature for Pebbles

## Implementation Plan

### Overview
Add Releases feature to Pebbles for organizing changes into versioned milestones (like Docket).

### Decisions Made
- Release IDs: Semver format (e.g., '1.0.0')
- One target_release per change
- Auto-create 'unscheduled' release on init
- Store release events in same events table
- Auto-generate changelog on ship (stdout or --file)
- Copy Docket's release show display exactly
- Support ISO 8601 + relative dates for target_date
- Delete: Error if has changes, --force (cascade), --reset (move to unscheduled)

### Implementation Phases
1. Core Data Models & Storage - Release struct, ReleaseStatus enum, extend Change and EventType
2. Validation & Utilities - semver validation, release sorting
3. CLI Commands - release new/list/show/edit/schedule/activate/freeze/ship/cancel/delete
4. UI Enhancements - show target_release in change details, --release filter
5. Changelog Generation - group by ChangelogType on ship
6. Migration & Edge Cases - auto-create unscheduled, handle existing data

### Files to Modify
- src/models.rs (Release, ReleaseStatus, Change.target_release, EventType variants)
- src/db.rs (releases HashMap, CRUD methods)
- src/repository.rs (ensure_unscheduled_release)
- src/cli.rs (Release command group)
- src/commands/release/*.rs (7 new files)
- src/commands/show.rs, list.rs (add release display)
- src/commands/init.rs (call ensure_unscheduled_release)
- src/release.rs (new file for validation)

### Dependencies
- semver crate
- chrono-english or similar for relative dates

### Acceptance Criteria
- [ ] Can create releases with semver IDs
- [ ] Changes can be assigned to releases via schedule command
- [ ] Release list shows progress (completed/total)
- [ ] Release show displays changes grouped by status (copy Docket)
- [ ] Ship command generates changelog
- [ ] Delete release handles assigned changes properly
- [ ] Unscheduled release auto-created on init
- [ ] All existing tests pass
- [ ] New tests for release functionality

## Events

1. 2026-02-19T18:51:24.791217+00:00 [f6wp] created
   {"parent":null,"priority":"medium","title":"Implement Releases feature for Pebbles"}


