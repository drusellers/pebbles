use anyhow::{Context, Result};
use colored::Colorize;


use crate::commands::resolve_id;
use crate::config::get_db_path;
use crate::repository::ChangeRepository;

pub async fn log(id: Option<String>) -> Result<()> {
    let db_path = get_db_path()
        .context("Not in a pebbles repository. Run 'pebbles init' first.")?;
    
    let repo = ChangeRepository::open(db_path).await?;
    
    let id = resolve_id(id)?;
    
    // Verify change exists
    // Verify change exists
    let _change = repo.find_by_id(&id)
        .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", id))?;
    
    println!("\n{}", format!("Event log for change {}", id).bold());
    println!("{}", "─".repeat(60).dimmed());
    
    let events = repo.get_events(&id);
    
    if events.is_empty() {
        println!("\n  No events recorded.\n");
        return Ok(());
    }
    
    for event in events {
        let timestamp = event.created_at.format("%Y-%m-%d %H:%M:%S").to_string();
        let event_type = format!("{:?}", event.event_type).cyan();
        
        println!("\n  {} {}", timestamp.dimmed(), event_type);
        
        // Print event data if present
        if event.data != serde_json::Value::Null {
            if let Some(obj) = event.data.as_object() {
                for (key, value) in obj {
                    println!("    {}: {}", key, value);
                }
            }
        }
    }
    
    println!();
    Ok(())
}
