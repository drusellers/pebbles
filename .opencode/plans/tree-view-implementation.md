# Tree View Implementation - COMPLETED

## Summary
Tree view for issues with parent/child relationships is now the default display. All changes have been implemented and tested.

## What Was Implemented

### 1. Table Display (`src/table.rs`)
- Added `borderless` mode to `SimpleTable`
- Borderless mode displays header with underline, no box borders
- Maintains column alignment
- Added comprehensive unit tests for both bordered and borderless modes
- Added `to_string()` method for testing output

### 2. CLI Arguments (`src/cli.rs`)
- Added `--flat` flag to `ListArgs` - shows old bordered table format
- Added `--parent` flag to `UpdateArgs` - allows reparenting existing changes

### 3. Tree View (`src/commands/list.rs`)
- Tree view is now the default display mode
- Hierarchical display with tree prefixes (├─, └─)
- Parent items show before children
- Completion indicators [x/y] in status column for parent items
- Handles orphans and circular reference prevention
- Added unit tests for tree prefix generation and ID formatting

### 4. Parent Management (`src/commands/update.rs`)
- Added `update_parent()` helper function
- Supports:
  - Setting parent: `pebbles update abc1 --parent xyz9`
  - Removing parent: `pebbles update abc1 --parent ""`
  - Circular reference detection
  - Bidirectional relationship updates (parent.children, child.parent)

## Usage Examples

### View changes (default tree view)
```bash
pebbles list
# ID    Status    Priority    Title
# ─────────────────────────────────────────────
# epic1 [1/3]     high        Epic Title
#                 ├─ Child 1
#                 ├─ Child 2  
#                 └─ Child 3
```

### View changes (flat mode)
```bash
pebbles list --flat
# Shows bordered table without hierarchy
```

### Create child issue
```bash
pebbles new "Child issue" --parent abc1
```

### Reparent existing issue
```bash
pebbles update child1 --parent newparent
```

### Remove parent
```bash
pebbles update child1 --parent ""
```

## Tests Added
- `test_simple_table_bordered` - Bordered table rendering
- `test_simple_table_borderless` - Borderless table rendering
- `test_column_widths_auto_adjust` - Width calculation
- `test_row_with_colored_content` - ANSI handling
- `test_strip_ansi` - ANSI stripping
- `test_build_tree_prefix` - Tree prefix generation
- `test_calculate_unique_prefixes` - ID prefix calculation
- `test_format_id_with_prefix` - ID formatting

## Files Modified
1. `src/table.rs` - Borderless mode + tests
2. `src/cli.rs` - --flat and --parent flags
3. `src/commands/list.rs` - Tree view implementation + tests
4. `src/commands/update.rs` - Parent reparenting logic

## Breaking Changes
- `pebbles list` now shows tree view by default (was bordered flat list)
- Users who want old behavior must use `pebbles list --flat`

## Architecture Notes

### Tree Building
The tree is built by:
1. Filtering changes based on criteria
2. Identifying roots (no parent OR parent not in filtered set)
3. Recursively adding children with depth tracking
4. Sorting within each level

### Borrow Checker Workarounds
The parent update logic required careful restructuring to avoid multiple mutable borrows of the repository. Information is gathered first, then all mutations happen in a controlled sequence.

### Cycle Detection
`would_create_cycle()` walks up the parent chain from the proposed parent. If it encounters the child ID, a cycle would be created and the operation is rejected.
