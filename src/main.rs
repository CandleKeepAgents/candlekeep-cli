mod api;
mod commands;
mod config;
mod output;

use anyhow::Result;
use clap::{Parser, Subcommand};
use commands::{auth, items};

#[derive(Parser)]
#[command(name = "ck")]
#[command(about = "CandleKeep CLI - Manage your document library", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output in JSON format
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Authentication commands
    Auth {
        #[command(subcommand)]
        command: AuthCommands,
    },
    /// Item management commands
    Items {
        #[command(subcommand)]
        command: ItemsCommands,
    },
}

#[derive(Subcommand)]
enum AuthCommands {
    /// Login via browser authentication
    Login,
    /// Remove stored credentials
    Logout,
    /// Show current user information
    Whoami,
}

#[derive(Subcommand)]
enum ItemsCommands {
    /// List all items in your library
    List,
    /// Show table of contents for items
    Toc {
        /// Comma-separated item IDs
        ids: String,
    },
    /// Read content from items
    Read {
        /// Item IDs with page ranges (e.g., "id:1-5,id2:all")
        ids: String,
    },
    /// Upload a PDF to your library
    Add {
        /// Path to PDF file
        file: String,
    },
    /// Remove items from your library
    Remove {
        /// Comma-separated item IDs
        ids: String,
        /// Skip confirmation prompt
        #[arg(long, short)]
        yes: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Auth { command } => match command {
            AuthCommands::Login => auth::login().await?,
            AuthCommands::Logout => auth::logout()?,
            AuthCommands::Whoami => auth::whoami(cli.json).await?,
        },
        Commands::Items { command } => match command {
            ItemsCommands::List => items::list(cli.json).await?,
            ItemsCommands::Toc { ids } => items::toc(&ids, cli.json).await?,
            ItemsCommands::Read { ids } => items::read(&ids, cli.json).await?,
            ItemsCommands::Add { file } => items::add(&file).await?,
            ItemsCommands::Remove { ids, yes } => items::remove(&ids, yes).await?,
        },
    }

    Ok(())
}
