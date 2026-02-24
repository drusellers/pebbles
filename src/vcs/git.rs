use super::{find_workspace_parent, run_cmd, Vcs};
use crate::id::Id;
use anyhow::Result;
use std::path::Path;
use std::process::Command;

pub struct Git;

impl Vcs for Git {
    fn name(&self) -> &'static str {
        "git"
    }

    fn is_repo(&self, path: &Path) -> bool {
        path.join(".git").exists()
    }

    fn detect(&self) -> bool {
        let output = Command::new("git")
            .args(["rev-parse", "--git-dir"])
            .output();

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

        // Create worktree
        let branch_name = format!("pebbles/{}", id);

        // Create branch if it doesn't exist
        let _ = Command::new("git").args(["branch", &branch_name]).output();

        // Create worktree
        run_cmd(Command::new("git").args([
            "worktree",
            "add",
            workspace_path.to_str().unwrap(),
            &branch_name,
        ]))?;

        Ok(workspace_path)
    }

    fn cleanup_workspace(&self, id: &Id) -> Result<()> {
        let parent_dir = find_workspace_parent()?;
        let workspace_path = parent_dir.join(format!("ws-{}", id));

        if !workspace_path.exists() {
            anyhow::bail!("Workspace 'ws-{}' does not exist", id);
        }

        // Remove worktree
        run_cmd(Command::new("git").args([
            "worktree",
            "remove",
            workspace_path.to_str().unwrap(),
        ]))?;

        // Remove branch
        let branch_name = format!("pebbles/{}", id);
        let _ = Command::new("git")
            .args(["branch", "-D", &branch_name])
            .output();

        Ok(())
    }

    fn current_workspace_id(&self) -> Option<Id> {
        let current_dir = std::env::current_dir().ok()?;
        let dir_name = current_dir.file_name()?.to_str()?;

        dir_name.strip_prefix("ws-").and_then(|s| Id::new(s).ok())
    }

    fn commit(&self, message: &str) -> Result<()> {
        run_cmd(Command::new("git").args(["commit", "-m", message]))?;
        Ok(())
    }
}
