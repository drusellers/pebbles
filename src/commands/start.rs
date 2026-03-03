use crate::commands::{print_info, print_success};
use crate::config::{get_config_path, Config};
use crate::harness::{detect_harness_with_preference, HarnessContext};
use crate::idish::IDish;
use crate::models::Status;
use crate::repository::ChangeRepository;
use crate::vcs::detect_vcs_with_preference;
use anyhow::{Context, Result};
use std::path::PathBuf;

pub async fn start(id: IDish, isolate: bool, wait: bool, print_logs: bool, skip_permissions: bool) -> Result<()> {
    let mut repo = ChangeRepository::open().await?;

    let full_id = id.resolve(&repo.db)?;

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

    let config_path = get_config_path()?;
    let config = Config::load(&config_path).await?;

    let vcs = detect_vcs_with_preference()
        .await?
        .context("No version control system detected (git or jujutsu)")?;

    let harness = detect_harness_with_preference(config.harness.prefer)
        .context("No AI harness detected (opencode)")?;

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
            "Launching {} to implement change '{}'",
            harness.name(),
            full_id
        ));
    } else {
        print_info(&format!("Launching {} for change '{}'", harness.name(), full_id));
    }

    let ctx = HarnessContext::new(vcs.name(), work_dir)
        .with_change_id(full_id.clone())
        .with_skip_permissions(skip_permissions || config.work.skip_permissions)
        .with_print_logs(print_logs)
        .with_wait_mode(wait);

    tracing::trace!("ENV: PEBBLES_CHANGE={}", full_id);
    tracing::trace!("ENV: PEBBLES_VCS={}", vcs.name());
    if skip_permissions || config.work.skip_permissions {
        tracing::trace!("ENV: OPENCODE_SKIP_PERMISSIONS=1");
    }
    if print_logs {
        tracing::trace!("ARG: --print-logs");
    }

    harness.implement(&ctx)
}
