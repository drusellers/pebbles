use anyhow::Result;
use clap::Parser;

mod cli;
mod commands;
mod config;
mod db;
mod idish;
mod models;
mod repository;
mod table;
mod vcs;

use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => commands::init().await,
        Commands::New(args) => commands::new(args).await,
        Commands::List(args) => commands::list(args).await,
        Commands::Show { id } => commands::show(id).await,
        Commands::Update(args) => commands::update(args).await,
        Commands::Approve { id } => commands::approve(id).await,
        Commands::Work { id, skip_permissions } => {
            commands::work(id, skip_permissions).await
        }
        Commands::Done { id, auto, force } => commands::done(id, auto, force).await,
        Commands::Cleanup { id } => commands::cleanup(id).await,
        Commands::Build { id, skip_permissions } => commands::build(id, skip_permissions).await,
        Commands::Log { id } => commands::log(id).await,
        Commands::Current => commands::current().await,
        Commands::Edit { id } => commands::edit(id).await,
        Commands::Completions { shell } => commands::completions(shell),
        Commands::Doctor => commands::doctor().await,
        Commands::Delete { id, force } => commands::delete(id, force).await,
    }
}
