use crate::commands::{print_success, resolve_id};
use crate::config::get_db_path;
use crate::db::Db;
use crate::idish::IDish;
use crate::vcs::detect_vcs;
use anyhow::{Context, Result};

pub async fn cleanup(id: Option<IDish>) -> Result<()> {
    let db_path = get_db_path()
        .context("Not in a pebbles repository. Run 'pebbles init' first.")?;

    // Handle ID resolution first
    let full_id = if let Some(id) = id {
        // Resolve ID to full ID using the db directly
        let db = Db::open(&db_path).await?;
        id.resolve(&db).map_err(|e| anyhow::anyhow!(e))?
    } else {
        // Use workspace detection
        resolve_id(None).await?
    };

    let repo = crate::repository::ChangeRepository::open(db_path).await?;

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
