use super::{find_workspace_parent, run_cmd, Vcs};
use crate::id::Id;
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

pub struct Jujutsu;

impl Vcs for Jujutsu {
    fn name(&self) -> &'static str {
        "jujutsu"
    }

    fn is_repo(&self, path: &Path) -> bool {
        path.join(".jj").exists()
    }

    fn detect(&self) -> bool {
        let output = Command::new("jj").args(["status"]).output();

        match output {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    fn create_workspace(&self, id: &Id) -> Result<std::path::PathBuf> {
        let parent_dir = find_workspace_parent()?;
        let workspace_path = parent_dir.join(format!("ws-{}", id));

        // Check if already exists
        if workspace_path.exists() {
            return Ok(workspace_path);
        }

        // Create jj workspace
        run_cmd(Command::new("jj").args([
            "workspace",
            "create",
            workspace_path.to_str().unwrap(),
        ]))?;

        Ok(workspace_path)
    }

    fn cleanup_workspace(&self, id: &Id) -> Result<()> {
        let parent_dir = find_workspace_parent()?;
        let workspace_path = parent_dir.join(format!("ws-{}", id));

        if !workspace_path.exists() {
            anyhow::bail!("Workspace 'ws-{}' does not exist", id);
        }

        // Remove workspace
        run_cmd(Command::new("jj").args([
            "workspace",
            "forget",
            workspace_path.to_str().unwrap(),
        ]))?;

        // Remove directory
        std::fs::remove_dir_all(&workspace_path).context("Failed to remove workspace directory")?;

        Ok(())
    }

    fn generate_commit_msg(&self, title: &str, _body: &str) -> Result<String> {
        // Simple implementation - could be enhanced with AI
        Ok(format!("{}\n\nImplemented change", title))
    }

    fn current_workspace_id(&self) -> Option<Id> {
        let current_dir = std::env::current_dir().ok()?;
        let dir_name = current_dir.file_name()?.to_str()?;

        dir_name.strip_prefix("ws-").and_then(|s| Id::new(s).ok())
    }

    fn commit(&self, message: &str) -> Result<()> {
        // In jj, we describe the change
        run_cmd(Command::new("jj").args(["describe", "-m", message]))?;
        Ok(())
    }
}
