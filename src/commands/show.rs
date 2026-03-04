use crate::commands::{format_id_with_unique_prefix, resolve_id};
use crate::idish::IDish;
use crate::repository::ChangeRepository;
use anyhow::Result;
use colored::Colorize;

pub async fn show(id: Option<IDish>) -> Result<()> {
    let repo = ChangeRepository::open().await?;

    let full_id = match id {
        Some(idish) => match idish.resolve(&repo.db) {
            Ok(id) => id,
            Err(err) => {
                if let Some(invalid_id) = repo.resolve_invalid_idish(&idish)?
                    && let Some(invalid) = repo.invalid_change_by_id(&invalid_id)
                {
                    println!("\n{}", "Invalid change file".red().bold());
                    println!("  id: {}", invalid.id.to_string().cyan().bold());
                    println!("  file: {}", invalid.path.display());
                    println!("  reason: {}", invalid.error.red());
                    println!(
                        "  fix: correct the markdown/frontmatter and re-run `pebbles show {}`",
                        invalid.id
                    );
                    println!();
                    return Ok(());
                }
                return Err(err);
            }
        },
        None => resolve_id(None).await?,
    };

    let change = repo.find_by_id(&full_id).ok_or_else(|| {
        if let Some(invalid) = repo.invalid_change_by_id(&full_id) {
            anyhow::anyhow!(
                "Change '{}' has invalid markdown at {}: {}",
                full_id,
                invalid.path.display(),
                invalid.error
            )
        } else {
            anyhow::anyhow!("Change '{}' not found", full_id)
        }
    })?;

    // Get all change IDs to calculate unique prefix
    let all_changes = repo.list(None, None, None, true);
    let all_ids: Vec<&str> = all_changes.iter().map(|c| c.id.as_str()).collect();

    // Print header
    println!("\n{}", "═".repeat(60).dimmed());
    println!(
        "{} {} {}",
        format_id_with_unique_prefix(change.id.as_str(), &all_ids),
        "─".dimmed(),
        change.title.white().bold()
    );
    println!("{}", "═".repeat(60).dimmed());

    // Print metadata
    println!("\n{}", "Status:".bold());
    println!("  {}", format_status(&change.status.to_string()));

    println!("\n{}", "Priority:".bold());
    println!("  {}", format_priority(&change.priority.to_string()));

    if let Some(ref changelog) = change.changelog_type {
        println!("\n{}", "Changelog:".bold());
        println!("  {}", format_changelog(&changelog.to_string()));
    }

    if let Some(ref parent) = change.parent {
        println!("\n{}", "Parent:".bold());
        println!("  {}", parent.as_str().cyan());
    }

    if !change.blocked_by.is_empty() {
        println!("\n{}", "Blocked by:".bold());
        for blocker in &change.blocked_by {
            println!("  {}", blocker.as_str().cyan());
        }
    }

    if !change.tags.is_empty() {
        println!("\n{}", "Tags:".bold());
        println!("  {}", change.tags.join(", "));
    }

    // Print timer info
    if change.accumulated_duration_secs > 0 || change.is_timer_running() {
        println!("\n{}", "Time:".bold());
        if change.is_timer_running() {
            println!(
                "  {} (running)",
                format_duration(change.total_duration_secs()).green().bold()
            );
        } else {
            println!("  {}", format_duration(change.accumulated_duration_secs).cyan());
        }
    }

    // Print dates
    println!("\n{}", "Created:".bold());
    println!("  {}", change.created_at.format("%Y-%m-%d %H:%M:%S UTC"));

    println!("\n{}", "Updated:".bold());
    println!("  {}", change.updated_at.format("%Y-%m-%d %H:%M:%S UTC"));

    // Print body
    if !change.body.is_empty() {
        println!("\n{}", "═".repeat(60).dimmed());
        println!("{}", change.body);
    }

    println!("{}", "═".repeat(60).dimmed());
    println!();

    Ok(())
}

fn format_status(status: &str) -> String {
    match status {
        "draft" => status.dimmed().to_string(),
        "approved" => status.yellow().to_string(),
        "in_progress" => status.blue().to_string(),
        "review" => status.magenta().to_string(),
        "done" => status.green().to_string(),
        "blocked" => status.red().to_string(),
        "paused" => status.dimmed().to_string(),
        _ => status.to_string(),
    }
}

fn format_priority(priority: &str) -> String {
    match priority {
        "low" => priority.dimmed().to_string(),
        "medium" => priority.to_string(),
        "high" => priority.yellow().to_string(),
        "critical" => priority.red().bold().to_string(),
        _ => priority.to_string(),
    }
}

fn format_changelog(changelog: &str) -> String {
    match changelog {
        "feature" => "feature".green().bold().to_string(),
        "fix" => "fix".red().to_string(),
        "change" => "change".yellow().to_string(),
        "deprecated" => "deprecated".dimmed().to_string(),
        "removed" => "removed".red().bold().to_string(),
        "security" => "security".red().bold().to_string(),
        "internal" => "internal".dimmed().to_string(),
        _ => changelog.to_string(),
    }
}

fn format_duration(total_secs: i64) -> String {
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}
