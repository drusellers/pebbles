/// Marks a change as completed (done).
///
/// This command updates the status of a change to Done in the pebbles database.
/// It works regardless of whether the change was started with or without --isolate.
///
/// When `--auto` is used:
/// - Checks that all acceptance criteria are completed (unless `--force` is used)
/// - Generates a commit message via the AI harness
/// - Commits the change via the detected VCS (Git or Jujutsu)
///
/// # Arguments
/// - `id`: Optional change ID. If not provided, attempts to detect from environment
/// - `auto`: If true, checks acceptance criteria and commits via VCS (requires harness and VCS)
/// - `force`: If true with --auto, bypasses acceptance criteria check
use crate::commands::{print_info, print_success, resolve_id};
use crate::harness::detect_harness;
use crate::idish::IDish;
use crate::models::Status;
use crate::repository::ChangeRepository;
use crate::vcs::detect_vcs_with_preference;
use anyhow::{anyhow, Context, Result};

pub async fn done(id: Option<IDish>, auto: bool, force: bool) -> Result<()> {

    if auto {
        detect_harness()
            .ok_or_else(|| anyhow!("No AI harness detected. --auto requires a harness to generate commit messages."))?;

        detect_vcs_with_preference()
            .await?
            .context("No version control system detected. --auto requires git or jujutsu.")?;
    }

    let full_id = resolve_id(id).await?;

    let mut repo = ChangeRepository::open().await?;

    let change = repo.find_by_id(&full_id)
        .ok_or_else(|| anyhow!("Change '{}' not found", full_id))?;

    // Check if already done
    if change.status == Status::Done {
        println!("Change '{}' is already marked as done", full_id);
        return Ok(());
    }

    // Check acceptance criteria if --auto flag
    if auto {
        let all_checked = change.check_acceptance_criteria();
        if !all_checked && !force {
            anyhow::bail!(
                "Not all acceptance criteria are checked. Use --force to override.\n\
                 Run 'pebbles show {}' to see unchecked items.",
                full_id
            );
        }
    }

    // For --auto mode: get harness and vcs (already validated above)
    let auto_commit = if auto {
        let harness = detect_harness().expect("harness validated above");
        let vcs = detect_vcs_with_preference()
            .await
            .expect("vcs detection should not fail")
            .expect("vcs should be present (validated above)");
        Some((harness, vcs))
    } else {
        None
    };

    // Update status in the database - this always succeeds regardless of VCS/worktree
    repo.update_status(&full_id, Status::Done).await?;

    print_success(&format!("Marked change {} as done", full_id));

    // For --auto mode: generate commit message and commit via VCS
    if let Some((harness, vcs)) = auto_commit {
        print_info(&format!("Using {} for version control", vcs.name()));
        print_info("Generating commit message...");
        let msg = harness.generate_commit_msg(&full_id)?;
        vcs.commit(&msg)?;
        print_success("Committed changes");
    }

    Ok(())
}
