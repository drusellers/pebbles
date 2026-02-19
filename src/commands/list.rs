use anyhow::{Context, Result};
use colored::Colorize;

use crate::cli::ListArgs;
use crate::config::get_db_path;
use crate::repository::ChangeRepository;
use crate::table::SimpleTable;

pub async fn list(args: ListArgs) -> Result<()> {
    let db_path = get_db_path()
        .context("Not in a pebbles repository. Run 'pebbles init' first.")?;

    let repo = ChangeRepository::open(db_path).await?;

    let status = args.status.as_deref();
    let priority = args.priority.as_deref();
    let changelog = args.changelog.as_deref();

    let mut changes = repo.list(status, priority, changelog, args.all);

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

    if changes.is_empty() {
        println!("No changes found.");
        return Ok(());
    }

    // Calculate unique prefixes for IDs (like jj)
    let ids: Vec<String> = changes.iter().map(|c| c.id.clone()).collect();
    let id_prefixes = calculate_unique_prefixes(&ids);

    // Create table
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
        let status_str = format_status(&change.status.to_string());
        let priority_str = format_priority(&change.priority.to_string());
        let changelog_str = change.changelog_type.as_ref()
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
        let prefix_len = id_prefixes.get(&change.id).copied().unwrap_or(change.id.len());
        let formatted_id = format_id_with_prefix(&change.id, prefix_len);

        table.add_row(vec![
            formatted_id,
            status_str,
            priority_str,
            changelog_str,
            age,
            title,
        ]);
    }

    table.print();

    Ok(())
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
fn calculate_unique_prefixes(ids: &[String]) -> std::collections::HashMap<String, usize> {
    let mut result = std::collections::HashMap::new();
    
    for id in ids {
        // Find the minimum prefix length that makes this ID unique
        let mut prefix_len = 1;
        'outer: while prefix_len <= id.len() {
            let prefix = &id[..prefix_len];
            
            // Check if this prefix is unique
            let conflicts: Vec<&String> = ids.iter()
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
