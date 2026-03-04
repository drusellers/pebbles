use crate::config::{Config, get_config_path};
use crate::id::Id;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

pub mod git;
pub mod jujutsu;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum VcsPreference {
    #[default]
    Auto,
    Git,
    Jujutsu,
}

pub trait Vcs: Send + Sync {
    fn name(&self) -> &'static str;
    #[allow(dead_code)]
    fn is_repo(&self, path: &Path) -> bool;
    fn detect(&self) -> bool;
    fn create_workspace(&self, id: &Id) -> Result<PathBuf>;
    fn cleanup_workspace(&self, id: &Id) -> Result<()>;
    fn current_workspace_id(&self) -> Option<Id>;
    fn commit(&self, message: &str) -> Result<()>;
}

pub fn detect_vcs() -> Option<Box<dyn Vcs>> {
    // Jujutsu wins over Git
    let jj = jujutsu::Jujutsu;
    if jj.detect() {
        return Some(Box::new(jj));
    }

    let git = git::Git;
    if git.detect() {
        return Some(Box::new(git));
    }

    None
}

pub async fn detect_vcs_with_preference() -> Result<Option<Box<dyn Vcs>>> {
    let config_path = get_config_path()?;
    let config = Config::load(&config_path).await?;

    match config.vcs.prefer {
        VcsPreference::Git => {
            let git = git::Git;
            if git.detect() {
                Ok(Some(Box::new(git)))
            } else {
                Ok(None)
            }
        }
        VcsPreference::Jujutsu => {
            let jj = jujutsu::Jujutsu;
            if jj.detect() {
                Ok(Some(Box::new(jj)))
            } else {
                Ok(None)
            }
        }
        VcsPreference::Auto => Ok(detect_vcs()),
    }
}

fn run_cmd(cmd: &mut Command) -> Result<String> {
    let output = cmd.output().context("Failed to execute command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Command failed: {}", stderr);
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn find_repo_root() -> Option<PathBuf> {
    let mut current = std::env::current_dir().ok()?;

    loop {
        if current.join(".git").exists() || current.join(".jj").exists() {
            return Some(current);
        }

        if !current.pop() {
            break;
        }
    }

    None
}

pub fn find_workspace_parent() -> Result<PathBuf> {
    let repo_root = find_repo_root().context("Not in a git or jujutsu repository")?;

    let parent = repo_root
        .parent()
        .context("Repository is at filesystem root, cannot create sibling workspace")?;

    Ok(parent.to_path_buf())
}
