mod api;
mod commands;
mod config;
mod output;

use anyhow::Result;
use clap::{Parser, Subcommand};
use commands::{access, auth, items, sources};

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

    /// Use specific session ID (hidden, used by agents)
    #[arg(long, global = true, hide = true)]
    session: Option<String>,

    /// Disable session tracking (hidden, used by book-enricher)
    #[arg(long, global = true, hide = true)]
    no_session: bool,
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
    /// Source management commands
    Sources {
        #[command(subcommand)]
        command: SourcesCommands,
    },
    /// Access session tracking (hidden, used by agents)
    #[command(hide = true)]
    Access {
        #[command(subcommand)]
        command: AccessCommands,
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
    /// Create a new markdown document
    Create {
        /// Document title
        title: String,
        /// Description
        #[arg(long, short)]
        description: Option<String>,
        /// Initial content
        #[arg(long, short)]
        content: Option<String>,
    },
    /// Get full content of a document (outputs to stdout)
    Get {
        /// Item ID
        id: String,
    },
    /// Replace document content (from file or stdin)
    Put {
        /// Item ID
        id: String,
        /// Read content from file
        #[arg(long, short)]
        file: Option<String>,
    },
}

#[derive(Subcommand)]
enum SourcesCommands {
    /// List saved sources
    List {
        /// Maximum number of sources to return
        #[arg(long)]
        limit: Option<u32>,
    },
    /// Delete sources
    Delete {
        /// Comma-separated source IDs
        ids: String,
        /// Skip confirmation prompt
        #[arg(long, short)]
        yes: bool,
    },
}

#[derive(Subcommand)]
enum AccessCommands {
    /// Start a new research session
    Start {
        /// Research intent/question
        #[arg(long)]
        intent: Option<String>,
    },
    /// Complete the current research session
    Complete,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Auth { command } => match command {
            AuthCommands::Login => auth::login(cli.session.clone(), cli.no_session).await?,
            AuthCommands::Logout => auth::logout()?,
            AuthCommands::Whoami => auth::whoami(cli.json, cli.session.clone(), cli.no_session).await?,
        },
        Commands::Items { command } => match command {
            ItemsCommands::List => items::list(cli.json, cli.session.clone(), cli.no_session).await?,
            ItemsCommands::Toc { ids } => items::toc(&ids, cli.json, cli.session.clone(), cli.no_session).await?,
            ItemsCommands::Read { ids } => items::read(&ids, cli.json, cli.session.clone(), cli.no_session).await?,
            ItemsCommands::Add { file } => items::add(&file, cli.session.clone(), cli.no_session).await?,
            ItemsCommands::Remove { ids, yes } => items::remove(&ids, yes, cli.session.clone(), cli.no_session).await?,
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
                    cli.session.clone(),
                    cli.no_session,
                )
                .await?
            }
            ItemsCommands::Flag { id } => items::flag(&id, cli.session.clone(), cli.no_session).await?,
            ItemsCommands::Create {
                title,
                description,
                content,
            } => {
                items::create(&title, description.as_deref(), content.as_deref(), cli.json, cli.session.clone(), cli.no_session).await?
            }
            ItemsCommands::Get { id } => items::get(&id, cli.session.clone(), cli.no_session).await?,
            ItemsCommands::Put { id, file } => items::put(&id, file.as_deref(), cli.session.clone(), cli.no_session).await?,
        },
        Commands::Sources { command } => match command {
            SourcesCommands::List { limit } => sources::list(cli.json, limit, cli.session.clone(), cli.no_session).await?,
            SourcesCommands::Delete { ids, yes } => sources::delete(&ids, yes, cli.session.clone(), cli.no_session).await?,
        },
        Commands::Access { command } => match command {
            AccessCommands::Start { intent } => {
                access::start(intent.as_deref(), cli.json, cli.session.clone(), cli.no_session).await?
            }
            AccessCommands::Complete => {
                access::complete(cli.json, cli.session.clone(), cli.no_session).await?
            }
        },
    }

    Ok(())
}
