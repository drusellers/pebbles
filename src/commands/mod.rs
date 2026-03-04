use crate::id::Id;
use crate::idish::IDish;
use anyhow::Result;
use colored::Colorize;

pub mod approve;
pub mod block;
pub mod cleanup;
pub mod completions;
pub mod current;
pub mod delete;
pub mod doctor;
pub mod done;
pub mod edit;
pub mod init;
pub mod intake;
pub mod list;
pub mod log;
pub mod migrate;
pub mod new;
pub mod plan;
pub mod ready;
pub mod show;
pub mod start;
pub mod status;
pub mod timer;
pub mod update;

pub use approve::approve;
pub use block::{block, unblock};
pub use cleanup::cleanup;
pub use completions::completions;
pub use current::current;
pub use delete::delete;
pub use doctor::doctor;
pub use done::done;
pub use edit::edit;
pub use init::init;
pub use intake::intake;
pub use list::list;
pub use log::log;
pub use migrate::migrate;
pub use new::new;
pub use plan::plan;
pub use ready::ready;
pub use show::show;
pub use start::start;
pub use status::status;
pub use timer::{timer_start, timer_stop, timer_status};
pub use update::update;

/// Resolve change ID from either explicit argument or current workspace.
/// This function handles the case where the user provides an IDish or is in a workspace.
pub async fn resolve_id(id: Option<IDish>) -> Result<Id> {
    use crate::db::Db;

    match id {
        Some(idish) => {
            let db_path = crate::config::get_db_path()?;
            let db = Db::open(&db_path).await?;
            idish.resolve(&db)
        }
        None => {
            // Try to detect current workspace
            if let Some(vcs) = crate::vcs::detect_vcs()
                && let Some(current_id) = vcs.current_workspace_id()
            {
                return Ok(current_id);
            }

            // Build a helpful error message
            let mut msg = String::from("No change ID provided and not in a workspace.");

            // Check if we're in a pebbles-enabled repository
            if crate::config::get_db_path().is_ok() {
                msg.push_str("\n\nYou are in a pebbles-enabled repository.");

                // List available workspaces (ws-* directories)
                if let Ok(root) = crate::config::find_pebbles_root() {
                    let workspaces: Vec<String> = std::fs::read_dir(&root)
                        .ok()
                        .into_iter()
                        .flatten()
                        .filter_map(|e| e.ok())
                        .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
                        .filter(|name| name.starts_with("ws-"))
                        .map(|name| name[3..].to_string())
                        .collect();

                    if !workspaces.is_empty() {
                        msg.push_str("\n\nAvailable workspaces:\n");
                        for ws in &workspaces {
                            msg.push_str(&format!("  ws-{ws}\n"));
                        }
                        msg.push_str(
                            "\nRun 'pebbles start --isolate <id>' to create a new workspace.",
                        );
                    } else {
                        msg.push_str("\n\nNo workspaces found. Run 'pebbles start --isolate <id>' to create one.");
                    }
                }
            } else {
                msg.push_str("\n\nRun 'pebbles init' to initialize a pebbles repository.");
            }

            anyhow::bail!("{}", msg)
        }
    }
}

fn print_success(msg: &str) {
    println!("{} {}", "✓".green(), msg);
}

fn print_info(msg: &str) {
    println!("{} {}", "→".blue(), msg);
}

/// Calculate the shortest unique prefix length for an ID among all other IDs.
/// Returns the minimum number of characters needed to uniquely identify this ID.
pub fn unique_prefix_len(id: &str, all_ids: &[&str]) -> usize {
    if all_ids.len() <= 1 {
        return 1;
    }

    // Collect all other IDs (excluding the target ID itself)
    let other_ids: Vec<&str> = all_ids
        .iter()
        .copied()
        .filter(|&other_id| other_id != id)
        .collect();

    // Find the minimum prefix length that makes this ID unique
    for prefix_len in 1..=id.len() {
        let prefix = &id[..prefix_len];
        let is_unique = other_ids.iter().all(|other_id| {
            if other_id.len() >= prefix_len {
                &other_id[..prefix_len] != prefix
            } else {
                true // Shorter IDs can't match a longer prefix
            }
        });

        if is_unique {
            return prefix_len;
        }
    }

    // If no unique prefix found, return the full length
    id.len()
}

/// Format a change ID with the unique prefix highlighted.
/// The unique prefix is shown in bold cyan, the rest is dimmed.
pub fn format_id_with_unique_prefix(id: &str, all_ids: &[&str]) -> String {
    let prefix_len = unique_prefix_len(id, all_ids);
    let prefix = &id[..prefix_len];
    let suffix = &id[prefix_len..];

    if suffix.is_empty() {
        prefix.cyan().bold().to_string()
    } else {
        format!("{}{}", prefix.cyan().bold(), suffix.dimmed())
    }
}
