//! Berry CLI - Command-line interface for the Berry memory system.

use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand, ValueEnum};

use berry::config::load_config;
use berry::{MemoryType, VisibilityLevel};

mod client;
mod commands;
mod output;

use client::BerryClient;

/// Berry - A semantic memory system for AI assistants.
#[derive(Parser)]
#[command(name = "berry")]
#[command(version, about, long_about = None)]
struct Cli {
    /// Output format
    #[arg(long, global = true, default_value = "text")]
    format: OutputFormat,

    #[command(subcommand)]
    command: Option<Commands>,
}

/// Output format options.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

/// CLI subcommands.
#[derive(Subcommand)]
enum Commands {
    /// Store a new memory
    Remember {
        /// Content of the memory
        content: Option<String>,

        /// Memory type
        #[arg(short = 't', long, value_parser = parse_memory_type)]
        r#type: Option<MemoryType>,

        /// Tags (comma-separated)
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,

        /// Creator identifier
        #[arg(long = "by", default_value = "user")]
        created_by: String,

        /// References to other memory IDs (comma-separated)
        #[arg(short = 'r', long, value_delimiter = ',')]
        references: Vec<String>,

        /// Visibility level
        #[arg(short = 'v', long, value_parser = parse_visibility)]
        visibility: Option<VisibilityLevel>,

        /// Share with these actors (comma-separated)
        #[arg(long = "shared-with", value_delimiter = ',')]
        shared_with: Vec<String>,
    },

    /// Retrieve a memory by ID
    Recall {
        /// Memory ID to retrieve
        id: String,
    },

    /// Delete a memory
    Forget {
        /// Memory ID to delete
        id: String,

        /// Skip confirmation prompt
        #[arg(short = 'f', long)]
        force: bool,
    },

    /// Search for memories
    Search {
        /// Search query
        query: String,

        /// Actor for visibility filtering
        #[arg(short = 'a', long = "as-actor")]
        as_actor: Option<String>,

        /// Filter by memory type
        #[arg(short = 't', long, value_parser = parse_memory_type)]
        r#type: Option<MemoryType>,

        /// Filter by tags (comma-separated)
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,

        /// Maximum number of results
        #[arg(short = 'l', long, default_value = "10")]
        limit: usize,

        /// Start date (ISO 8601)
        #[arg(long, value_parser = parse_datetime)]
        from: Option<DateTime<Utc>>,

        /// End date (ISO 8601)
        #[arg(long, value_parser = parse_datetime)]
        to: Option<DateTime<Utc>>,
    },

    /// Start the HTTP server
    Serve {
        /// Port to listen on
        #[arg(short = 'p', long, default_value = "4114")]
        port: u16,

        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// Run in foreground (always runs in foreground now)
        #[arg(short = 'f', long)]
        foreground: bool,
    },

    /// Start the MCP server
    Mcp {
        /// Berry server URL
        #[arg(short = 's', long = "server-url")]
        server_url: Option<String>,

        /// Verbose logging
        #[arg(short = 'v', long)]
        verbose: bool,
    },

    /// Manage documents
    Doc {
        #[command(subcommand)]
        action: DocCommands,
    },

    /// Initialize configuration
    Init {
        /// Overwrite existing config
        #[arg(long)]
        force: bool,
    },

    /// Migrate memories to a new collection with current embedding model
    Migrate {
        /// Dry run - only show what would be migrated
        #[arg(long)]
        dry_run: bool,

        /// Target collection name (defaults to <current>_migrated)
        #[arg(long = "collection")]
        new_collection: Option<String>,

        /// Migrate from ChromaDB to LanceDB
        #[arg(long)]
        to_lance: bool,
    },
}

/// Document subcommands.
#[derive(Subcommand)]
enum DocCommands {
    /// Create a new document
    Create {
        /// Document title
        title: String,

        /// Document content (markdown)
        #[arg(short = 'c', long)]
        content: Option<String>,

        /// Tags (comma-separated)
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,

        /// Creator identifier
        #[arg(long = "by", default_value = "user")]
        created_by: String,
    },

    /// Read a document by ID
    Read {
        /// Document ID (slug)
        id: String,
    },

    /// Update a document
    Update {
        /// Document ID (slug)
        id: String,

        /// New title
        #[arg(short = 't', long)]
        title: Option<String>,

        /// New content (markdown)
        #[arg(short = 'c', long)]
        content: Option<String>,

        /// New tags (comma-separated, replaces existing)
        #[arg(long, value_delimiter = ',')]
        tags: Option<Vec<String>>,
    },

    /// Delete a document
    Delete {
        /// Document ID (slug)
        id: String,

        /// Skip confirmation prompt
        #[arg(short = 'f', long)]
        force: bool,
    },

    /// List documents
    List {
        /// Filter by tags (comma-separated)
        #[arg(long, value_delimiter = ',')]
        tags: Option<Vec<String>>,

        /// Filter by creator
        #[arg(long = "by")]
        created_by: Option<String>,
    },
}

/// Parse memory type from string.
fn parse_memory_type(s: &str) -> Result<MemoryType, String> {
    s.parse()
}

/// Parse visibility level from string.
fn parse_visibility(s: &str) -> Result<VisibilityLevel, String> {
    s.parse()
}

/// Parse datetime from string.
fn parse_datetime(s: &str) -> Result<DateTime<Utc>, String> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| format!("Invalid date format: {}. Use ISO 8601 format.", e))
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    berry::logging::init();

    let cli = Cli::parse();
    let json_output = matches!(cli.format, OutputFormat::Json);

    // Load configuration
    let config = load_config().unwrap_or_default();

    match cli.command {
        Some(Commands::Remember {
            content,
            r#type,
            tags,
            created_by,
            references,
            visibility,
            shared_with,
        }) => {
            let client = BerryClient::new(&config.server.url, config.server.timeout)?;
            let args = commands::remember::RememberArgs {
                content,
                memory_type: r#type,
                tags,
                created_by,
                references,
                visibility,
                shared_with,
                json_output,
            };
            commands::remember(client, args).await
        }

        Some(Commands::Recall { id }) => {
            let client = BerryClient::new(&config.server.url, config.server.timeout)?;
            let args = commands::recall::RecallArgs { id, json_output };
            commands::recall(client, args).await
        }

        Some(Commands::Forget { id, force }) => {
            let client = BerryClient::new(&config.server.url, config.server.timeout)?;
            let args = commands::forget::ForgetArgs {
                id,
                force,
                json_output,
            };
            commands::forget(client, args).await
        }

        Some(Commands::Search {
            query,
            as_actor,
            r#type,
            tags,
            limit,
            from,
            to,
        }) => {
            let client = BerryClient::new(&config.server.url, config.server.timeout)?;
            let args = commands::search::SearchArgs {
                query,
                as_actor,
                memory_type: r#type,
                tags,
                limit,
                from,
                to,
                json_output,
            };
            commands::search(client, args).await
        }

        Some(Commands::Doc { action }) => {
            let client = BerryClient::new(&config.server.url, config.server.timeout)?;
            let doc_action = match action {
                DocCommands::Create {
                    title,
                    content,
                    tags,
                    created_by,
                } => commands::doc::DocAction::Create {
                    title,
                    content,
                    tags,
                    created_by,
                },
                DocCommands::Read { id } => commands::doc::DocAction::Read { id },
                DocCommands::Update {
                    id,
                    title,
                    content,
                    tags,
                } => commands::doc::DocAction::Update {
                    id,
                    title,
                    content,
                    tags,
                },
                DocCommands::Delete { id, force } => commands::doc::DocAction::Delete { id, force },
                DocCommands::List { tags, created_by } => {
                    commands::doc::DocAction::List { tags, created_by }
                }
            };
            let args = commands::doc::DocArgs {
                action: doc_action,
                json_output,
            };
            commands::doc(client, args).await
        }

        Some(Commands::Serve {
            port,
            host,
            foreground,
        }) => {
            let args = commands::serve::ServeArgs {
                port,
                host,
                foreground,
            };
            commands::serve(args).await
        }

        Some(Commands::Mcp {
            server_url,
            verbose,
        }) => {
            let args = commands::mcp::McpArgs {
                server_url,
                verbose,
            };
            commands::mcp(args).await
        }

        Some(Commands::Init { force }) => {
            let args = commands::init::InitArgs { force };
            commands::init(args).await
        }

        Some(Commands::Migrate {
            dry_run,
            new_collection,
            to_lance,
        }) => {
            let args = commands::migrate::MigrateArgs {
                dry_run,
                new_collection,
                to_lance,
            };
            commands::migrate(args).await
        }

        None => {
            // Interactive mode - show help for now
            println!("Berry - A semantic memory system for AI assistants\n");
            println!("Run 'berry --help' for usage information.");
            println!("Run 'berry init' to initialize configuration.");
            Ok(())
        }
    }
}
