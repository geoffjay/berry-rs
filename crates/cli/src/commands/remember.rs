//! Remember command - store a new memory.

use anyhow::Result;
use inquire::{Select, Text};

use berry::{CreateMemoryRequest, MemoryType, VisibilityLevel};

use crate::client::BerryClient;
use crate::output::{format_memory, print_error, print_success};

/// Remember command arguments.
#[derive(Debug)]
pub struct RememberArgs {
    pub content: Option<String>,
    pub memory_type: Option<MemoryType>,
    pub tags: Vec<String>,
    pub created_by: String,
    pub references: Vec<String>,
    pub visibility: Option<VisibilityLevel>,
    pub shared_with: Vec<String>,
    pub json_output: bool,
}

/// Run the remember command.
pub async fn run(client: BerryClient, args: RememberArgs) -> Result<()> {
    // Get content interactively if not provided
    let content = match args.content {
        Some(c) => c,
        None => Text::new("What would you like to remember?")
            .with_help_message("Enter the content of your memory")
            .prompt()?,
    };

    // Get memory type interactively if not provided
    let memory_type = match args.memory_type {
        Some(t) => t,
        None => {
            let options = vec!["information", "question", "request"];
            let selection = Select::new("What type of memory is this?", options)
                .with_help_message("Select the type of memory")
                .prompt()?;
            selection.parse::<MemoryType>().map_err(|e| anyhow::anyhow!(e))?
        }
    };

    // Get tags interactively if not provided
    let tags = if args.tags.is_empty() {
        let input = Text::new("Tags (comma-separated, or leave empty)")
            .with_help_message("Enter tags separated by commas")
            .prompt_skippable()?;
        match input {
            Some(t) if !t.is_empty() => t.split(',').map(|s| s.trim().to_string()).collect(),
            _ => vec![],
        }
    } else {
        args.tags
    };

    // Get visibility interactively if not provided
    let visibility = match args.visibility {
        Some(v) => v,
        None => {
            let options = vec!["public", "shared", "private"];
            let selection = Select::new("Visibility level?", options)
                .with_help_message("Who can see this memory?")
                .prompt()?;
            selection.parse::<VisibilityLevel>().map_err(|e| anyhow::anyhow!(e))?
        }
    };

    // Get shared_with if visibility is shared
    let shared_with = if visibility == VisibilityLevel::Shared && args.shared_with.is_empty() {
        let input = Text::new("Share with (comma-separated actor IDs)")
            .with_help_message("Enter actor IDs to share with")
            .prompt()?;
        input.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        args.shared_with
    };

    let request = CreateMemoryRequest {
        content,
        memory_type,
        tags,
        created_by: args.created_by,
        references: args.references,
        visibility,
        shared_with,
    };

    match client.create_memory(request).await {
        Ok(memory) => {
            if args.json_output {
                println!("{}", serde_json::to_string_pretty(&memory)?);
            } else {
                print_success(&format!("Memory created: {}", memory.id));
                println!("\n{}", format_memory(&memory));
            }
        }
        Err(e) => {
            print_error(&format!("Failed to create memory: {}", e));
            return Err(e);
        }
    }

    Ok(())
}
