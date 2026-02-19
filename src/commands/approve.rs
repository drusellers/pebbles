use crate::commands::print_success;
use crate::idish::IDish;
use crate::models::Status;
use crate::repository::ChangeRepository;
use anyhow::Result;

pub async fn approve(id: IDish) -> Result<()> {
    let mut repo = ChangeRepository::open().await?;

    let full_id = id.resolve(&repo.db)?;

    let change = repo.find_by_id(&full_id)
        .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", full_id))?;

    // Check current status
    if change.status != Status::Draft {
        anyhow::bail!(
            "Cannot approve change '{}': status is '{}' (must be 'draft')",
            full_id,
            change.status
        );
    }

    repo.update_status(&full_id, Status::Approved).await?;

    print_success(&format!("Approved change {} for work", full_id));

    Ok(())
}
