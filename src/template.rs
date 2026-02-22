use anyhow::{Context, Result};
use std::path::Path;

/// Template names for OpenCode commands
pub const IMPLEMENT_TEMPLATE: &str = include_str!("../templates/opencode/implement.md");
pub const DESCRIBE_TEMPLATE: &str = include_str!("../templates/opencode/describe.md");
pub const PLAN_TEMPLATE: &str = include_str!("../templates/opencode/plan.md");
pub const INTAKE_TEMPLATE: &str = include_str!("../templates/opencode/intake.md");

/// Template names for issue creation
pub const NEW_ISSUE_TEMPLATE: &str = include_str!("../templates/new_issue.md");

/// Write all OpenCode command templates to the repository
pub async fn write_opencode_templates(repo_root: &Path) -> Result<()> {
    let opencode_dir = repo_root.join(".opencode").join("commands");

    tokio::fs::create_dir_all(&opencode_dir)
        .await
        .context("Failed to create .opencode/commands directory")?;

    // Write implement command
    tokio::fs::write(opencode_dir.join("implement.md"), IMPLEMENT_TEMPLATE)
        .await
        .context("Failed to write implement.md template")?;

    // Write describe command
    tokio::fs::write(opencode_dir.join("describe.md"), DESCRIBE_TEMPLATE)
        .await
        .context("Failed to write describe.md template")?;

    // Write plan command
    tokio::fs::write(opencode_dir.join("plan.md"), PLAN_TEMPLATE)
        .await
        .context("Failed to write plan.md template")?;

    // Write intake command
    tokio::fs::write(opencode_dir.join("intake.md"), INTAKE_TEMPLATE)
        .await
        .context("Failed to write intake.md template")?;

    Ok(())
}

/// Get the new issue template content
pub fn get_new_issue_template() -> &'static str {
    NEW_ISSUE_TEMPLATE
}
