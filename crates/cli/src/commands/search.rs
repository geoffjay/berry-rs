//! Search command - search for memories.

use anyhow::Result;
use chrono::{DateTime, Utc};

use berry::{MemoryType, SearchRequest};

use crate::client::BerryClient;
use crate::output::{format_memory_table, print_error, print_info};

/// Search command arguments.
#[derive(Debug)]
pub struct SearchArgs {
    pub query: String,
    pub as_actor: Option<String>,
    pub memory_type: Option<MemoryType>,
    pub tags: Vec<String>,
    pub limit: usize,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub json_output: bool,
}

/// Run the search command.
pub async fn run(client: BerryClient, args: SearchArgs) -> Result<()> {
    let request = SearchRequest {
        query: args.query.clone(),
        as_actor: args.as_actor,
        memory_type: args.memory_type,
        tags: args.tags,
        limit: args.limit,
        from: args.from,
        to: args.to,
    };

    match client.search(request).await {
        Ok(memories) => {
            if args.json_output {
                println!("{}", serde_json::to_string_pretty(&memories)?);
            } else if memories.is_empty() {
                print_info(&format!("No memories found for query: {}", args.query));
            } else {
                println!(
                    "Found {} memor{} for \"{}\":\n",
                    memories.len(),
                    if memories.len() == 1 { "y" } else { "ies" },
                    args.query
                );
                println!("{}", format_memory_table(&memories));
            }
        }
        Err(e) => {
            print_error(&format!("Search failed: {}", e));
            return Err(e);
        }
    }

    Ok(())
}
