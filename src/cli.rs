use crate::idish::IDish;
use clap::{Parser, Subcommand, ValueEnum};
use std::fmt;

#[derive(Parser)]
#[command(name = "pebbles")]
#[command(about = "Task tracking for AI-assisted development")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new pebbles repository
    Init,

    /// Create a new change
    #[command(next_help_heading = "Tasks")]
    New(NewArgs),

    /// List all changes
    #[command(visible_alias = "ls", next_help_heading = "Tasks")]
    List(ListArgs),

    /// Show details of a change
    #[command(next_help_heading = "Tasks")]
    Show {
        /// Change ID (or current change if not specified)
        id: Option<IDish>,
    },

    /// Update a change
    #[command(next_help_heading = "Tasks")]
    Update(UpdateArgs),

    /// Approve a change for work
    #[command(next_help_heading = "Tasks")]
    Approve {
        /// Change ID
        id: IDish,
    },

    /// Start working on a change
    #[command(visible_alias = "work")]
    Start {
        /// Change ID
        id: IDish,
        /// Create isolated workspace (ws-<id>/)
        #[arg(short, long)]
        isolate: bool,
        /// Don't auto-run /implement (opens opencode TUI instead)
        #[arg(long)]
        wait: bool,
        /// Print opencode output/logs to console
        #[arg(long)]
        print_logs: bool,
        /// Skip opencode permission prompts
        #[arg(long)]
        skip_permissions: bool,
        /// Verbose output - log CLI commands and ENV
        #[arg(short, long)]
        verbose: bool,
    },

    /// Mark a change as done
    #[command(next_help_heading = "Tasks")]
    Done {
        /// Change ID (or current change if not specified)
        id: Option<IDish>,
        /// Verify all acceptance criteria are checked
        #[arg(long)]
        auto: bool,
        /// Force mark done even if criteria not met
        #[arg(long)]
        force: bool,
    },

    /// Clean up a workspace
    Cleanup {
        /// Change ID (or current change if not specified)
        id: Option<IDish>,
    },

    /// Show event history for a change
    Log {
        /// Change ID (or current change if not specified)
        id: Option<IDish>,
    },

    /// Show current change (when in workspace)
    Current,

    /// Show workspace status including change details
    Status,

    /// Migrate legacy JSON storage to markdown files
    Migrate,

    /// Edit a change in your editor
    #[command(next_help_heading = "Tasks")]
    Edit {
        /// Change ID (or current change if not specified)
        id: Option<IDish>,
    },

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },

    /// Check for required dependencies
    Doctor,

    /// Delete a change
    #[command(
        visible_alias = "rm",
        visible_alias = "del",
        next_help_heading = "Tasks"
    )]
    Delete {
        /// Change ID
        id: IDish,
        /// Force deletion without confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// Add a blocking dependency to a change
    Block {
        /// Change ID that will be blocked
        change_id: IDish,
        /// Change ID that is the blocker (must be done first)
        dependency_id: IDish,
    },

    /// Remove a blocking dependency from a change
    Unblock {
        /// Change ID to unblock
        change_id: IDish,
        /// Change ID to remove as a dependency
        dependency_id: IDish,
    },

    /// Plan and break down a change into actionable steps
    #[command(next_help_heading = "Tasks")]
    Plan {
        /// Change ID (or current change if not specified)
        id: Option<IDish>,
        /// Don't auto-run /plan (opens opencode TUI instead)
        #[arg(long)]
        wait: bool,
    },

    /// Intake text from file or STDIN to create changes
    #[command(next_help_heading = "Tasks")]
    Intake {
        /// Path to file containing the text (reads from STDIN if not provided)
        file: Option<std::path::PathBuf>,
    },

    /// Manage time tracking for changes
    #[command(next_help_heading = "Tasks", visible_alias = "t")]
    Timer {
        #[command(subcommand)]
        command: TimerCommands,
    },

    /// List changes ready to work on (not blocked, not done)
    Ready,
}

#[derive(Subcommand)]
pub enum TimerCommands {
    /// Start the timer for a change
    Start {
        /// Change ID (or current change if not specified)
        id: Option<IDish>,
    },
    /// Stop the timer for a change
    Stop {
        /// Change ID (or current change if not specified)
        id: Option<IDish>,
    },
    /// Show timer status for a change
    Status {
        /// Change ID (or current change if not specified)
        id: Option<IDish>,
    },
}

#[derive(Parser)]
pub struct NewArgs {
    /// Change title
    pub title: Option<String>,

    /// Priority level
    #[arg(short, long, value_enum, default_value = "medium")]
    pub priority: PriorityArg,

    /// Initial body content
    #[arg(short, long)]
    pub body: Option<String>,

    /// Open editor to write body
    #[arg(short, long)]
    pub edit: bool,

    /// Parent change ID
    #[arg(long)]
    pub parent: Option<IDish>,
}

#[derive(Parser)]
pub struct ListArgs {
    /// Filter by status
    #[arg(short, long)]
    pub status: Option<String>,

    /// Filter by priority
    #[arg(short, long)]
    pub priority: Option<String>,

    /// Filter by changelog type
    #[arg(short, long)]
    pub changelog: Option<String>,

    /// Show all changes including done
    #[arg(short, long)]
    pub all: bool,

    /// Sort by field
    #[arg(short = 'S', long, default_value = "created")]
    pub sort: String,

    /// Reverse sort order
    #[arg(short, long)]
    pub reverse: bool,

    /// Display changes in a flat list instead of tree view
    #[arg(long)]
    pub flat: bool,
}

#[derive(Parser)]
pub struct UpdateArgs {
    /// Change ID (or current change if not specified)
    pub id: Option<IDish>,

    /// New title
    #[arg(short, long)]
    pub title: Option<String>,

    /// New body (prefix with @ to read from file, e.g., @file.txt)
    #[arg(short, long)]
    pub body: Option<String>,

    /// New priority
    #[arg(short, long)]
    pub priority: Option<PriorityArg>,

    /// New status
    #[arg(short, long)]
    pub status: Option<String>,

    /// Changelog type
    #[arg(short, long, value_enum)]
    pub changelog: Option<ChangelogTypeArg>,

    /// Open editor to modify body
    #[arg(short, long)]
    pub edit: bool,

    /// New parent change ID (use empty string to remove parent)
    #[arg(long)]
    pub parent: Option<IDish>,
}

#[derive(Clone, ValueEnum)]
pub enum PriorityArg {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Clone, ValueEnum)]
pub enum ChangelogTypeArg {
    Feature,
    Fix,
    Change,
    Deprecated,
    Removed,
    Security,
    Internal,
}

impl fmt::Display for PriorityArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PriorityArg::Low => write!(f, "low"),
            PriorityArg::Medium => write!(f, "medium"),
            PriorityArg::High => write!(f, "high"),
            PriorityArg::Critical => write!(f, "critical"),
        }
    }
}

#[derive(Clone, ValueEnum)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
}

impl Shell {
    pub fn to_clap_shell(&self) -> clap_complete::Shell {
        match self {
            Shell::Bash => clap_complete::Shell::Bash,
            Shell::Zsh => clap_complete::Shell::Zsh,
            Shell::Fish => clap_complete::Shell::Fish,
        }
    }
}
