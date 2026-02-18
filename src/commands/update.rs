use anyhow::{Context, Result};

use crate::cli::UpdateArgs;
use crate::commands::{print_success, resolve_id};
use crate::config::get_db_path;
use crate::models::{Event, EventType, Priority, Status};
use crate::repository::ChangeRepository;

pub async fn update(args: UpdateArgs) -> Result<()> {
    let db_path = get_db_path()
        .context("Not in a pebbles repository. Run 'pebbles init' first.")?;
    
    let mut repo = ChangeRepository::open(db_path).await?;
    
    let id = resolve_id(args.id)?;
    
    // Track events to add later
    let mut events = Vec::new();
    let mut updated = false;
    
    // Update title
    if let Some(title) = args.title {
        let change = repo.find_by_id_mut(&id)
            .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", id))?;
        
        let old_title = change.title.clone();
        change.update_title(title);
        
        events.push(Event::new(
            id.clone(),
            EventType::Updated,
            serde_json::json!({
                "field": "title",
                "from": old_title,
                "to": change.title.clone(),
            }),
        ));
        
        updated = true;
    }
    
    // Update body (direct)
    if let Some(body) = args.body {
        let change = repo.find_by_id_mut(&id)
            .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", id))?;
        
        change.update_body(body);
        
        events.push(Event::new(
            id.clone(),
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
            &repo.find_by_id(&id).unwrap().body,
            &crate::config::get_config_path().unwrap()
        ).await?;
        
        let change = repo.find_by_id_mut(&id)
            .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", id))?;
        
        change.update_body(body);
        
        events.push(Event::new(
            id.clone(),
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
        
        let change = repo.find_by_id_mut(&id)
            .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", id))?;
        
        let old_priority = change.priority.clone();
        change.update_priority(priority.clone());
        
        events.push(Event::new(
            id.clone(),
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
        
        repo.update_status(&id, new_status).await?;
        updated = true;
    }
    
    // Add all events
    for event in events {
        repo.db.add_event(event);
    }
    
    if updated {
        repo.save().await?;
        print_success(&format!("Updated change {}", id));
    } else {
        println!("No changes to save");
    }
    
    Ok(())
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
    
    tokio::fs::read_to_string(&temp_path).await.context("Failed to read edited file")
}
