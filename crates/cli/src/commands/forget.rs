//! Forget command - delete a memory by ID.

use anyhow::Result;
use inquire::Confirm;

use crate::client::BerryClient;
use crate::output::{print_error, print_info, print_success};

/// Forget command arguments.
#[derive(Debug)]
pub struct ForgetArgs {
    pub id: String,
    pub force: bool,
    pub json_output: bool,
}

/// Run the forget command.
pub async fn run(client: BerryClient, args: ForgetArgs) -> Result<()> {
    // Confirm deletion unless force flag is set
    if !args.force {
        let confirm = Confirm::new(&format!("Delete memory {}?", args.id))
            .with_default(false)
            .with_help_message("This action cannot be undone")
            .prompt()?;

        if !confirm {
            print_info("Deletion cancelled.");
            return Ok(());
        }
    }

    match client.delete_memory(&args.id).await {
        Ok(true) => {
            if args.json_output {
                println!(r#"{{"success": true, "deleted": true, "id": "{}"}}"#, args.id);
            } else {
                print_success(&format!("Memory {} deleted.", args.id));
            }
        }
        Ok(false) => {
            if args.json_output {
                println!(r#"{{"success": true, "deleted": false, "id": "{}"}}"#, args.id);
            } else {
                print_info(&format!("Memory {} not found.", args.id));
            }
        }
        Err(e) => {
            print_error(&format!("Failed to delete memory: {}", e));
            return Err(e);
        }
    }

    Ok(())
}
