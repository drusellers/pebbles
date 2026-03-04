use anyhow::{Context, Result};

use crate::commands::print_success;
use crate::config::{Config, find_pebbles_root};
use crate::template;
use crate::vcs::find_repo_root;

pub async fn init() -> Result<()> {
    // Check if already initialized
    if find_pebbles_root().is_ok() {
        anyhow::bail!("Pebbles is already initialized in this repository");
    }

    // Find git/jj root
    let repo_root = find_repo_root().context("Not in a git or jujutsu repository")?;

    let pebbles_dir = repo_root.join(".pebbles");
    let changes_dir = pebbles_dir.join("changes");
    let config_path = pebbles_dir.join("config.toml");

    // Create .pebbles directory
    tokio::fs::create_dir_all(&pebbles_dir)
        .await
        .context("Failed to create .pebbles directory")?;

    tokio::fs::create_dir_all(&changes_dir)
        .await
        .context("Failed to create .pebbles/changes directory")?;

    // Create default config
    let config = Config::default();
    config.save(&config_path).await?;

    // Create .opencode directory with command templates
    template::write_opencode_templates(&repo_root)
        .await
        .context("Failed to write OpenCode command templates")?;

    // Create .opencode/agents directory with agent templates
    template::write_opencode_agents(&repo_root)
        .await
        .context("Failed to write OpenCode agent templates")?;

    print_success(&format!("Initialized pebbles in {}", pebbles_dir.display()));

    Ok(())
}
