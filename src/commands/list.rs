use anyhow::{Context, Result};
use colored::Colorize;
use std::collections::{HashMap, HashSet};

use crate::cli::ListArgs;
use crate::config::get_db_path;
use crate::models::{Change, Status};
use crate::repository::ChangeRepository;
use crate::table::SimpleTable;

pub async fn list(args: ListArgs) -> Result<()> {
    let db_path = get_db_path()
        .context("Not in a pebbles repository. Run 'pebbles init' first.")?;

    let repo = ChangeRepository::open(db_path).await?;

    let status = args.status.as_deref();
    let priority = args.priority.as_deref();
    let changelog = args.changelog.as_deref();

    let changes = repo.list(status, priority, changelog, args.all);

    if changes.is_empty() {
        println!("No changes found.");
        return Ok(());
    }

    // Calculate unique prefixes for IDs (like jj)
    let ids: Vec<String> = changes.iter().map(|c| c.id.clone()).collect();
    let id_prefixes = calculate_unique_prefixes(&ids);

    if args.flat {
        // Flat list mode - use bordered table
        render_flat_list(changes, &id_prefixes, &args)?;
    } else {
        // Tree mode - use borderless table with tree structure
        render_tree_view(changes, &id_prefixes, &args)?;
    }

    Ok(())
}

fn render_flat_list(
    mut changes: Vec<&Change>,
    id_prefixes: &HashMap<String, usize>,
    args: &ListArgs,
) -> Result<()> {
    // Sort
    changes.sort_by(|a, b| {
        let cmp = match args.sort.as_str() {
            "created" => a.created_at.cmp(&b.created_at),
            "updated" => a.updated_at.cmp(&b.updated_at),
            "priority" => priority_rank(&a.priority).cmp(&priority_rank(&b.priority)),
            _ => a.created_at.cmp(&b.created_at),
        };

        if args.reverse {
            cmp.reverse()
        } else {
            cmp
        }
    });

    // Create table with borders
    let mut table = SimpleTable::new(vec![
        "ID".bold().to_string(),
        "Status".bold().to_string(),
        "Priority".bold().to_string(),
        "Chg".bold().to_string(),
        "Age".bold().to_string(),
        "Title".bold().to_string(),
    ]);

    // Add rows
    for change in changes {
        let row = format_change_row(change, id_prefixes);
        table.add_row(row);
    }

    table.print();

    Ok(())
}

fn render_tree_view(
    changes: Vec<&Change>,
    id_prefixes: &HashMap<String, usize>,
    args: &ListArgs,
) -> Result<()> {
    // Create a map of ID -> change for quick lookup (needed for completion counts)
    let change_map: HashMap<String, &Change> =
        changes.iter().map(|c| (c.id.clone(), *c)).collect();

    // Build tree structure
    let tree = build_tree_structure(&changes, args)?;

    // Create borderless table
    let mut table = SimpleTable::borderless(vec![
        "ID".bold().to_string(),
        "Status".bold().to_string(),
        "Priority".bold().to_string(),
        "Chg".bold().to_string(),
        "Age".bold().to_string(),
        "Title".bold().to_string(),
    ]);

    // Add rows with tree prefixes
    for node in tree {
        let mut row = format_change_row(node.change, id_prefixes);

        // Apply tree prefix to title column (last column)
        if node.depth > 0 {
            let title = &row[5];
            let tree_prefix = build_tree_prefix(node.depth, node.is_last);
            row[5] = format!("{}{}", tree_prefix, title);
        }

        // For parents, show completion count in status
        if !node.change.children.is_empty() {
            let done_count = node
                .change
                .children
                .iter()
                .filter(|child_id| {
                    // Look up the child to get its status
                    change_map
                        .get(*child_id)
                        .map(|c| c.status == Status::Done)
                        .unwrap_or(false)
                })
                .count();
            let total = node.change.children.len();
            row[1] = format!("[{}/{}]", done_count, total);
        }

        table.add_row(row);
    }

    table.print();

    Ok(())
}

/// Tree node structure
struct TreeNode<'a> {
    change: &'a Change,
    depth: usize,
    is_last: bool,
}

/// Build tree structure from flat list of changes
fn build_tree_structure<'a>(
    changes: &[&'a Change],
    args: &ListArgs,
) -> Result<Vec<TreeNode<'a>>> {
    let mut result = Vec::new();
    let mut visited = HashSet::new();

    // Create a map of ID -> change for quick lookup
    let change_map: HashMap<String, &'a Change> =
        changes.iter().map(|c| (c.id.clone(), *c)).collect();

    // Find root items (no parent, or parent not in the filtered list)
    let mut roots: Vec<&'a Change> = changes
        .iter()
        .filter(|c| {
            c.parent
                .as_ref()
                .map(|p| !change_map.contains_key(p))
                .unwrap_or(true)
        })
        .copied()
        .collect();

    // Sort roots
    sort_changes(&mut roots, args.sort.as_str(), args.reverse);

    // Build tree recursively
    for (i, root) in roots.iter().enumerate() {
        let is_last = i == roots.len() - 1;
        add_node_recursive(
            *root,
            0,
            is_last,
            &change_map,
            &mut result,
            &mut visited,
            args,
        );
    }

    // Handle orphaned children (children whose parent is not in the list)
    // They should appear as roots
    for change in changes {
        if !visited.contains(&change.id) {
            // This shouldn't happen with the above logic, but just in case
            result.push(TreeNode {
                change,
                depth: 0,
                is_last: true,
            });
        }
    }

    Ok(result)
}

fn add_node_recursive<'a>(
    change: &'a Change,
    depth: usize,
    is_last: bool,
    change_map: &HashMap<String, &'a Change>,
    result: &mut Vec<TreeNode<'a>>,
    visited: &mut HashSet<String>,
    args: &ListArgs,
) {
    if visited.contains(&change.id) {
        return; // Prevent infinite recursion on cycles
    }

    visited.insert(change.id.clone());

    result.push(TreeNode {
        change,
        depth,
        is_last,
    });

    // Find children
    let children: Vec<&'a Change> = change
        .children
        .iter()
        .filter_map(|child_id| change_map.get(child_id).copied())
        .collect();

    // Sort children
    let mut sorted_children = children;
    sort_changes(&mut sorted_children, args.sort.as_str(), args.reverse);

    // Recursively add children
    for (i, child) in sorted_children.iter().enumerate() {
        let child_is_last = i == sorted_children.len() - 1;
        add_node_recursive(
            *child,
            depth + 1,
            child_is_last,
            change_map,
            result,
            visited,
            args,
        );
    }
}

fn sort_changes<'a>(changes: &mut Vec<&'a Change>, sort_by: &str, reverse: bool) {
    changes.sort_by(|a, b| {
        let cmp = match sort_by {
            "created" => a.created_at.cmp(&b.created_at),
            "updated" => a.updated_at.cmp(&b.updated_at),
            "priority" => priority_rank(&a.priority).cmp(&priority_rank(&b.priority)),
            _ => a.created_at.cmp(&b.created_at),
        };

        if reverse {
            cmp.reverse()
        } else {
            cmp
        }
    });
}

fn build_tree_prefix(depth: usize, is_last: bool) -> String {
    let mut prefix = String::new();

    // Add indentation for each level of depth
    for _ in 0..depth {
        prefix.push_str("  ");
    }

    // Add branch character
    if is_last {
        prefix.push_str("└─ ");
    } else {
        prefix.push_str("├─ ");
    }

    prefix
}

fn format_change_row(change: &Change, id_prefixes: &HashMap<String, usize>) -> Vec<String> {
    let status_str = format_status(&change.status.to_string());
    let priority_str = format_priority(&change.priority.to_string());
    let changelog_str = change
        .changelog_type
        .as_ref()
        .map(|ct| format_changelog_abbrev(&ct.to_string()))
        .unwrap_or_else(|| "".to_string());
    let age = format_age(&change.created_at);

    // Truncate title if too long
    let title = if change.title.len() > 60 {
        format!("{}...", &change.title[..57])
    } else {
        change.title.clone()
    };

    // Format ID with unique prefix highlighted
    let prefix_len = id_prefixes
        .get(&change.id)
        .copied()
        .unwrap_or(change.id.len());
    let formatted_id = format_id_with_prefix(&change.id, prefix_len);

    vec![
        formatted_id,
        status_str,
        priority_str,
        changelog_str,
        age,
        title,
    ]
}

fn format_status(status: &str) -> String {
    let styled = match status {
        "draft" => "draft".dimmed(),
        "approved" => "approved".yellow(),
        "in_progress" => "in_progress".blue(),
        "review" => "review".magenta(),
        "done" => "done".green(),
        "blocked" => "blocked".red(),
        "paused" => "paused".dimmed(),
        _ => status.normal(),
    };
    styled.to_string()
}

fn format_priority(priority: &str) -> String {
    let styled = match priority {
        "low" => "low".dimmed(),
        "medium" => "medium".normal(),
        "high" => "high".yellow(),
        "critical" => "critical".red().bold(),
        _ => priority.normal(),
    };
    styled.to_string()
}

fn priority_rank(priority: &crate::models::Priority) -> u8 {
    use crate::models::Priority;
    match priority {
        Priority::Critical => 0,
        Priority::High => 1,
        Priority::Medium => 2,
        Priority::Low => 3,
    }
}

fn format_changelog_abbrev(changelog: &str) -> String {
    use colored::Colorize;
    match changelog {
        "feature" => "F".green().bold().to_string(),
        "fix" => "X".red().to_string(),
        "change" => "C".yellow().to_string(),
        "deprecated" => "D".dimmed().to_string(),
        "removed" => "R".red().bold().to_string(),
        "security" => "S".red().bold().to_string(),
        "internal" => "I".dimmed().to_string(),
        _ => changelog.to_string(),
    }
}

fn format_age(datetime: &chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(*datetime);

    if duration.num_days() > 0 {
        format!("{}d", duration.num_days())
    } else if duration.num_hours() > 0 {
        format!("{}h", duration.num_hours())
    } else if duration.num_minutes() > 0 {
        format!("{}m", duration.num_minutes())
    } else {
        "now".to_string()
    }
}

/// Calculate the unique prefix length for each ID
/// Returns a map of ID -> prefix length needed to be unique
fn calculate_unique_prefixes(ids: &[String]) -> HashMap<String, usize> {
    let mut result = HashMap::new();

    for id in ids {
        // Find the minimum prefix length that makes this ID unique
        let mut prefix_len = 1;
        'outer: while prefix_len <= id.len() {
            let prefix = &id[..prefix_len];

            // Check if this prefix is unique
            let conflicts: Vec<&String> = ids
                .iter()
                .filter(|other| other.starts_with(prefix) && *other != id)
                .collect();

            if conflicts.is_empty() {
                // This prefix is unique
                break 'outer;
            }

            prefix_len += 1;
        }

        result.insert(id.clone(), prefix_len);
    }

    result
}

/// Format an ID with its unique prefix highlighted
fn format_id_with_prefix(id: &str, prefix_len: usize) -> String {
    if prefix_len >= id.len() {
        // Full ID is the unique prefix
        id.cyan().bold().to_string()
    } else {
        // Split into prefix (bold) and rest (dimmed)
        let prefix = &id[..prefix_len];
        let rest = &id[prefix_len..];
        format!("{}{}", prefix.cyan().bold(), rest.cyan().dimmed())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_change(id: &str, parent: Option<&str>, title: &str) -> Change {
        Change {
            id: id.to_string(),
            title: title.to_string(),
            body: "".to_string(),
            status: Status::Draft,
            priority: crate::models::Priority::Medium,
            changelog_type: None,
            parent: parent.map(|p| p.to_string()),
            children: Vec::new(),
            dependencies: Vec::new(),
            tags: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_build_tree_prefix() {
        assert_eq!(build_tree_prefix(1, false), "  ├─ ");
        assert_eq!(build_tree_prefix(1, true), "  └─ ");
        assert_eq!(build_tree_prefix(2, false), "    ├─ ");
        assert_eq!(build_tree_prefix(2, true), "    └─ ");
    }

    #[test]
    fn test_calculate_unique_prefixes() {
        let ids = vec![
            "abc1".to_string(),
            "abc2".to_string(),
            "def3".to_string(),
        ];

        let prefixes = calculate_unique_prefixes(&ids);

        // "abc1" and "abc2" share prefix "abc", so need 4 chars to be unique
        assert_eq!(prefixes.get("abc1").unwrap(), &4);
        assert_eq!(prefixes.get("abc2").unwrap(), &4);
        // "def3" is unique with just "d"
        assert_eq!(prefixes.get("def3").unwrap(), &1);
    }

    #[test]
    fn test_format_id_with_prefix() {
        let id = "abc123";

        // Full ID should be the whole thing
        let result = format_id_with_prefix(id, 6);
        assert!(result.contains("abc123"), "Should contain full ID: {}", result);

        // Partial should split into prefix and rest
        let result = format_id_with_prefix(id, 3);
        assert!(result.contains("abc"), "Should contain prefix: {}", result);
        assert!(result.contains("123"), "Should contain rest: {}", result);
    }
}
