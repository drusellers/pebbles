use crate::commands::format_id_with_unique_prefix;
use crate::models::Status;
use crate::repository::ChangeRepository;
use anyhow::Result;
use colored::Colorize;

pub async fn ready() -> Result<()> {
    let repo = ChangeRepository::open().await?;

    let ready_changes = repo.list_ready();

    if ready_changes.is_empty() {
        println!("No changes are ready to work on.");
        println!();
        println!("Changes may be:");
        println!("  • Blocked by other changes (check with {})", "pebbles show <id>".cyan());
        println!("  • Already done");
        println!("  • In Draft status (needs approval with {})", "pebbles approve".cyan());
        return Ok(());
    }

    println!("\n{}", "Ready Changes".bold());
    println!("{}", "─".repeat(60).dimmed());

    // Get all IDs to calculate unique prefixes
    let all_changes = repo.list(None, None, None, true);
    let all_ids: Vec<&str> = all_changes.iter().map(|c| c.id.as_str()).collect();

    for change in ready_changes {
        let id_display = format_id_with_unique_prefix(change.id.as_str(), &all_ids);
        let status_display = match change.status {
            Status::Draft => "draft".dimmed(),
            Status::Approved => "approved".yellow(),
            Status::InProgress => "in_progress".blue().bold(),
            _ => change.status.to_string().normal(),
        };

        println!(
            "  {} {} {} {}",
            id_display,
            "─".dimmed(),
            status_display,
            change.title.white()
        );
    }

    println!();
    println!(
        "Start working: {}",
        format!("pebbles start <id>",).cyan()
    );

    Ok(())
}
