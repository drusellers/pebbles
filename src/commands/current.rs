use anyhow::{Context, Result};

use crate::commands::print_warning;
use crate::vcs::detect_vcs;

pub async fn current() -> Result<()> {
    let vcs = detect_vcs()
        .context("No version control system detected")?;
    
    if let Some(id) = vcs.current_workspace_id() {
        println!("{}", id);
    } else {
        print_warning("Not in a pebbles workspace");
        std::process::exit(1);
    }
    
    Ok(())
}
