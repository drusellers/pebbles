use anyhow::{Context, Result};

use crate::commands::{print_success};
use crate::config::get_db_path;
use crate::vcs::detect_vcs;

pub async fn cleanup(id: Option<String>) -> Result<()> {
    use crate::commands::resolve_id;
    
    let db_path = get_db_path()
        .context("Not in a pebbles repository. Run 'pebbles init' first.")?;
    
    let repo = crate::repository::ChangeRepository::open(db_path).await?;
    
    let id = resolve_id(id)?;
    
    // Verify change exists
    let _change = repo.find_by_id(&id)
        .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", id))?;
    
    // Detect VCS
    let vcs = detect_vcs()
        .context("No version control system detected")?;
    
    // Clean up workspace
    vcs.cleanup_workspace(&id)?;
    
    print_success(&format!("Cleaned up workspace for change {}", id));
    
    Ok(())
}
