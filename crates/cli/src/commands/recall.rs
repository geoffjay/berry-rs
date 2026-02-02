//! Recall command - retrieve a memory by ID.

use anyhow::Result;

use crate::client::BerryClient;
use crate::output::{format_memory, print_error, print_info};

/// Recall command arguments.
#[derive(Debug)]
pub struct RecallArgs {
    pub id: String,
    pub json_output: bool,
}

/// Run the recall command.
pub async fn run(client: BerryClient, args: RecallArgs) -> Result<()> {
    match client.get_memory(&args.id).await {
        Ok(Some(memory)) => {
            if args.json_output {
                println!("{}", serde_json::to_string_pretty(&memory)?);
            } else {
                println!("{}", format_memory(&memory));
            }
        }
        Ok(None) => {
            print_info(&format!("Memory not found: {}", args.id));
        }
        Err(e) => {
            print_error(&format!("Failed to recall memory: {}", e));
            return Err(e);
        }
    }

    Ok(())
}
