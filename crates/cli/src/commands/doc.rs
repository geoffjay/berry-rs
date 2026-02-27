//! Document commands - manage markdown documents.

use anyhow::Result;
use colored::Colorize;
use inquire::{Confirm, Text};

use berry::{CreateDocumentRequest, Document, ListDocumentsRequest, UpdateDocumentRequest};

use crate::client::BerryClient;
use crate::output::{print_error, print_success};

/// Document subcommand variants.
#[derive(Debug)]
pub enum DocAction {
    Create {
        title: String,
        content: Option<String>,
        tags: Vec<String>,
        created_by: String,
    },
    Read {
        id: String,
    },
    Update {
        id: String,
        title: Option<String>,
        content: Option<String>,
        tags: Option<Vec<String>>,
    },
    Delete {
        id: String,
        force: bool,
    },
    List {
        tags: Option<Vec<String>>,
        created_by: Option<String>,
    },
}

/// Document command arguments.
#[derive(Debug)]
pub struct DocArgs {
    pub action: DocAction,
    pub json_output: bool,
}

/// Format a document for display.
fn format_document(doc: &Document) -> String {
    let mut output = String::new();
    output.push_str(&format!("{}: {}\n", "ID".cyan().bold(), doc.id));
    output.push_str(&format!("{}: {}\n", "Title".cyan().bold(), doc.title));
    output.push_str(&format!(
        "{}: {}\n",
        "Created By".cyan().bold(),
        doc.created_by
    ));
    output.push_str(&format!(
        "{}: {}\n",
        "Created At".cyan().bold(),
        doc.created_at.format("%Y-%m-%d %H:%M:%S UTC")
    ));
    output.push_str(&format!(
        "{}: {}\n",
        "Updated At".cyan().bold(),
        doc.updated_at.format("%Y-%m-%d %H:%M:%S UTC")
    ));
    if !doc.tags.is_empty() {
        output.push_str(&format!(
            "{}: {}\n",
            "Tags".cyan().bold(),
            doc.tags.join(", ")
        ));
    }
    output.push_str(&format!("\n{}\n", doc.content));
    output
}

/// Run the doc command.
pub async fn run(client: BerryClient, args: DocArgs) -> Result<()> {
    match args.action {
        DocAction::Create {
            title,
            content,
            tags,
            created_by,
        } => {
            let content = match content {
                Some(c) => c,
                None => Text::new("Document content (markdown)")
                    .with_help_message("Enter the document content")
                    .prompt()?,
            };

            let request = CreateDocumentRequest {
                title,
                content,
                tags,
                created_by,
            };

            match client.create_document(request).await {
                Ok(doc) => {
                    if args.json_output {
                        println!("{}", serde_json::to_string_pretty(&doc)?);
                    } else {
                        print_success(&format!("Document created: {}", doc.id));
                        println!("\n{}", format_document(&doc));
                    }
                }
                Err(e) => {
                    print_error(&format!("Failed to create document: {}", e));
                    return Err(e);
                }
            }
        }

        DocAction::Read { id } => match client.get_document(&id).await {
            Ok(Some(doc)) => {
                if args.json_output {
                    println!("{}", serde_json::to_string_pretty(&doc)?);
                } else {
                    println!("{}", format_document(&doc));
                }
            }
            Ok(None) => {
                print_error(&format!("Document not found: {}", id));
                anyhow::bail!("Document not found: {}", id);
            }
            Err(e) => {
                print_error(&format!("Failed to get document: {}", e));
                return Err(e);
            }
        },

        DocAction::Update {
            id,
            title,
            content,
            tags,
        } => {
            let request = UpdateDocumentRequest {
                title,
                content,
                tags,
            };

            match client.update_document(&id, request).await {
                Ok(doc) => {
                    if args.json_output {
                        println!("{}", serde_json::to_string_pretty(&doc)?);
                    } else {
                        print_success(&format!("Document updated: {}", doc.id));
                        println!("\n{}", format_document(&doc));
                    }
                }
                Err(e) => {
                    print_error(&format!("Failed to update document: {}", e));
                    return Err(e);
                }
            }
        }

        DocAction::Delete { id, force } => {
            if !force {
                let confirmed = Confirm::new(&format!("Delete document '{}'?", id))
                    .with_default(false)
                    .prompt()?;
                if !confirmed {
                    println!("Cancelled.");
                    return Ok(());
                }
            }

            match client.delete_document(&id).await {
                Ok(true) => {
                    if args.json_output {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "success": true,
                                "deleted": true
                            }))?
                        );
                    } else {
                        print_success(&format!("Document deleted: {}", id));
                    }
                }
                Ok(false) => {
                    print_error(&format!("Document not found: {}", id));
                }
                Err(e) => {
                    print_error(&format!("Failed to delete document: {}", e));
                    return Err(e);
                }
            }
        }

        DocAction::List { tags, created_by } => {
            let request = ListDocumentsRequest { tags, created_by };

            match client.list_documents(request).await {
                Ok(documents) => {
                    if args.json_output {
                        println!("{}", serde_json::to_string_pretty(&documents)?);
                    } else if documents.is_empty() {
                        println!("No documents found.");
                    } else {
                        for doc in &documents {
                            println!(
                                "{} {} {}",
                                doc.id.cyan(),
                                doc.title.bold(),
                                if doc.tags.is_empty() {
                                    String::new()
                                } else {
                                    format!("[{}]", doc.tags.join(", "))
                                }
                            );
                        }
                        println!("\n{} document(s)", documents.len());
                    }
                }
                Err(e) => {
                    print_error(&format!("Failed to list documents: {}", e));
                    return Err(e);
                }
            }
        }
    }

    Ok(())
}
