use crate::commands::print_success;
use crate::config::get_db_path;
use crate::db::Db;
use crate::idish::IDish;
use crate::models::Event;
use crate::models::EventType;
use crate::repository::ChangeRepository;
use anyhow::{Context, Result};

pub async fn block(change_id: IDish, dependency_id: IDish) -> Result<()> {
    let db_path = get_db_path()
        .context("Not in a pebbles repository. Run 'pebbles init' first.")?;

    // Resolve both IDs
    let db = Db::open(&db_path).await?;
    let full_change_id = change_id.resolve(&db).map_err(|e| anyhow::anyhow!(e))?;
    let full_dependency_id = dependency_id.resolve(&db).map_err(|e| anyhow::anyhow!(e))?;

    // Check that we're not trying to block ourselves
    if full_change_id == full_dependency_id {
        anyhow::bail!("A change cannot block itself");
    }

    let mut repo = ChangeRepository::open(db_path).await?;

    // Verify both changes exist
    let change = repo
        .find_by_id(&full_change_id)
        .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", full_change_id))?
        .clone();

    let _dependency = repo
        .find_by_id(&full_dependency_id)
        .ok_or_else(|| anyhow::anyhow!("Dependency '{}' not found", full_dependency_id))?
        .clone();

    // Check if dependency already exists
    if change.dependencies.contains(&full_dependency_id) {
        anyhow::bail!(
            "Change '{}' already depends on '{}'",
            full_change_id,
            full_dependency_id
        );
    }

    // Add the dependency
    let change_mut = repo
        .find_by_id_mut(&full_change_id)
        .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", full_change_id))?;
    change_mut.add_dependency(full_dependency_id.clone());

    // Add event
    let event = Event::new(
        full_change_id.clone(),
        EventType::DependencyAdded,
        serde_json::json!({
            "dependency_id": full_dependency_id.to_string(),
        }),
    );
    repo.db.add_event(event);

    repo.save().await?;

    print_success(&format!(
        "Change '{}' now depends on '{}'",
        full_change_id, full_dependency_id
    ));

    Ok(())
}

pub async fn unblock(change_id: IDish, dependency_id: IDish) -> Result<()> {
    let db_path = get_db_path()
        .context("Not in a pebbles repository. Run 'pebbles init' first.")?;

    // Resolve both IDs
    let db = Db::open(&db_path).await?;
    let full_change_id = change_id.resolve(&db).map_err(|e| anyhow::anyhow!(e))?;
    let full_dependency_id = dependency_id.resolve(&db).map_err(|e| anyhow::anyhow!(e))?;

    let mut repo = ChangeRepository::open(db_path).await?;

    // Verify the change exists
    let change = repo
        .find_by_id(&full_change_id)
        .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", full_change_id))?
        .clone();

    // Check if dependency exists
    if !change.dependencies.contains(&full_dependency_id) {
        anyhow::bail!(
            "Change '{}' does not depend on '{}'",
            full_change_id,
            full_dependency_id
        );
    }

    // Remove the dependency
    let change_mut = repo
        .find_by_id_mut(&full_change_id)
        .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", full_change_id))?;
    change_mut.remove_dependency(&full_dependency_id);

    // Add event
    let event = Event::new(
        full_change_id.clone(),
        EventType::DependencyRemoved,
        serde_json::json!({
            "dependency_id": full_dependency_id.to_string(),
        }),
    );
    repo.db.add_event(event);

    repo.save().await?;

    print_success(&format!(
        "Removed dependency: '{}' no longer depends on '{}'",
        full_change_id, full_dependency_id
    ));

    Ok(())
}
