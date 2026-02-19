use crate::commands::{print_info, resolve_id};
use crate::config::{get_config_path, get_db_path, Config};
use crate::idish::IDish;
use crate::repository::ChangeRepository;
use crate::vcs::detect_vcs_with_preference;
use anyhow::{Context, Result};
use colored::Colorize;
use std::process::Command;

pub async fn plan(id: Option<IDish>, wait: bool) -> Result<()> {
    let db_path = get_db_path()
        .context("Not in a pebbles repository. Run 'pebbles init' first.")?;

    let db = crate::db::Db::open(&db_path).await?;
    let full_id = match id {
        Some(id) => id.resolve(&db).map_err(|e| anyhow::anyhow!(e))?,
        None => resolve_id(None).await?,
    };

    let repo = ChangeRepository::open(db_path).await?;

    let change = repo
        .find_by_id(&full_id)
        .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", full_id))?;

    let config_path = get_config_path().unwrap();
    let config = Config::load(&config_path).await?;

    let vcs = detect_vcs_with_preference(&config.vcs.prefer)
        .context("No version control system detected (git or jujutsu)")?;

    // Display the change
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

    if !wait {
        print_info(&format!(
            "Launching opencode to plan change '{}'",
            full_id
        ));
    } else {
        print_info(&format!("Launching opencode for change '{}'", full_id));
    }

    let mut cmd = Command::new("opencode");
    cmd.env("PEBBLES_CHANGE", full_id.to_string());
    cmd.env("PEBBLES_VCS", vcs.name());

    if !wait {
        cmd.args(["run", "/plan"]);
    }

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let _ = cmd.exec();
        anyhow::bail!("Failed to launch opencode");
    }

    #[cfg(not(unix))]
    {
        let mut child = cmd
            .spawn()
            .context("Failed to launch opencode. Is it installed?")?;
        child.wait()?;
        Ok(())
    }
}
