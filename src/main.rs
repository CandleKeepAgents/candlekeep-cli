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
    /// Enrich item metadata (title, author, description, table of contents)
    Enrich {
        /// Item ID
        id: String,
        /// New title
        #[arg(long)]
        title: Option<String>,
        /// Author name
        #[arg(long)]
        author: Option<String>,
        /// Description
        #[arg(long)]
        description: Option<String>,
        /// Confidence score (0.0-1.0)
        #[arg(long)]
        confidence: Option<f64>,
        /// Table of contents as JSON array: [{"title":"Chapter 1","page":1,"level":1}]
        #[arg(long)]
        toc: Option<String>,
    },
    /// Flag item as needing metadata enrichment
    Flag {
        /// Item ID
        id: String,
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
            ItemsCommands::Enrich {
                id,
                title,
                author,
                description,
                confidence,
                toc,
            } => {
                items::enrich(
                    &id,
                    title.as_deref(),
                    author.as_deref(),
                    description.as_deref(),
                    confidence,
                    toc.as_deref(),
                )
                .await?
            }
            ItemsCommands::Flag { id } => items::flag(&id).await?,
        },
    }

    Ok(())
}
