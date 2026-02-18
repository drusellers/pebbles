use anyhow::{Context, Result};
use rand::{thread_rng, Rng};

use crate::cli::NewArgs;
use crate::commands::{print_info, print_success};
use crate::config::{get_db_path, get_config_path};
use crate::models::{Change, Priority};
use crate::repository::ChangeRepository;

const ALPHANUMERIC: &str = "abcdefghijklmnopqrstuvwxyz0123456789";

pub async fn new(args: NewArgs) -> Result<()> {
    let db_path = get_db_path()
        .context("Not in a pebbles repository. Run 'pebbles init' first.")?;
    
    let mut repo = ChangeRepository::open(db_path).await?;
    
    // Generate unique ID
    let id = generate_unique_id(&repo).await?;
    
    // Get title
    let title = if let Some(title) = args.title {
        title
    } else {
        dialoguer::Input::new()
            .with_prompt("Title")
            .interact_text()
            .context("Failed to read title")?
    };
    
    let priority: Priority = args.priority.into();
    
    // Create change
    let mut change = Change::new(id.clone(), title, priority);
    
    // Set parent if provided
    if let Some(parent) = args.parent {
        change.parent = Some(parent);
    }
    
    // Handle body
    let body = if args.edit {
        edit_in_editor("", &get_config_path().unwrap()).await?
    } else if let Some(body) = args.body {
        body
    } else {
        String::new()
    };
    change.body = body;
    
    // Save
    repo.create(change).await?;
    
    print_success(&format!("Created change {}: {}", id, repo.find_by_id(&id).unwrap().title));
    
    if args.edit {
        print_info("Use 'pebbles edit <id>' to edit the body later");
    }
    
    Ok(())
}

fn generate_id() -> String {
    let mut rng = thread_rng();
    (0..4)
        .map(|_| {
            let idx = rng.gen_range(0..ALPHANUMERIC.len());
            ALPHANUMERIC.chars().nth(idx).unwrap()
        })
        .collect()
}

async fn generate_unique_id(repo: &ChangeRepository) -> Result<String> {
    for _ in 0..100 {
        let id = generate_id();
        if repo.find_by_id(&id).is_none() {
            return Ok(id);
        }
    }
    anyhow::bail!("Failed to generate unique ID after 100 attempts")
}

async fn edit_in_editor(initial: &str, config_path: &std::path::Path) -> Result<String> {
    use crate::config::Config;
    
    let config = Config::load(config_path).await?;
    let editor = config.get_editor();
    
    let temp_file = tempfile::NamedTempFile::new()?;
    let temp_path = temp_file.path().to_path_buf();
    
    // Write initial content
    tokio::fs::write(&temp_path, initial).await?;
    
    // Launch editor
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("{} {}", editor, temp_path.display()))
        .status()
        .context("Failed to launch editor")?;
    
    if !status.success() {
        anyhow::bail!("Editor exited with non-zero status");
    }
    
    // Read back
    let content = tokio::fs::read_to_string(&temp_path).await?;
    Ok(content)
}
