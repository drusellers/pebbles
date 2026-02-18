use anyhow::{Context, Result};

use crate::commands::{print_success};
use crate::config::get_db_path;
use crate::models::Status;
use crate::repository::ChangeRepository;

pub async fn approve(id: String) -> Result<()> {
    let db_path = get_db_path()
        .context("Not in a pebbles repository. Run 'pebbles init' first.")?;
    
    let mut repo = ChangeRepository::open(db_path).await?;
    
    let change = repo.find_by_id(&id)
        .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", id))?;
    
    // Check current status
    if change.status != Status::Draft {
        anyhow::bail!(
            "Cannot approve change '{}': status is '{}' (must be 'draft')",
            id,
            change.status.to_string()
        );
    }
    
    repo.update_status(&id, Status::Approved).await?;
    
    print_success(&format!("Approved change {} for work", id));
    
    Ok(())
}
