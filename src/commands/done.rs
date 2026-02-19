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

    // Update status
    repo.update_status(&full_id, Status::Done).await?;

    print_success(&format!("Marked change {} as done", full_id));

    // Try to generate commit message
    if let Some(vcs) = detect_vcs() {
        print_info("Generating commit message...");
        let msg = vcs.generate_commit_msg(&title, &body)?;
        println!("\nProposed commit message:\n{}", msg);

        // Note: Actual commit happens outside docket via opencode's describe command
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
