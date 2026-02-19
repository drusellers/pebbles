use crate::id::Id;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

pub mod git;
pub mod jujutsu;

pub trait Vcs: Send + Sync {
    fn name(&self) -> &'static str;
    #[allow(dead_code)]
    fn is_repo(&self, path: &Path) -> bool;
    fn detect(&self) -> bool;
    fn create_workspace(&self, id: &Id) -> Result<PathBuf>;
    fn cleanup_workspace(&self, id: &Id) -> Result<()>;
    fn generate_commit_msg(&self, title: &str, body: &str) -> Result<String>;
    fn current_workspace_id(&self) -> Option<Id>;
    #[allow(dead_code)]
    fn commit(&self, message: &str) -> Result<()>;
}

pub fn detect_vcs() -> Option<Box<dyn Vcs>> {
    let git = git::Git;
    if git.detect() {
        return Some(Box::new(git));
    }

    let jj = jujutsu::Jujutsu;
    if jj.detect() {
        return Some(Box::new(jj));
    }

    None
}

pub fn detect_vcs_with_preference(prefer: &str) -> Option<Box<dyn Vcs>> {
    match prefer {
        "git" => {
            let git = git::Git;
            if git.detect() {
                Some(Box::new(git))
            } else {
                None
            }
        }
        "jujutsu" | "jj" => {
            let jj = jujutsu::Jujutsu;
            if jj.detect() {
                Some(Box::new(jj))
            } else {
                None
            }
        }
        _ => detect_vcs(),
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
