use crate::commands::{print_success, resolve_id};
use crate::config::{get_config_path, get_db_path, Config};
use crate::db::Db;
use crate::idish::IDish;
use crate::models::{Event, EventType};
use crate::repository::ChangeRepository;
use anyhow::{Context, Result};

pub async fn edit(id: Option<IDish>) -> Result<()> {
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

    let mut repo = ChangeRepository::open(db_path).await?;

    let change = repo.find_by_id(&full_id)
        .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", full_id))?;

    let config_path = get_config_path().unwrap();
    let config = Config::load(&config_path).await?;
    let editor = config.get_editor();

    let temp_file = tempfile::NamedTempFile::new()?;
    let temp_path = temp_file.path().to_path_buf();

    // Write current body
    tokio::fs::write(&temp_path, &change.body).await?;

    // Launch editor
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("{} {}", editor, temp_path.display()))
        .status()
        .context("Failed to launch editor")?;

    if !status.success() {
        anyhow::bail!("Editor exited with non-zero status");
    }

    // Read back
    let new_body = tokio::fs::read_to_string(&temp_path).await?;

    // Update if changed
    if new_body != change.body {
        let change = repo.find_by_id_mut(&full_id).unwrap();
        change.update_body(new_body);

        let event = Event::new(
            full_id.clone(),
            EventType::Updated,
            serde_json::json!({
                "field": "body",
                "editor": editor,
            }),
        );
        repo.db.add_event(event);

        repo.save().await?;
        print_success(&format!("Updated body of change {}", full_id));
    } else {
        println!("No changes made");
    }

    Ok(())
}
