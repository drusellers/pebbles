use anyhow::Result;
use std::path::Path;
use std::process::Command;

use super::{run_cmd, Vcs};

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

    fn create_workspace(&self, name: &str) -> Result<std::path::PathBuf> {
        let workspace_path = std::env::current_dir()?.join(format!("ws-{}", name));

        // Check if already exists
        if workspace_path.exists() {
            return Ok(workspace_path);
        }

        // Create worktree
        let branch_name = format!("pebbles/{}", name);

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

    fn cleanup_workspace(&self, name: &str) -> Result<()> {
        let workspace_path = std::env::current_dir()?.join(format!("ws-{}", name));

        if !workspace_path.exists() {
            anyhow::bail!("Workspace 'ws-{}' does not exist", name);
        }

        // Remove worktree
        run_cmd(Command::new("git").args([
            "worktree",
            "remove",
            workspace_path.to_str().unwrap(),
        ]))?;

        // Remove branch
        let branch_name = format!("pebbles/{}", name);
        let _ = Command::new("git")
            .args(["branch", "-D", &branch_name])
            .output();

        Ok(())
    }

    fn generate_commit_msg(&self, title: &str, _body: &str) -> Result<String> {
        // Simple implementation - could be enhanced with AI
        Ok(format!("{}\n\nImplemented change", title))
    }

    fn current_workspace_id(&self) -> Option<String> {
        let current_dir = std::env::current_dir().ok()?;
        let dir_name = current_dir.file_name()?.to_str()?;

        if dir_name.starts_with("ws-") {
            Some(dir_name[3..].to_string())
        } else {
            None
        }
    }

    fn commit(&self, message: &str) -> Result<()> {
        run_cmd(Command::new("git").args(["commit", "-m", message]))?;
        Ok(())
    }
}
