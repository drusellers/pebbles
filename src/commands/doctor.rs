use anyhow::Result;
use colored::Colorize;
use std::process::Command;

pub async fn doctor() -> Result<()> {
    println!("{}", "Pebbles Doctor".bold());
    println!("{}", "═".repeat(50).dimmed());

    let mut all_good = true;

    // Check for Jujutsu
    print!("Checking for Jujutsu... ");
    match Command::new("jj").args(["--version"]).output() {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("{}", format!("✓ {}", version).green());
        }
        _ => {
            println!("{}", "✗ not found".red());
            println!("   {}", "Install from: https://github.com/martinvonz/jj".dimmed());
            all_good = false;
        }
    }

    // Check for Git
    print!("Checking for Git... ");
    match Command::new("git").args(["--version"]).output() {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("{}", format!("✓ {}", version).green());
        }
        _ => {
            println!("{}", "✗ not found".red());
            println!("   {}", "Install git from your package manager".dimmed());
            all_good = false;
        }
    }

    // Check for EDITOR
    print!("Checking for EDITOR... ");
    match std::env::var("EDITOR") {
        Ok(editor) => {
            // Verify the editor actually exists
            let parts: Vec<&str> = editor.split_whitespace().collect();
            let cmd = parts[0];
            
            match Command::new("which").arg(cmd).output() {
                Ok(output) if output.status.success() => {
                    println!("{}", format!("✓ {}", editor).green());
                }
                _ => {
                    println!("{}", format!("⚠ {} (command not found in PATH)", editor).yellow());
                    println!("   {}", format!("EDITOR is set to '{}' but command not found", cmd).dimmed());
                    all_good = false;
                }
            }
        }
        Err(_) => {
            println!("{}", "⚠ not set".yellow());
            println!("   {}", "Set EDITOR environment variable (e.g., export EDITOR=vim)".dimmed());
            println!("   {}", "Will use default: vim (Unix) or notepad (Windows)".dimmed());
        }
    }

    // Check for opencode
    print!("Checking for opencode... ");
    match Command::new("opencode").args(["--version"]).output() {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("{}", format!("✓ {}", version).green());
        }
        _ => {
            println!("{}", "✗ not found".red());
            println!("   {}", "Install from: https://opencode.ai".dimmed());
            all_good = false;
        }
    }

    println!("{}", "═".repeat(50).dimmed());

    if all_good {
        println!("{}", "All systems ready! ✓".green().bold());
    } else {
        println!("{}", "Some dependencies are missing. Please install them.".yellow().bold());
    }

    Ok(())
}
