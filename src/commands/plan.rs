use crate::commands::{print_info, resolve_id};
use crate::config::{Config, get_config_path};
use crate::harness::{HarnessContext, detect_harness_with_preference};
use crate::idish::IDish;
use crate::repository::ChangeRepository;
use crate::vcs::detect_vcs_with_preference;
use anyhow::{Context, Result};
use colored::Colorize;
use std::env;

const AGENT_PREFIX: &str = "!agent:";

fn extract_agent_instructions(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            trimmed
                .strip_prefix(AGENT_PREFIX)
                .or_else(|| trimmed.strip_prefix(&AGENT_PREFIX.to_uppercase()))
                .or_else(|| {
                    trimmed
                        .strip_prefix("!agent ")
                        .or_else(|| trimmed.strip_prefix("!AGENT "))
                })
                .map(|instruction| instruction.trim().to_string())
        })
        .collect()
}

pub async fn plan(id: Option<IDish>, wait: bool) -> Result<()> {
    let repo = ChangeRepository::open().await?;

    let full_id = match id {
        Some(id) => id.resolve(&repo.db)?,
        None => resolve_id(None).await?,
    };

    let change = repo
        .find_by_id(&full_id)
        .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", full_id))?;

    let config_path = get_config_path()?;
    let config = Config::load(&config_path).await?;

    let vcs = detect_vcs_with_preference()
        .await?
        .context("No version control system detected (git or jujutsu)")?;

    let harness = detect_harness_with_preference(config.harness.prefer)
        .context("No AI harness detected (opencode)")?;

    println!("\n{}", "═".repeat(60).dimmed());
    println!(
        "{} {} {}",
        change.id.as_str().cyan().bold(),
        "─".dimmed(),
        change.title.white().bold()
    );
    println!("{}", "═".repeat(60).dimmed());

    if !change.body.is_empty() {
        println!("\n{}", change.body);
        println!("{}", "═".repeat(60).dimmed());
    }

    let title_instructions = extract_agent_instructions(&change.title);
    let body_instructions = extract_agent_instructions(&change.body);
    let all_instructions: Vec<String> = title_instructions
        .into_iter()
        .chain(body_instructions)
        .collect();

    if !all_instructions.is_empty() {
        println!(
            "\n{}",
            format!(
                "🤖 {} agent instruction(s) embedded",
                all_instructions.len()
            )
            .yellow()
            .dimmed()
        );
    }

    if !wait {
        print_info(&format!(
            "Launching {} to plan change '{}'",
            harness.name(),
            full_id
        ));
    } else {
        print_info(&format!(
            "Launching {} for change '{}'",
            harness.name(),
            full_id
        ));
    }

    let ctx = HarnessContext::new(vcs.name(), env::current_dir()?)
        .with_change_id(full_id)
        .with_agent_instructions(all_instructions)
        .with_wait_mode(wait);

    harness.plan(&ctx)
}
