use crate::cli::UpdateArgs;
use crate::commands::{print_success, resolve_id};
use crate::id::Id;
use crate::idish::IDish;
use crate::models::{Event, EventType, Priority, Status};
use crate::repository::ChangeRepository;
use anyhow::{Context, Result};

pub async fn update(args: UpdateArgs) -> Result<()> {
    let full_id = resolve_id(args.id).await?;

    let mut repo = ChangeRepository::open().await?;

    // Track events to add later
    let mut events = Vec::new();
    let mut updated = false;

    // Update title
    if let Some(title) = args.title {
        let change = repo
            .find_by_id_mut(&full_id)
            .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", full_id))?;

        let old_title = change.title.clone();
        change.update_title(title);

        events.push(Event::new(
            full_id.clone(),
            EventType::Updated,
            serde_json::json!({
                "field": "title",
                "from": old_title,
                "to": change.title.clone(),
            }),
        ));

        updated = true;
    }

    // Update body (direct or from file)
    if let Some(body) = args.body {
        let body_content = if let Some(file_path) = body.strip_prefix('@') {
            tokio::fs::read_to_string(file_path)
                .await
                .with_context(|| format!("Failed to read body from file: {}", file_path))?
        } else {
            body
        };

        let change = repo
            .find_by_id_mut(&full_id)
            .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", full_id))?;

        change.update_body(body_content);

        events.push(Event::new(
            full_id.clone(),
            EventType::Updated,
            serde_json::json!({
                "field": "body",
            }),
        ));

        updated = true;
    }

    // Edit in editor
    if args.edit {
        let body = edit_in_editor(
            &repo.find_by_id(&full_id).unwrap().body,
            &crate::config::get_config_path()?,
        )
        .await?;

        let change = repo
            .find_by_id_mut(&full_id)
            .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", full_id))?;

        change.update_body(body);

        events.push(Event::new(
            full_id.clone(),
            EventType::Updated,
            serde_json::json!({
                "field": "body",
            }),
        ));

        updated = true;
    }

    // Update priority
    if let Some(priority_arg) = args.priority {
        let priority: Priority = priority_arg.into();

        let change = repo
            .find_by_id_mut(&full_id)
            .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", full_id))?;

        let old_priority = change.priority.clone();
        change.update_priority(priority.clone());

        events.push(Event::new(
            full_id.clone(),
            EventType::PriorityChanged,
            serde_json::json!({
                "from": old_priority.to_string(),
                "to": priority.to_string(),
            }),
        ));

        updated = true;
    }

    // Update status
    if let Some(status_str) = args.status {
        let new_status = Status::from_string(&status_str)
            .ok_or_else(|| anyhow::anyhow!("Invalid status: {}", status_str))?;

        repo.update_status(&full_id, new_status).await?;
        updated = true;
    }

    // Update changelog type
    if let Some(changelog_arg) = args.changelog {
        let changelog_type: crate::models::ChangelogType = changelog_arg.into();

        let change = repo
            .find_by_id_mut(&full_id)
            .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", full_id))?;

        let old_changelog = change.changelog_type.as_ref().map(|ct| ct.to_string());
        change.update_changelog_type(changelog_type.clone());

        events.push(Event::new(
            full_id.clone(),
            EventType::ChangelogTypeChanged,
            serde_json::json!({
                "from": old_changelog,
                "to": changelog_type.to_string(),
            }),
        ));

        updated = true;
    }

    // Update parent - handle this specially to avoid borrow checker issues
    if let Some(parent) = args.parent {
        updated = update_parent(&mut repo, &full_id, parent, &mut events).await? || updated;
    }

    // Add all events
    for event in events {
        repo.db.add_event(event);
    }

    if updated {
        repo.save().await?;
        print_success(&format!("Updated change {}", full_id));
    } else {
        println!("No changes to save");
    }

    Ok(())
}

/// Update the parent of a change
/// Returns true if a change was made
async fn update_parent(
    repo: &mut ChangeRepository,
    full_id: &Id,
    parent: IDish,
    events: &mut Vec<Event>,
) -> Result<bool> {
    // First, gather all the information we need without holding any borrows
    let old_parent = {
        let change = repo
            .find_by_id(full_id)
            .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", full_id))?;
        change.parent.clone()
    };

    if parent.as_str().is_empty() {
        // Remove parent (unparent the change)
        if old_parent.is_none() {
            return Ok(false); // No change needed
        }

        let old_parent_id = old_parent.clone().unwrap();

        // Remove from old parent's children list
        if let Some(old_parent_change) = repo.find_by_id_mut(&old_parent_id) {
            old_parent_change.children.retain(|id| id != full_id);
        }

        // Set parent to None
        if let Some(change) = repo.find_by_id_mut(full_id) {
            change.parent = None;
        }

        events.push(Event::new(
            full_id.clone(),
            EventType::ParentChanged,
            serde_json::json!({
                "from": old_parent,
                "to": null,
            }),
        ));

        Ok(true)
    } else {
        // Set new parent - resolve the IDish
        let new_parent_id = parent
            .resolve(&repo.db)
            .map_err(|e| anyhow::anyhow!("Invalid parent ID: {}", e))?;

        // Validate parent exists
        if repo.find_by_id(&new_parent_id).is_none() {
            anyhow::bail!("Parent change '{}' not found", new_parent_id);
        }

        // Prevent self-parenting
        if new_parent_id == *full_id {
            anyhow::bail!("Cannot set a change as its own parent");
        }

        // Check for circular reference (would create a cycle)
        if would_create_cycle(full_id, &new_parent_id, &repo.db)? {
            anyhow::bail!("Setting this parent would create a circular dependency");
        }

        // Check if parent is already set to this value
        if old_parent.as_ref() == Some(&new_parent_id) {
            return Ok(false); // No change needed
        }

        // Remove from old parent if exists
        if let Some(old_parent_id) = &old_parent
            && let Some(old_parent_change) = repo.find_by_id_mut(old_parent_id)
        {
            old_parent_change.children.retain(|id| id != full_id);
        }

        // Set new parent
        if let Some(change) = repo.find_by_id_mut(full_id) {
            change.parent = Some(new_parent_id.clone());
        }

        // Add to new parent's children list
        if let Some(new_parent) = repo.find_by_id_mut(&new_parent_id)
            && !new_parent.children.contains(full_id)
        {
            new_parent.children.push(full_id.clone());
        }

        events.push(Event::new(
            full_id.clone(),
            EventType::ParentChanged,
            serde_json::json!({
                "from": old_parent.as_ref().map(|id| id.to_string()),
                "to": new_parent_id.to_string(),
            }),
        ));

        Ok(true)
    }
}

async fn edit_in_editor(initial: &str, config_path: &std::path::Path) -> Result<String> {
    use crate::config::Config;

    let config = Config::load(config_path).await?;
    let editor = config.get_editor();

    let temp_file = tempfile::NamedTempFile::new()?;
    let temp_path = temp_file.path().to_path_buf();

    tokio::fs::write(&temp_path, initial).await?;

    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("{} {}", editor, temp_path.display()))
        .status()
        .context("Failed to launch editor")?;

    if !status.success() {
        anyhow::bail!("Editor exited with non-zero status");
    }

    tokio::fs::read_to_string(&temp_path)
        .await
        .context("Failed to read edited file")
}

/// Check if setting `child_id`'s parent to `parent_id` would create a cycle
fn would_create_cycle(child_id: &Id, parent_id: &Id, db: &crate::db::Db) -> Result<bool> {
    use std::collections::HashSet;

    let mut visited = HashSet::new();
    let mut current = parent_id.clone();

    // Walk up the parent chain from the proposed parent
    // If we encounter child_id, it would create a cycle
    loop {
        if current == *child_id {
            return Ok(true); // Cycle detected
        }

        if !visited.insert(current.clone()) {
            // Already visited this node - shouldn't happen with proper data
            break;
        }

        // Get the parent's parent
        match db.data.changes.get(&current) {
            Some(change) => {
                match &change.parent {
                    Some(grandparent) => current = grandparent.clone(),
                    None => break, // Reached root
                }
            }
            None => break, // Parent not found
        }
    }

    Ok(false) // No cycle
}
