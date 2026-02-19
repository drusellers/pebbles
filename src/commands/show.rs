use anyhow::{Context, Result};
use colored::Colorize;

use crate::commands::{resolve_id, format_id_with_unique_prefix};
use crate::config::get_db_path;
use crate::db::Db;
use crate::idish::IDish;
use crate::repository::ChangeRepository;

pub async fn show(id: Option<IDish>) -> Result<()> {
    let db_path = get_db_path()
        .context("Not in a pebbles repository. Run 'pebbles init' first.")?;

    // Handle ID resolution first
    let full_id = if let Some(id) = id {
        // Resolve IDish to full ID using the db directly
        let db = Db::open(&db_path).await?;
        id.resolve(&db).map_err(|e| anyhow::anyhow!(e))?
    } else {
        // Use workspace detection
        resolve_id(None)?
    };

    let repo = ChangeRepository::open(db_path).await?;

    let change = repo.find_by_id(&full_id)
        .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", full_id))?;

    // Get all change IDs to calculate unique prefix
    let all_changes = repo.list(None, None, None, true);
    let all_ids: Vec<&str> = all_changes.iter()
        .map(|c| c.id.as_str())
        .collect();

    // Print header
    println!("\n{}", "═".repeat(60).dimmed());
    println!("{} {} {}",
        format_id_with_unique_prefix(&change.id, &all_ids),
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
        println!("  {}", parent.cyan());
    }

    if !change.children.is_empty() {
        println!("\n{}", "Children:".bold());
        for child in &change.children {
            println!("  {}", child.cyan());
        }
    }

    if !change.dependencies.is_empty() {
        println!("\n{}", "Dependencies:".bold());
        for dep in &change.dependencies {
            println!("  {}", dep.cyan());
        }
    }

    if !change.tags.is_empty() {
        println!("\n{}", "Tags:".bold());
        println!("  {}", change.tags.join(", "));
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
