use crate::commands::{print_info, print_success};
use crate::config::get_pebbles_dir;
use crate::db::Database;
use crate::markdown_store::write_change_file_to_path;
use anyhow::{Context, Result};

pub async fn migrate() -> Result<()> {
    let pebbles_dir = get_pebbles_dir()?;
    let db_path = pebbles_dir.join("db.json");
    let changes_dir = pebbles_dir.join("changes");

    if !db_path.exists() {
        anyhow::bail!("No legacy database found at {}", db_path.display());
    }

    let content = tokio::fs::read_to_string(&db_path)
        .await
        .with_context(|| format!("Failed to read {}", db_path.display()))?;
    let data: Database = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse {}", db_path.display()))?;

    tokio::fs::create_dir_all(&changes_dir)
        .await
        .with_context(|| format!("Failed to create {}", changes_dir.display()))?;

    print_info("Migrating legacy .pebbles/db.json to markdown files");

    for change in data.changes.values() {
        let path = changes_dir.join(format!("{}.md", change.id));
        let events = data
            .events
            .iter()
            .filter(|event| event.change_id == change.id)
            .cloned()
            .collect::<Vec<_>>();
        write_change_file_to_path(&path, change, &events).await?;
    }

    tokio::fs::remove_file(&db_path)
        .await
        .with_context(|| format!("Failed to remove {}", db_path.display()))?;

    print_success(&format!(
        "Migrated {} change(s) to {} and removed {}",
        data.changes.len(),
        changes_dir.display(),
        db_path.display()
    ));

    Ok(())
}
