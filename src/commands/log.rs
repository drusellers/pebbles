use crate::commands::resolve_id;
use crate::config::get_db_path;
use crate::db::Db;
use crate::idish::IDish;
use crate::repository::ChangeRepository;
use anyhow::{Context, Result};
use colored::Colorize;

pub async fn log(id: Option<IDish>) -> Result<()> {
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

    let repo = ChangeRepository::open(db_path).await?;

    // Verify change exists
    let _change = repo.find_by_id(&full_id)
        .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", full_id))?;

    println!("\n{}", format!("Event log for change {}", full_id).bold());
    println!("{}", "─".repeat(60).dimmed());

    let events = repo.get_events(&full_id);

    if events.is_empty() {
        println!("\n  No events recorded.\n");
        return Ok(());
    }

    for event in events {
        let timestamp = event.created_at.format("%Y-%m-%d %H:%M:%S").to_string();
        let event_type = format!("{:?}", event.event_type).cyan();

        println!("\n  {} {}", timestamp.dimmed(), event_type);

        // Print event data if present
        if event.data != serde_json::Value::Null
            && let Some(obj) = event.data.as_object()
        {
            for (key, value) in obj {
                println!("    {}: {}", key, value);
            }
        }
    }

    println!();
    Ok(())
}
