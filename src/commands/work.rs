use anyhow::{Context, Result};
use std::process::Command;

use crate::commands::{print_info, print_success};
use crate::config::{get_config_path, get_db_path, Config};
use crate::models::Status;
use crate::repository::ChangeRepository;
use crate::vcs::detect_vcs_with_preference;

pub async fn work(id: String, skip_permissions: bool) -> Result<()> {
    let db_path = get_db_path()
        .context("Not in a pebbles repository. Run 'pebbles init' first.")?;

    let mut repo = ChangeRepository::open(db_path).await?;

    let change = repo.find_by_id(&id)
        .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", id))?;

    // Check status
    match change.status {
        Status::Draft => {
            anyhow::bail!(
                "Change '{}' is in Draft status. Approve it first with 'pebbles approve {}'",
                id, id
            );
        }
        Status::Approved | Status::InProgress => {}
        Status::Done => {
            anyhow::bail!("Change '{}' is already done", id);
        }
        _ => {}
    }
    
    // Detect VCS
    let config_path = get_config_path().unwrap();
    let config = Config::load(&config_path).await?;
    
    let vcs = detect_vcs_with_preference(&config.vcs.prefer)
        .context("No version control system detected (git or jujutsu)")?;
    
    print_info(&format!("Using {} for version control", vcs.name()));
    
    // Create workspace
    let workspace_path = vcs.create_workspace(&id)?;
    print_success(&format!("Created workspace at {}", workspace_path.display()));
    
    // Update status to InProgress
    if change.status != Status::InProgress {
        repo.update_status(&id, Status::InProgress).await?;
        print_info("Updated status to InProgress");
    }
    
    // Launch opencode
    print_info("Launching opencode...");
    
    let mut cmd = Command::new("opencode");
    cmd.current_dir(&workspace_path);
    cmd.env("PEBBLES_CHANGE", &id);
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
