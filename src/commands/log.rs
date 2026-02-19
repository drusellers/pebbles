use crate::commands::resolve_id;
use crate::idish::IDish;
use crate::repository::ChangeRepository;
use anyhow::Result;
use colored::Colorize;

pub async fn log(id: Option<IDish>) -> Result<()> {
    let full_id = resolve_id(id).await?;

    let repo = ChangeRepository::open().await?;

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
