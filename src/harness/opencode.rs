use super::{check_binary, run_harness_command, Harness, HarnessContext};
use crate::id::Id;
use anyhow::{Context, Result};
use std::process::Command;

pub struct OpenCode;

impl Harness for OpenCode {
    fn name(&self) -> &'static str {
        "opencode"
    }

    fn detect(&self) -> bool {
        check_binary("opencode")
    }

    fn plan(&self, ctx: &HarnessContext) -> Result<()> {
        let mut cmd = Command::new("opencode");
        cmd.current_dir(&ctx.work_dir);

        if let Some(ref id) = ctx.change_id {
            cmd.env("PEBBLES_CHANGE", id.to_string());
        }
        cmd.env("PEBBLES_VCS", &ctx.vcs_name);

        if !ctx.agent_instructions.is_empty() {
            let instructions_text = ctx.agent_instructions.join("\n");
            cmd.env("PEBBLES_AGENT_INSTRUCTIONS", instructions_text);
        }

        if !ctx.wait_mode {
            cmd.args(["run", "/plan"]);
        }

        run_harness_command(&mut cmd)
    }

    fn implement(&self, ctx: &HarnessContext) -> Result<()> {
        let mut cmd = Command::new("opencode");
        cmd.current_dir(&ctx.work_dir);

        if let Some(ref id) = ctx.change_id {
            cmd.env("PEBBLES_CHANGE", id.to_string());
        }
        cmd.env("PEBBLES_VCS", &ctx.vcs_name);

        if ctx.skip_permissions {
            cmd.env("OPENCODE_SKIP_PERMISSIONS", "1");
        }

        if !ctx.wait_mode {
            cmd.args(["run", "/implement"]);
            if ctx.print_logs {
                cmd.arg("--print-logs");
            }
        }

        run_harness_command(&mut cmd)
    }

    fn intake(&self, ctx: &HarnessContext) -> Result<()> {
        let mut cmd = Command::new("opencode");
        cmd.current_dir(&ctx.work_dir);

        if let Some(ref path) = ctx.intake_file {
            cmd.env("PEBBLES_INTAKE_FILE", path.to_str().unwrap());
        }

        cmd.env("PEBBLES_VCS", &ctx.vcs_name);

        if let Some(ref path) = ctx.db_path {
            cmd.env("PEBBLES_DB_PATH", path.to_str().unwrap());
        }

        cmd.args(["run", "/intake"]);

        run_harness_command(&mut cmd)
    }

    fn generate_commit_msg(&self, change_id: &Id) -> Result<String> {
        let mut cmd = Command::new("opencode");
        cmd.current_dir(std::env::current_dir()?);

        cmd.env("PEBBLES_CHANGE", change_id.to_string());
        cmd.args(["run", "/describe"]);

        let output = cmd.output().context("Failed to run opencode /describe")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("opencode /describe failed: {}", stderr);
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    fn answer_directives(
        &self,
        ctx: &HarnessContext,
        directives: Vec<String>,
    ) -> Result<Vec<String>> {
        let mut cmd = Command::new("opencode");
        cmd.current_dir(&ctx.work_dir);

        if let Some(ref id) = ctx.change_id {
            cmd.env("PEBBLES_CHANGE", id.to_string());
        }
        cmd.env("PEBBLES_VCS", &ctx.vcs_name);

        // Pass directives as numbered list
        let directives_text = directives
            .iter()
            .enumerate()
            .map(|(i, d)| format!("{}. {}", i + 1, d))
            .collect::<Vec<_>>()
            .join("\n");
        cmd.env("PEBBLES_DIRECTIVES", directives_text);
        cmd.env("PEBBLES_DIRECTIVE_COUNT", directives.len().to_string());

        cmd.args(["run", "/answer"]);

        let output = cmd.output().context("Failed to run opencode /answer")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("opencode /answer failed: {}", stderr);
        }

        let response = String::from_utf8_lossy(&output.stdout);
        let answers = parse_numbered_list(&response, directives.len())?;

        Ok(answers)
    }
}

fn parse_numbered_list(text: &str, expected_count: usize) -> Result<Vec<String>> {
    let mut answers: Vec<Option<String>> = vec![None; expected_count];

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Try to match "N. " or "N) " patterns
        if let Some((num_part, rest)) = trimmed.split_once('.')
            && let Ok(num) = num_part.trim().parse::<usize>()
            && num > 0 && num <= expected_count
        {
            answers[num - 1] = Some(rest.trim().to_string());
        } else if let Some((num_part, rest)) = trimmed.split_once(')')
            && let Ok(num) = num_part.trim().parse::<usize>()
            && num > 0 && num <= expected_count
        {
            answers[num - 1] = Some(rest.trim().to_string());
        }
    }

    // Verify all answers were found
    let mut result = Vec::with_capacity(expected_count);
    for (i, answer) in answers.iter().enumerate() {
        match answer {
            Some(a) => result.push(a.clone()),
            None => {
                anyhow::bail!(
                    "LLM response missing answer for directive {}. Expected {} answers.",
                    i + 1,
                    expected_count
                );
            }
        }
    }

    Ok(result)
}
