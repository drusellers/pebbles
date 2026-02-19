use crate::commands::{print_success, resolve_id};
use crate::idish::IDish;
use crate::repository::ChangeRepository;
use crate::vcs::detect_vcs;
use anyhow::{Context, Result};

pub async fn cleanup(id: Option<IDish>) -> Result<()> {
    let full_id = resolve_id(id).await?;

    let repo = ChangeRepository::open().await?;

    // Verify change exists
    let _change = repo.find_by_id(&full_id)
        .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", full_id))?;

    // Detect VCS
    let vcs = detect_vcs()
        .context("No version control system detected")?;

    // Clean up workspace
    vcs.cleanup_workspace(&full_id)?;

    print_success(&format!("Cleaned up workspace for change {}", full_id));

    Ok(())
}
