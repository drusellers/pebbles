use crate::id::Id;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

pub mod opencode;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum HarnessPreference {
    #[default]
    Auto,
    OpenCode,
}

pub trait Harness: Send + Sync {
    fn name(&self) -> &'static str;
    fn detect(&self) -> bool;

    fn plan(&self, ctx: &HarnessContext) -> Result<()>;
    fn implement(&self, ctx: &HarnessContext) -> Result<()>;
    fn intake(&self, ctx: &HarnessContext) -> Result<()>;

    /// Generate a commit message for a completed change
    /// Returns the formatted commit message as a string
    fn generate_commit_msg(&self, change_id: &Id) -> Result<String>;
}

#[derive(Debug, Clone)]
pub struct HarnessContext {
    pub change_id: Option<Id>,
    pub vcs_name: String,
    pub work_dir: PathBuf,
    pub intake_file: Option<PathBuf>,
    pub db_path: Option<PathBuf>,
    pub agent_instructions: Vec<String>,
    pub skip_permissions: bool,
    pub print_logs: bool,
    pub wait_mode: bool,
}

impl HarnessContext {
    pub fn new(vcs_name: impl Into<String>, work_dir: impl Into<PathBuf>) -> Self {
        Self {
            change_id: None,
            vcs_name: vcs_name.into(),
            work_dir: work_dir.into(),
            intake_file: None,
            db_path: None,
            agent_instructions: Vec::new(),
            skip_permissions: false,
            print_logs: false,
            wait_mode: false,
        }
    }

    pub fn with_change_id(mut self, id: Id) -> Self {
        self.change_id = Some(id);
        self
    }

    pub fn with_intake_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.intake_file = Some(path.into());
        self
    }

    pub fn with_db_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.db_path = Some(path.into());
        self
    }

    pub fn with_agent_instructions(mut self, instructions: Vec<String>) -> Self {
        self.agent_instructions = instructions;
        self
    }

    pub fn with_skip_permissions(mut self, skip: bool) -> Self {
        self.skip_permissions = skip;
        self
    }

    pub fn with_print_logs(mut self, print: bool) -> Self {
        self.print_logs = print;
        self
    }

    pub fn with_wait_mode(mut self, wait: bool) -> Self {
        self.wait_mode = wait;
        self
    }
}

pub fn detect_harness() -> Option<Box<dyn Harness>> {
    let opencode = opencode::OpenCode;
    if opencode.detect() {
        return Some(Box::new(opencode));
    }

    None
}

pub fn detect_harness_with_preference(prefer: HarnessPreference) -> Option<Box<dyn Harness>> {
    match prefer {
        HarnessPreference::OpenCode => {
            let opencode = opencode::OpenCode;
            if opencode.detect() {
                Some(Box::new(opencode))
            } else {
                None
            }
        }
        HarnessPreference::Auto => detect_harness(),
    }
}

pub fn run_harness_command(cmd: &mut Command) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let _ = cmd.exec();
        anyhow::bail!("Failed to launch {}", cmd.get_program().to_string_lossy());
    }

    #[cfg(not(unix))]
    {
        use std::process::Stdio;

        let program = cmd.get_program().to_string_lossy().to_string();
        let mut child = cmd
            .spawn()
            .with_context(|| format!("Failed to launch {}. Is it installed?", program))?;
        child.wait()?;
        Ok(())
    }
}

pub fn check_binary(name: &str) -> bool {
    Command::new(name)
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
