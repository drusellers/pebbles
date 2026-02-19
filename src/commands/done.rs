/// Marks a change as completed (done).
///
/// This command updates the status of a change to Done in the pebbles database.
/// It works regardless of whether the change was started with or without --isolate:
/// - Without isolate: Changes status and optionally generates a commit message
/// - With isolate: Same behavior, commit message generation may differ based on VCS
///
/// If a VCS (Git or Jujutsu) is detected in the current directory, the command will
/// also generate a proposed commit message based on the change's title and acceptance
/// criteria. This is purely informational and does not modify any VCS state.
///
/// # Arguments
/// - `id`: Optional change ID. If not provided, attempts to detect from environment
/// - `auto`: If true, checks that all acceptance criteria are completed before marking done
/// - `force`: If true with --auto, bypasses acceptance criteria check
use crate::commands::{print_info, print_success, resolve_id};
use crate::idish::IDish;
use crate::models::Status;
use crate::repository::ChangeRepository;
use crate::vcs::detect_vcs;
use anyhow::Result;

pub async fn done(id: Option<IDish>, auto: bool, force: bool) -> Result<()> {
    let full_id = resolve_id(id).await?;

    let mut repo = ChangeRepository::open().await?;

    let change = repo.find_by_id(&full_id)
        .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", full_id))?;

    // Clone data we need later
    let title = change.title.clone();
    let body = change.body.clone();

    // Check if already done
    if change.status == Status::Done {
        println!("Change '{}' is already marked as done", full_id);
        return Ok(());
    }

    // Check acceptance criteria if --auto flag
    if auto {
        let all_checked = check_acceptance_criteria(&body);
        if !all_checked && !force {
            anyhow::bail!(
                "Not all acceptance criteria are checked. Use --force to override.\n\
                 Run 'pebbles show {}' to see unchecked items.",
                full_id
            );
        }
    }

    // Update status in the database - this always succeeds regardless of VCS/worktree
    repo.update_status(&full_id, Status::Done).await?;

    print_success(&format!("Marked change {} as done", full_id));

    // Try to generate a commit message if VCS is available
    // This is optional and works whether or not --isolate was used with 'start'
    // The commit message is just displayed, not applied automatically
    if let Some(vcs) = detect_vcs() {
        print_info("Generating commit message...");
        let msg = vcs.generate_commit_msg(&title, &body)?;
        println!("\nProposed commit message:\n{}", msg);

        // Note: Actual commit happens outside pebbles via opencode's describe command
        print_info("Use 'opencode /describe' to generate and apply commit message");
    }

    Ok(())
}

fn check_acceptance_criteria(body: &str) -> bool {
    // Look for unchecked items in acceptance criteria section
    let mut in_acceptance_criteria = false;

    for line in body.lines() {
        let trimmed = line.trim();

        // Check for acceptance criteria header
        if trimmed.to_lowercase().contains("acceptance criteria") {
            in_acceptance_criteria = true;
            continue;
        }

        // Exit if we hit another section
        if in_acceptance_criteria && trimmed.starts_with("##") {
            break;
        }

        // Check for unchecked items
        if in_acceptance_criteria && trimmed.starts_with("- [ ]") {
            return false;
        }
    }

    true
}
