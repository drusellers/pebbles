use crate::commands::print_success;
use crate::idish::IDish;
use crate::models::{Event, EventType, Status};
use crate::repository::ChangeRepository;
use anyhow::Result;

pub async fn block(change_id: IDish, blocker_id: IDish) -> Result<()> {
    let mut repo = ChangeRepository::open().await?;

    // Resolve both IDs
    let full_change_id = change_id.resolve(&repo.db)?;
    let full_blocker_id = blocker_id.resolve(&repo.db)?;

    // Check that we're not trying to block ourselves
    if full_change_id == full_blocker_id {
        anyhow::bail!("A change cannot block itself");
    }

    // Verify both changes exist
    let change = repo
        .find_by_id(&full_change_id)
        .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", full_change_id))?
        .clone();

    let _blocker = repo
        .find_by_id(&full_blocker_id)
        .ok_or_else(|| anyhow::anyhow!("Blocker '{}' not found", full_blocker_id))?
        .clone();

    // Check if blocker already exists
    if change.is_blocked_by(&full_blocker_id) {
        anyhow::bail!(
            "Change '{}' is already blocked by '{}'",
            full_change_id,
            full_blocker_id
        );
    }

    // Add the blocker
    let change_mut = repo
        .find_by_id_mut(&full_change_id)
        .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", full_change_id))?;
    change_mut.add_blocker(full_blocker_id.clone());

    // Set status to Blocked (Tasuku-style: blocking means status=blocked)
    if change_mut.status != Status::Blocked {
        change_mut.update_status(Status::Blocked);
    }

    // Add event
    let event = Event::new(
        full_change_id.clone(),
        EventType::DependencyAdded,
        serde_json::json!({
            "blocker_id": full_blocker_id.to_string(),
        }),
    );
    repo.db.add_event(event);

    repo.save().await?;

    print_success(&format!(
        "Change '{}' is now blocked by '{}'",
        full_change_id, full_blocker_id
    ));

    Ok(())
}

pub async fn unblock(change_id: IDish, blocker_id: IDish) -> Result<()> {
    let mut repo = ChangeRepository::open().await?;

    // Resolve both IDs
    let full_change_id = change_id.resolve(&repo.db)?;
    let full_blocker_id = blocker_id.resolve(&repo.db)?;

    // Verify the change exists
    let change = repo
        .find_by_id(&full_change_id)
        .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", full_change_id))?
        .clone();

    // Check if blocker exists
    if !change.is_blocked_by(&full_blocker_id) {
        anyhow::bail!(
            "Change '{}' is not blocked by '{}'",
            full_change_id,
            full_blocker_id
        );
    }

    // Remove the blocker
    let change_mut = repo
        .find_by_id_mut(&full_change_id)
        .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", full_change_id))?;
    change_mut.remove_blocker(&full_blocker_id);

    // If no more blockers and status is Blocked, change status to Draft
    let still_blocked = !change_mut.blocked_by.is_empty();
    if !still_blocked && change_mut.status == Status::Blocked {
        change_mut.update_status(Status::Draft);
    }

    // Add event
    let event = Event::new(
        full_change_id.clone(),
        EventType::DependencyRemoved,
        serde_json::json!({
            "blocker_id": full_blocker_id.to_string(),
        }),
    );
    repo.db.add_event(event);

    repo.save().await?;

    print_success(&format!(
        "Removed blocker: '{}' is no longer blocked by '{}'",
        full_change_id, full_blocker_id
    ));

    Ok(())
}
