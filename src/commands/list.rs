use anyhow::{Context, Result};
use colored::Colorize;

use crate::cli::ListArgs;
use crate::config::get_db_path;
use crate::repository::ChangeRepository;

pub async fn list(args: ListArgs) -> Result<()> {
    let db_path = get_db_path()
        .context("Not in a pebbles repository. Run 'pebbles init' first.")?;
    
    let repo = ChangeRepository::open(db_path).await?;
    
    let status = args.status.as_deref();
    let priority = args.priority.as_deref();
    
    let mut changes = repo.list(status, priority, args.all);
    
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
    
    // Print header
    println!("{:<6} {:<12} {:<10} {:<8} {}",
        "ID".bold(),
        "Status".bold(),
        "Priority".bold(),
        "Age".bold(),
        "Title".bold()
    );
    
    // Print changes
    for change in changes {
        let status_str = format_status(&change.status.to_string());
        let priority_str = format_priority(&change.priority.to_string());
        let age = format_age(&change.created_at);
        
        // Truncate title if too long
        let title = if change.title.len() > 50 {
            format!("{}...", &change.title[..47])
        } else {
            change.title.clone()
        };
        
        println!("{:<6} {:<12} {:<10} {:<8} {}",
            change.id.cyan(),
            status_str,
            priority_str,
            age,
            title
        );
    }
    
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

fn priority_rank(priority: &crate::models::Priority) -> u8 {
    use crate::models::Priority;
    match priority {
        Priority::Critical => 0,
        Priority::High => 1,
        Priority::Medium => 2,
        Priority::Low => 3,
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
