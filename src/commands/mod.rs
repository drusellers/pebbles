use anyhow::Result;
use colored::Colorize;

pub mod init;
pub mod new;
pub mod list;
pub mod show;
pub mod update;
pub mod approve;
pub mod work;
pub mod done;
pub mod cleanup;
pub mod log;
pub mod current;
pub mod edit;
pub mod completions;
pub mod doctor;

pub use init::init;
pub use new::new;
pub use list::list;
pub use show::show;
pub use update::update;
pub use approve::approve;
pub use work::work;
pub use done::done;
pub use cleanup::cleanup;
pub use log::log;
pub use current::current;
pub use edit::edit;
pub use completions::completions;
pub use doctor::doctor;

/// Resolve change ID from either explicit argument or current workspace.
fn resolve_id(id: Option<String>) -> Result<String> {
    match id {
        Some(id) => Ok(id),
        None => {
            // Try to detect current workspace
            if let Some(vcs) = crate::vcs::detect_vcs() {
                if let Some(current_id) = vcs.current_workspace_id() {
                    return Ok(current_id);
                }
            }
            
            anyhow::bail!(
                "No change ID provided and not in a workspace.\n\
                 Either provide a change ID or run from a workspace directory (ws-{{id}})."
            )
        }
    }
}

fn print_success(msg: &str) {
    println!("{} {}", "✓".green(), msg);
}

fn print_info(msg: &str) {
    println!("{} {}", "→".blue(), msg);
}

fn print_warning(msg: &str) {
    println!("{} {}", "!".yellow(), msg);
}

fn print_error(msg: &str) {
    eprintln!("{} {}", "✗".red(), msg);
}
