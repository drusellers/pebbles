use crate::commands::{print_info, resolve_id};
use crate::config::{Config, get_config_path};
use crate::harness::{HarnessContext, detect_harness_with_preference};
use crate::idish::IDish;
use crate::repository::ChangeRepository;
use crate::vcs::detect_vcs_with_preference;
use anyhow::{Context, Result};
use chrono::Utc;
use colored::Colorize;
use std::env;

const AGENT_PREFIX: &str = "!agent:";
const AGENT_PREFIX_ALT: &str = "!agent ";

#[derive(Debug, Clone)]
struct AgentDirective {
    line_number: usize,
    content: String,
    indentation: String,
}

fn extract_agent_directives(text: &str) -> Vec<AgentDirective> {
    text.lines()
        .enumerate()
        .filter_map(|(idx, line)| {
            let trimmed = line.trim_start();
            let indent = &line[..line.len() - trimmed.len()];
            
            // Check for !agent: or !agent (with space)
            let directive = trimmed
                .strip_prefix(AGENT_PREFIX)
                .or_else(|| trimmed.strip_prefix(&AGENT_PREFIX.to_uppercase()))
                .map(|s| s.trim().to_string())
                .or_else(|| {
                    trimmed
                        .strip_prefix(AGENT_PREFIX_ALT)
                        .or_else(|| trimmed.strip_prefix(&AGENT_PREFIX_ALT.to_uppercase()))
                        .map(|s| s.trim().to_string())
                });
            
            directive.map(|content| AgentDirective {
                line_number: idx,
                content,
                indentation: indent.to_string(),
            })
        })
        .collect()
}

fn replace_directives_in_text(text: &str, directives: &[AgentDirective], answers: &[String]) -> String {
    let mut lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();
    
    for (idx, directive) in directives.iter().enumerate() {
        if idx < answers.len() {
            let replacement = format!("{}{}", directive.indentation, answers[idx]);
            if directive.line_number < lines.len() {
                lines[directive.line_number] = replacement;
            }
        }
    }
    
    lines.join("\n")
}

pub async fn plan(id: Option<IDish>, _wait: bool) -> Result<()> {
    let mut repo = ChangeRepository::open().await?;

    let full_id = match id {
        Some(id) => id.resolve(&repo.db)?,
        None => resolve_id(None).await?,
    };

    let change = repo
        .find_by_id(&full_id)
        .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", full_id))?
        .clone();

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

    // Extract directives from title and body
    let title_directives = extract_agent_directives(&change.title);
    let body_directives = extract_agent_directives(&change.body);
    
    let all_directives: Vec<AgentDirective> = title_directives
        .clone()
        .into_iter()
        .chain(body_directives.clone())
        .collect();

    if all_directives.is_empty() {
        print_info("No !agent directives found. Nothing to plan.");
        return Ok(());
    }

    println!(
        "\n{}",
        format!("🤖 Processing {} agent directive(s)", all_directives.len())
            .yellow()
            .dimmed()
    );

    // Extract just the directive content for the LLM
    let directive_contents: Vec<String> = all_directives
        .iter()
        .map(|d| d.content.clone())
        .collect();

    let ctx = HarnessContext::new(vcs.name(), env::current_dir()?)
        .with_change_id(full_id.clone());

    print_info(&format!(
        "Querying {} for {} directive(s)...",
        harness.name(),
        all_directives.len()
    ));

    // Call the LLM synchronously to get answers
    let answers = harness.answer_directives(&ctx, directive_contents)?;

    if answers.len() != all_directives.len() {
        anyhow::bail!(
            "LLM returned {} answers but {} were expected",
            answers.len(),
            all_directives.len()
        );
    }

    // Split answers between title and body
    let title_answer_count = title_directives.len();
    let title_answers = &answers[..title_answer_count];
    let body_answers = &answers[title_answer_count..];

    // Replace directives in title if any
    if !title_directives.is_empty() {
        let new_title = replace_directives_in_text(&change.title, &title_directives, title_answers);
        
        let change_mut = repo
            .find_by_id_mut(&full_id)
            .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", full_id))?;
        change_mut.update_title(new_title);
    }

    // Replace directives in body if any
    if !body_directives.is_empty() {
        let new_body = replace_directives_in_text(&change.body, &body_directives, body_answers);
        
        let change_mut = repo
            .find_by_id_mut(&full_id)
            .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", full_id))?;
        change_mut.update_body(new_body);
    }

    // Add log entry
    let today = Utc::now().format("%Y-%m-%d").to_string();
    let log_entry = format!(
        "- {}: Planning completed - processed {} agent directive(s)",
        today,
        all_directives.len()
    );

    let change_mut = repo
        .find_by_id_mut(&full_id)
        .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", full_id))?;
    
    let current_body = change_mut.body.clone();
    let updated_body = if current_body.contains("## Log") {
        // Append to existing log section - find the line after "## Log"
        let lines: Vec<&str> = current_body.lines().collect();
        let mut result = Vec::new();
        let mut found_log = false;
        
        for line in lines {
            result.push(line);
            if line.trim() == "## Log" {
                found_log = true;
                result.push(&log_entry);
            }
        }
        
        if !found_log {
            result.push(&log_entry);
        }
        
        result.join("\n")
    } else {
        // Create new log section
        format!("{}\n\n## Log\n{}", current_body, log_entry)
    };
    
    change_mut.update_body(updated_body);

    // Save the changes
    repo.save().await?;

    print_info(&format!(
        "✓ Updated change with {} LLM response(s)",
        all_directives.len()
    ));

    Ok(())
}
