//! Output formatting for CLI display.

use colored::Colorize;
use tabled::{Table, Tabled};

use berry::{Memory, MemoryType, VisibilityLevel};

/// Format a single memory for display.
pub fn format_memory(memory: &Memory) -> String {
    let mut output = String::new();

    output.push_str(&format!("{}: {}\n", "ID".cyan().bold(), memory.id));
    output.push_str(&format!(
        "{}: {}\n",
        "Type".cyan().bold(),
        memory.memory_type
    ));
    output.push_str(&format!(
        "{}: {}\n",
        "Visibility".cyan().bold(),
        format_visibility(&memory.visibility)
    ));
    output.push_str(&format!(
        "{}: {}\n",
        "Created By".cyan().bold(),
        memory.created_by
    ));
    output.push_str(&format!(
        "{}: {}\n",
        "Created At".cyan().bold(),
        memory.created_at.format("%Y-%m-%d %H:%M:%S UTC")
    ));

    if !memory.tags.is_empty() {
        output.push_str(&format!(
            "{}: {}\n",
            "Tags".cyan().bold(),
            memory.tags.join(", ")
        ));
    }

    if let Some(ref owner) = memory.owner {
        output.push_str(&format!("{}: {}\n", "Owner".cyan().bold(), owner));
    }

    if !memory.shared_with.is_empty() {
        output.push_str(&format!(
            "{}: {}\n",
            "Shared With".cyan().bold(),
            memory.shared_with.join(", ")
        ));
    }

    output.push_str(&format!("\n{}\n", memory.content));

    output
}

/// Format visibility level with color.
fn format_visibility(visibility: &VisibilityLevel) -> String {
    match visibility {
        VisibilityLevel::Public => "public".green().to_string(),
        VisibilityLevel::Shared => "shared".yellow().to_string(),
        VisibilityLevel::Private => "private".red().to_string(),
    }
}

/// Row for table display of memories.
#[derive(Tabled)]
struct MemoryRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Type")]
    memory_type: String,
    #[tabled(rename = "Content")]
    content: String,
    #[tabled(rename = "Tags")]
    tags: String,
    #[tabled(rename = "Created")]
    created_at: String,
}

impl From<&Memory> for MemoryRow {
    fn from(memory: &Memory) -> Self {
        let content = if memory.content.len() > 50 {
            format!("{}...", &memory.content[..47])
        } else {
            memory.content.clone()
        };

        Self {
            id: memory.id.clone(),
            memory_type: memory.memory_type.to_string(),
            content,
            tags: memory.tags.join(", "),
            created_at: memory.created_at.format("%Y-%m-%d").to_string(),
        }
    }
}

/// Format multiple memories as a table.
pub fn format_memory_table(memories: &[Memory]) -> String {
    if memories.is_empty() {
        return "No memories found.".to_string();
    }

    let rows: Vec<MemoryRow> = memories.iter().map(MemoryRow::from).collect();
    Table::new(rows).to_string()
}

/// Format memory type for display.
pub fn format_memory_type(memory_type: &MemoryType) -> String {
    match memory_type {
        MemoryType::Question => "question".blue().to_string(),
        MemoryType::Request => "request".magenta().to_string(),
        MemoryType::Information => "information".white().to_string(),
    }
}

/// Print a success message.
pub fn print_success(message: &str) {
    println!("{} {}", "✓".green().bold(), message);
}

/// Print an error message.
pub fn print_error(message: &str) {
    eprintln!("{} {}", "✗".red().bold(), message);
}

/// Print an info message.
pub fn print_info(message: &str) {
    println!("{} {}", "ℹ".blue().bold(), message);
}
