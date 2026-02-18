use clap::{Parser, Subcommand, ValueEnum};

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
    New(NewArgs),

    /// List all changes
    List(ListArgs),

    /// Show details of a change
    Show {
        /// Change ID (or current change if not specified)
        id: Option<String>,
    },

    /// Update a change
    Update(UpdateArgs),

    /// Approve a change for work
    Approve {
        /// Change ID
        id: String,
    },

    /// Start working on a change
    Work {
        /// Change ID
        id: String,
        /// Skip opencode permission prompts
        #[arg(long)]
        skip_permissions: bool,
    },

    /// Mark a change as done
    Done {
        /// Change ID (or current change if not specified)
        id: Option<String>,
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
        id: Option<String>,
    },

    /// Show event history for a change
    Log {
        /// Change ID (or current change if not specified)
        id: Option<String>,
    },

    /// Show current change (when in workspace)
    Current,

    /// Edit a change in your editor
    Edit {
        /// Change ID (or current change if not specified)
        id: Option<String>,
    },

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },

    /// Check for required dependencies
    Doctor,
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
    #[arg(short, long)]
    pub parent: Option<String>,
}

#[derive(Parser)]
pub struct ListArgs {
    /// Filter by status
    #[arg(short, long)]
    pub status: Option<String>,

    /// Filter by priority
    #[arg(short, long)]
    pub priority: Option<String>,

    /// Show all changes including done
    #[arg(short, long)]
    pub all: bool,

    /// Sort by field
    #[arg(short, long, default_value = "created")]
    pub sort: String,

    /// Reverse sort order
    #[arg(short, long)]
    pub reverse: bool,
}

#[derive(Parser)]
pub struct UpdateArgs {
    /// Change ID (or current change if not specified)
    pub id: Option<String>,

    /// New title
    #[arg(short, long)]
    pub title: Option<String>,

    /// New body
    #[arg(short, long)]
    pub body: Option<String>,

    /// New priority
    #[arg(short, long)]
    pub priority: Option<PriorityArg>,

    /// New status
    #[arg(short, long)]
    pub status: Option<String>,

    /// Open editor to modify body
    #[arg(short, long)]
    pub edit: bool,
}

#[derive(Clone, ValueEnum)]
pub enum PriorityArg {
    Low,
    Medium,
    High,
    Critical,
}

impl PriorityArg {
    pub fn to_string(&self) -> String {
        match self {
            PriorityArg::Low => "low".to_string(),
            PriorityArg::Medium => "medium".to_string(),
            PriorityArg::High => "high".to_string(),
            PriorityArg::Critical => "critical".to_string(),
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
