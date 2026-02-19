use anyhow::{Context, Result};

use crate::commands::print_success;
use crate::config::{Config, find_pebbles_root};
use crate::db::Db;
use crate::vcs::find_repo_root;

pub async fn init() -> Result<()> {
    // Check if already initialized
    if find_pebbles_root().is_some() {
        anyhow::bail!("Pebbles is already initialized in this repository");
    }

    // Find git/jj root
    let repo_root = find_repo_root()
        .context("Not in a git or jujutsu repository")?;

    let pebbles_dir = repo_root.join(".pebbles");
    let db_path = pebbles_dir.join("db.json");
    let config_path = pebbles_dir.join("config.toml");

    // Create .pebbles directory
    tokio::fs::create_dir_all(&pebbles_dir)
        .await
        .context("Failed to create .pebbles directory")?;

    // Create empty database
    let db = Db::open(&db_path).await?;
    db.save().await?;

    // Create default config
    let config = Config::default();
    config.save(&config_path).await?;

    // Create .opencode directory with commands
    create_opencode_commands(&repo_root).await?;

    print_success(&format!(
        "Initialized pebbles in {}",
        pebbles_dir.display()
    ));

    Ok(())
}

async fn create_opencode_commands(repo_root: &std::path::Path) -> Result<()> {
    let opencode_dir = repo_root.join(".opencode").join("commands");

    tokio::fs::create_dir_all(&opencode_dir)
        .await
        .context("Failed to create .opencode/commands directory")?;

    // Create implement command
    let implement_content = include_str!("../../.opencode/commands/implement.md");
    tokio::fs::write(
        opencode_dir.join("implement.md"),
        implement_content
    ).await?;

    // Create describe command
    let describe_content = include_str!("../../.opencode/commands/describe.md");
    tokio::fs::write(
        opencode_dir.join("describe.md"),
        describe_content
    ).await?;

    Ok(())
}
