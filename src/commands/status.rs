use crate::commands::format_id_with_unique_prefix;
use crate::repository::ChangeRepository;
use crate::vcs::detect_vcs;
use anyhow::{Context, Result};
use colored::Colorize;

pub async fn status() -> Result<()> {
    // Check if we're in a pebbles-enabled repository first
    if crate::config::get_db_path().is_err() {
        print_not_initialized();
        std::process::exit(1);
    }

    let vcs = detect_vcs().context("No version control system detected")?;
    let current_id = vcs.current_workspace_id();

    // Get workspace directory path
    let current_dir = std::env::current_dir().ok();
    let workspace_path = current_dir
        .as_ref()
        .and_then(|p| {
            p.file_name()
                .and_then(|n| n.to_str().map(|s| s.to_string()))
        })
        .unwrap_or_else(|| "unknown".to_string());

    if let Some(id) = current_id {
        // In a workspace - show full details
        let repo = ChangeRepository::open().await?;
        let change = repo.find_by_id(&id);

        // Get all change IDs to calculate unique prefix
        let all_changes = repo.list(None, None, None, true);
        let all_ids: Vec<&str> = all_changes.iter().map(|c| c.id.as_str()).collect();

        // Display status information
        println!("Workspace: {}/", workspace_path.cyan());

        match change {
            Some(change) => {
                println!(
                    "Change: {} - {}",
                    format_id_with_unique_prefix(change.id.as_str(), &all_ids),
                    change.title.white().bold()
                );
                println!("Status: {}", format_status(&change.status.to_string()));
            }
            None => {
                // Change ID exists in workspace but not in database
                println!("Change: {}", id.as_str().cyan());
                println!("Status: {}", "not found".red());
            }
        }

        println!("VCS: {}", vcs.name());
    } else {
        // Not in a workspace - show repository context
        println!("{} Not in a pebbles workspace.", "!".yellow().bold());

        if let Ok(root) = crate::config::find_pebbles_root() {
            println!("\nRepository: {}", root.display().to_string().cyan());
        }

        println!("VCS: {}", vcs.name());

        // Show available workspaces
        if let Ok(root) = crate::config::find_pebbles_root() {
            let workspaces_dir = root.join("workspaces");
            let workspaces: Vec<String> = std::fs::read_dir(&workspaces_dir)
                .ok()
                .into_iter()
                .flatten()
                .filter_map(|e| e.ok())
                .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
                .filter(|name| name.starts_with("ws-"))
                .map(|name| name[3..].to_string())
                .collect();

            if !workspaces.is_empty() {
                println!("\nAvailable workspaces:");

                // Get repo to fetch change titles
                let repo = ChangeRepository::open().await.ok();

                for ws_id in workspaces {
                    if let Some(ref r) = repo {
                        if let Ok(id) = ws_id.parse::<crate::id::Id>() {
                            if let Some(change) = r.find_by_id(&id) {
                                println!("  ws-{} - {}", ws_id.cyan(), change.title.dimmed());
                            } else {
                                println!("  ws-{}", ws_id.cyan());
                            }
                        } else {
                            println!("  ws-{}", ws_id.cyan());
                        }
                    } else {
                        println!("  ws-{}", ws_id.cyan());
                    }
                }

                println!(
                    "\nRun '{}' to enter a workspace.",
                    "pebbles work <id>".to_string().cyan()
                );
            } else {
                println!("\n{} No workspaces found.", "!".yellow());
                println!(
                    "Run '{}' to create a workspace.",
                    "pebbles start --isolate <id>".to_string().cyan()
                );
            }
        }
    }

    Ok(())
}

fn print_not_initialized() {
    eprintln!(
        "{} Not in a pebbles-enabled repository.",
        "Error:".red().bold()
    );
    eprintln!();
    eprintln!(
        "Run '{}' to initialize a pebbles repository.",
        "pebbles init".cyan()
    );
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
