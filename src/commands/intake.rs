use crate::commands::print_info;
use crate::config::{get_config_path, get_db_path, Config};
use crate::vcs::detect_vcs_with_preference;
use anyhow::{Context, Result};
use colored::Colorize;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::Command;

pub async fn intake(file: Option<PathBuf>) -> Result<()> {
    // Validate pebbles repository is initialized
    let db_path = get_db_path()?;

    // Read content from file or stdin
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

    // Handle empty input
    if content.trim().is_empty() {
        anyhow::bail!("Input is empty. Please provide text to process.");
    }

    let config_path = get_config_path()?;
    let config = Config::load(&config_path).await?;

    let vcs = detect_vcs_with_preference(config.vcs.prefer)
        .context("No version control system detected (git or jujutsu)")?;

    // Create a temporary file with the content
    // This is more reliable than passing large content via environment variable
    let temp_file = tempfile::NamedTempFile::new()
        .context("Failed to create temporary file")?;
    let temp_path = temp_file.path().to_path_buf();
    tokio::fs::write(&temp_path, &content)
        .await
        .context("Failed to write to temporary file")?;

    println!("\n{}", "═".repeat(60).dimmed());
    println!("{}", "Intake Content Preview".white().bold());
    println!("{}", "═".repeat(60).dimmed());

    // Show preview of content (first 500 chars)
    let preview = if content.len() > 500 {
        format!("{}...", &content[..500])
    } else {
        content.clone()
    };
    println!("\n{}", preview);
    println!("{}", "═".repeat(60).dimmed());

    print_info("Launching opencode to process intake and create changes");

    let mut cmd = Command::new("opencode");
    cmd.env("PEBBLES_INTAKE_FILE", temp_path.to_str().unwrap());
    cmd.env("PEBBLES_VCS", vcs.name());
    cmd.env("PEBBLES_DB_PATH", db_path.to_str().unwrap());
    cmd.args(["run", "/intake"]);

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // Keep the temp file alive by leaking it (it will be cleaned up by OS)
        let _ = temp_file.into_temp_path();
        let _ = cmd.exec();
        anyhow::bail!("Failed to launch opencode");
    }

    #[cfg(not(unix))]
    {
        let mut child = cmd
            .spawn()
            .context("Failed to launch opencode. Is it installed?")?;
        child.wait()?;
        // Temp file will be deleted automatically when temp_file is dropped
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_empty_input_detection() {
        // Test various forms of empty input
        assert!("".trim().is_empty());
        assert!("   ".trim().is_empty());
        assert!("\n\n\n".trim().is_empty());
        assert!("\t\n ".trim().is_empty());

        // Test non-empty input
        assert!(!"Hello".trim().is_empty());
        assert!(!"  Hello  ".trim().is_empty());
        assert!(!"\nHello\n".trim().is_empty());
    }

    #[tokio::test]
    async fn test_read_from_file() {
        // Create a temporary file with test content
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let test_content = "Feature: Add user authentication\n\n- Create login page\n- Implement backend API\n";
        tokio::fs::write(temp_file.path(), test_content)
            .await
            .unwrap();

        // Read it back
        let content = tokio::fs::read_to_string(temp_file.path())
            .await
            .unwrap();

        assert_eq!(content, test_content);
    }
}
