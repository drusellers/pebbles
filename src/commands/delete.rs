use crate::commands::print_success;
use crate::config::get_db_path;
use crate::db::Db;
use crate::idish::IDish;
use anyhow::Result;

pub async fn delete(id: IDish, force: bool) -> Result<()> {
    let db_path = get_db_path()?;

    let mut db = Db::open(&db_path).await?;

    // Resolve ID to full ID
    let full_id = id.resolve(&db)?;

    // Check if change exists
    let change = db.get_change(&full_id)
        .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", full_id))?;

    // Confirm deletion unless --force
    if !force {
        let confirm = dialoguer::Confirm::new()
            .with_prompt(format!(
                "Are you sure you want to delete change '{}: {}'?",
                full_id, change.title
            ))
            .default(false)
            .interact()?;
        
        if !confirm {
            println!("Deletion cancelled.");
            return Ok(());
        }
    }

    // Delete the change
    db.delete_change(&full_id)?;
    db.save().await?;

    print_success(&format!("Deleted change {}", full_id));

    Ok(())
}
