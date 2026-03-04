use crate::commands::resolve_id;
use crate::idish::IDish;
use crate::repository::ChangeRepository;
use anyhow::Result;
use colored::Colorize;

pub async fn log(id: Option<IDish>) -> Result<()> {
    let repo = ChangeRepository::open().await?;

    let full_id = match id {
        Some(idish) => match idish.resolve(&repo.db) {
            Ok(id) => id,
            Err(err) => {
                if let Some(invalid_id) = repo.resolve_invalid_idish(&idish)?
                    && let Some(invalid) = repo.invalid_change_by_id(&invalid_id)
                {
                    println!("\n{}", "Invalid change file".red().bold());
                    println!("  id: {}", invalid.id.to_string().cyan().bold());
                    println!("  file: {}", invalid.path.display());
                    println!("  reason: {}", invalid.error.red());
                    println!(
                        "  fix: correct the markdown/frontmatter and re-run `pebbles log {}`",
                        invalid.id
                    );
                    println!();
                    return Ok(());
                }
                return Err(err);
            }
        },
        None => resolve_id(None).await?,
    };

    // Verify change exists
    let _change = repo.find_by_id(&full_id).ok_or_else(|| {
        if let Some(invalid) = repo.invalid_change_by_id(&full_id) {
            anyhow::anyhow!(
                "Change '{}' has invalid markdown at {}: {}",
                full_id,
                invalid.path.display(),
                invalid.error
            )
        } else {
            anyhow::anyhow!("Change '{}' not found", full_id)
        }
    })?;

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
