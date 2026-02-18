use anyhow::{Context, Result};
use std::process::Command;

use crate::commands::{print_info, print_success};
use crate::config::{get_config_path, get_db_path, Config};
use crate::db::Db;
use crate::idish::IDish;
use crate::models::Status;
use crate::repository::ChangeRepository;
use crate::vcs::detect_vcs_with_preference;

pub async fn build(id: IDish, skip_permissions: bool) -> Result<()> {
    let db_path = get_db_path()
        .context("Not in a pebbles repository. Run 'pebbles init' first.")?;

    // Resolve IDish to full ID first
    let db = Db::open(&db_path).await?;
    let full_id = id.resolve(&db).map_err(|e| anyhow::anyhow!(e))?;

    let mut repo = ChangeRepository::open(db_path).await?;

    let change = repo.find_by_id(&full_id)
        .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", full_id))?;

    // Check status
    match change.status {
        Status::Draft => {
            anyhow::bail!(
                "Change '{}' is in Draft status. Approve it first with 'pebbles approve {}'",
                full_id, full_id
            );
        }
        Status::Approved | Status::InProgress => {}
        Status::Done => {
            anyhow::bail!("Change '{}' is already done", full_id);
        }
        _ => {}
    }

    // Detect VCS (for setting environment variable)
    let config_path = get_config_path().unwrap();
    let config = Config::load(&config_path).await?;

    let vcs = detect_vcs_with_preference(&config.vcs.prefer)
        .context("No version control system detected (git or jujutsu)")?;

    print_info(&format!("Using {} for version control", vcs.name()));

    // Update status to InProgress
    if change.status != Status::InProgress {
        repo.update_status(&full_id, Status::InProgress).await?;
        print_info("Updated status to InProgress");
    }

    // Launch opencode in current directory (no workspace created)
    print_info(&format!("Launching opencode to work on change '{}' in current directory", full_id));

    let mut cmd = Command::new("opencode");
    cmd.env("PEBBLES_CHANGE", &full_id);
    cmd.env("PEBBLES_VCS", vcs.name());

    if skip_permissions {
        cmd.env("OPENCODE_SKIP_PERMISSIONS", "1");
    }

    // Use exec on Unix to replace process
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let _ = cmd.exec();
        // If we get here, exec failed
        anyhow::bail!("Failed to launch opencode");
    }

    // On Windows, spawn and wait
    #[cfg(not(unix))]
    {
        let mut child = cmd.spawn()
            .context("Failed to launch opencode. Is it installed?")?;
        child.wait()?;
        Ok(())
    }
}
