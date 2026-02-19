use crate::commands::print_info;
use crate::config::{get_config_path, get_db_path, Config};
use crate::harness::{detect_harness_with_preference, HarnessContext};
use crate::vcs::detect_vcs_with_preference;
use anyhow::{Context, Result};
use colored::Colorize;
use std::env;
use std::io::{self, Read};
use std::path::PathBuf;

pub async fn intake(file: Option<PathBuf>) -> Result<()> {
    let db_path = get_db_path()?;

    let content = match file {
        Some(path) => {
            print_info(&format!("Reading from file: {}", path.display()));
            tokio::fs::read_to_string(&path)
                .await
                .with_context(|| format!("Failed to read file: {}", path.display()))?
        }
        None => {
            print_info("Reading from STDIN (press Ctrl+D when done)");
            let mut buffer = String::new();
            io::stdin()
                .read_to_string(&mut buffer)
                .context("Failed to read from STDIN")?;
            buffer
        }
    };

    if content.trim().is_empty() {
        anyhow::bail!("Input is empty. Please provide text to process.");
    }

    let config_path = get_config_path()?;
    let config = Config::load(&config_path).await?;

    let vcs = detect_vcs_with_preference(config.vcs.prefer)
        .context("No version control system detected (git or jujutsu)")?;

    let harness = detect_harness_with_preference(config.harness.prefer)
        .context("No AI harness detected (opencode)")?;

    let temp_file = tempfile::NamedTempFile::new()
        .context("Failed to create temporary file")?;
    let temp_path = temp_file.path().to_path_buf();
    tokio::fs::write(&temp_path, &content)
        .await
        .context("Failed to write to temporary file")?;

    println!("\n{}", "═".repeat(60).dimmed());
    println!("{}", "Intake Content Preview".white().bold());
    println!("{}", "═".repeat(60).dimmed());

    let preview = if content.len() > 500 {
        format!("{}...", &content[..500])
    } else {
        content.clone()
    };
    println!("\n{}", preview);
    println!("{}", "═".repeat(60).dimmed());

    print_info(&format!("Launching {} to process intake and create changes", harness.name()));

    let ctx = HarnessContext::new(vcs.name(), env::current_dir()?)
        .with_intake_file(&temp_path)
        .with_db_path(&db_path);

    #[cfg(unix)]
    {
        let _ = temp_file.into_temp_path();
    }

    harness.intake(&ctx)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_empty_input_detection() {
        assert!("".trim().is_empty());
        assert!("   ".trim().is_empty());
        assert!("\n\n\n".trim().is_empty());
        assert!("\t\n ".trim().is_empty());

        assert!(!"Hello".trim().is_empty());
        assert!(!"  Hello  ".trim().is_empty());
        assert!(!"\nHello\n".trim().is_empty());
    }

    #[tokio::test]
    async fn test_read_from_file() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let test_content = "Feature: Add user authentication\n\n- Create login page\n- Implement backend API\n";
        tokio::fs::write(temp_file.path(), test_content)
            .await
            .unwrap();

        let content = tokio::fs::read_to_string(temp_file.path())
            .await
            .unwrap();

        assert_eq!(content, test_content);
    }
}
