use crate::vcs::detect_vcs;
use anyhow::{Context, Result};

pub async fn current() -> Result<()> {
    let vcs = detect_vcs()
        .context("No version control system detected")?;
    
    if let Some(id) = vcs.current_workspace_id() {
        println!("{}", id);
    } else {
        print_not_in_workspace();
        std::process::exit(1);
    }
    
    Ok(())
}

fn print_not_in_workspace() {
    use colored::Colorize;
    
    eprintln!("{} Not in a pebbles workspace.", "!".yellow());
    
    if crate::config::get_db_path().is_some() {
        eprintln!("\nYou are in a pebbles-enabled repository.");
        
        if let Some(root) = crate::config::find_pebbles_root() {
            let workspaces: Vec<String> = std::fs::read_dir(&root)
                .ok()
                .into_iter()
                .flatten()
                .filter_map(|e| e.ok())
                .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
                .filter(|name| name.starts_with("ws-"))
                .map(|name| name[3..].to_string())
                .collect();
            
            if !workspaces.is_empty() {
                eprintln!("\nAvailable workspaces:");
                for ws in workspaces {
                    eprintln!("  ws-{}", ws);
                }
                eprintln!("\nRun 'pebbles work <id>' to create a new workspace.");
            } else {
                eprintln!("\nNo workspaces found. Run 'pebbles work <id>' to create one.");
            }
        }
    } else {
        eprintln!("\nRun 'pebbles init' to initialize a pebbles repository.");
    }
}
