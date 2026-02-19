use crate::commands::print_success;
use crate::idish::IDish;
use crate::repository::ChangeRepository;
use anyhow::Result;

pub async fn delete(id: IDish, force: bool) -> Result<()> {
    let mut repo = ChangeRepository::open().await?;

    // Resolve ID to full ID
    let full_id = id.resolve(&repo.db)?;

    // Check if change exists
    let change = repo.db.get_change(&full_id)
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
    repo.db.delete_change(&full_id)?;
    repo.save().await?;

    print_success(&format!("Deleted change {}", full_id));

    Ok(())
}
