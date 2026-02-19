use crate::commands::{print_info, print_success};
use crate::config::{get_config_path, get_db_path, Config};
use crate::db::Db;
use crate::idish::IDish;
use crate::models::Status;
use crate::repository::ChangeRepository;
use crate::vcs::detect_vcs_with_preference;
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

pub async fn start(id: IDish, isolate: bool, wait: bool, print_logs: bool, skip_permissions: bool) -> Result<()> {
    let db_path = get_db_path()
        .context("Not in a pebbles repository. Run 'pebbles init' first.")?;

    let db = Db::open(&db_path).await?;
    let full_id = id.resolve(&db).map_err(|e| anyhow::anyhow!(e))?;

    let mut repo = ChangeRepository::open(db_path).await?;

    let change = repo.find_by_id(&full_id)
        .ok_or_else(|| anyhow::anyhow!("Change '{}' not found", full_id))?;

    match change.status {
        Status::Draft => {
            anyhow::bail!(
                "Change '{}' is in Draft status. Approve it first with 'pebbles approve {}'",
                full_id, full_id
            );
        }
        Status::Approved | Status::InProgress => {}
        Status::Done => {
            anyhow::bail!("Change '{}' is already done", full_id);
        }
        _ => {}
    }

    let config_path = get_config_path().unwrap();
    let config = Config::load(&config_path).await?;

    let vcs = detect_vcs_with_preference(&config.vcs.prefer)
        .context("No version control system detected (git or jujutsu)")?;

    print_info(&format!("Using {} for version control", vcs.name()));

    let work_dir: PathBuf = if isolate {
        let workspace_path = vcs.create_workspace(&full_id)?;
        print_success(&format!("Created workspace at {}", workspace_path.display()));
        workspace_path
    } else {
        std::env::current_dir()?
    };

    if change.status != Status::InProgress {
        repo.update_status(&full_id, Status::InProgress).await?;
        print_info("Updated status to InProgress");
    }

    if !wait {
        print_info(&format!(
            "Launching opencode to implement change '{}'",
            full_id
        ));
    } else {
        print_info(&format!("Launching opencode for change '{}'", full_id));
    }

    let mut cmd = Command::new("opencode");
    cmd.current_dir(&work_dir);
    cmd.env("PEBBLES_CHANGE", full_id.to_string());
    cmd.env("PEBBLES_VCS", vcs.name());

    if skip_permissions {
        cmd.env("OPENCODE_SKIP_PERMISSIONS", "1");
    }

    if !wait {
        cmd.args(["run", "/implement"]);
        if print_logs {
            cmd.arg("--print-logs");
        }
    }

    tracing::trace!("ENV: PEBBLES_CHANGE={}", full_id);
    tracing::trace!("ENV: PEBBLES_VCS={}", vcs.name());
    if skip_permissions {
        tracing::trace!("ENV: OPENCODE_SKIP_PERMISSIONS=1");
    }
    if print_logs {
        tracing::trace!("ARG: --print-logs");
    }
    tracing::trace!("CLI: {:?}", cmd);

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let _ = cmd.exec();
        anyhow::bail!("Failed to launch opencode");
    }

    #[cfg(not(unix))]
    {
        use std::process::Stdio;
        if print_logs {
            cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
        }
        let mut child = cmd.spawn()
            .context("Failed to launch opencode. Is it installed?")?;
        child.wait()?;
        Ok(())
    }
}
