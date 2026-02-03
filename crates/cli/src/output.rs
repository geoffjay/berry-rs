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
#[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_memory() -> Memory {
        Memory {
            id: "mem_123_abc".to_string(),
            content: "Test memory content".to_string(),
            memory_type: MemoryType::Information,
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            created_by: "testuser".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            owner: None,
            visibility: VisibilityLevel::Public,
            shared_with: vec![],
        }
    }

    #[test]
    fn test_format_memory_contains_id() {
        let memory = create_test_memory();
        let output = format_memory(&memory);
        assert!(output.contains("mem_123_abc"));
    }

    #[test]
    fn test_format_memory_contains_type() {
        let memory = create_test_memory();
        let output = format_memory(&memory);
        assert!(output.contains("information"));
    }

    #[test]
    fn test_format_memory_contains_content() {
        let memory = create_test_memory();
        let output = format_memory(&memory);
        assert!(output.contains("Test memory content"));
    }

    #[test]
    fn test_format_memory_contains_tags() {
        let memory = create_test_memory();
        let output = format_memory(&memory);
        assert!(output.contains("tag1"));
        assert!(output.contains("tag2"));
    }

    #[test]
    fn test_format_memory_contains_created_by() {
        let memory = create_test_memory();
        let output = format_memory(&memory);
        assert!(output.contains("testuser"));
    }

    #[test]
    fn test_format_memory_no_tags_section_when_empty() {
        let mut memory = create_test_memory();
        memory.tags = vec![];
        let output = format_memory(&memory);
        // Tags line should not appear when tags are empty
        assert!(!output.contains("Tags:"));
    }

    #[test]
    fn test_format_memory_with_owner() {
        let mut memory = create_test_memory();
        memory.owner = Some("project_lead".to_string());
        let output = format_memory(&memory);
        assert!(output.contains("project_lead"));
    }

    #[test]
    fn test_format_memory_with_shared_with() {
        let mut memory = create_test_memory();
        memory.visibility = VisibilityLevel::Shared;
        memory.shared_with = vec!["alice".to_string(), "bob".to_string()];
        let output = format_memory(&memory);
        assert!(output.contains("alice"));
        assert!(output.contains("bob"));
    }

    #[test]
    fn test_format_memory_table_empty() {
        let memories: Vec<Memory> = vec![];
        let output = format_memory_table(&memories);
        assert_eq!(output, "No memories found.");
    }

    #[test]
    fn test_format_memory_table_single() {
        let memories = vec![create_test_memory()];
        let output = format_memory_table(&memories);
        assert!(output.contains("mem_123_abc"));
        assert!(output.contains("information"));
    }

    #[test]
    fn test_format_memory_table_multiple() {
        let mut mem1 = create_test_memory();
        mem1.id = "mem_1".to_string();
        let mut mem2 = create_test_memory();
        mem2.id = "mem_2".to_string();
        mem2.memory_type = MemoryType::Question;

        let memories = vec![mem1, mem2];
        let output = format_memory_table(&memories);
        assert!(output.contains("mem_1"));
        assert!(output.contains("mem_2"));
        assert!(output.contains("question"));
    }

    #[test]
    fn test_memory_row_truncates_long_content() {
        let mut memory = create_test_memory();
        memory.content = "A".repeat(100); // 100 characters

        let row = MemoryRow::from(&memory);
        assert_eq!(row.content.len(), 50); // "AAA..." (47 + 3)
        assert!(row.content.ends_with("..."));
    }

    #[test]
    fn test_memory_row_preserves_short_content() {
        let mut memory = create_test_memory();
        memory.content = "Short content".to_string();

        let row = MemoryRow::from(&memory);
        assert_eq!(row.content, "Short content");
    }

    #[test]
    fn test_format_memory_type_question() {
        let output = format_memory_type(&MemoryType::Question);
        // Just check it returns something (colored output)
        assert!(!output.is_empty());
    }

    #[test]
    fn test_format_memory_type_request() {
        let output = format_memory_type(&MemoryType::Request);
        assert!(!output.is_empty());
    }

    #[test]
    fn test_format_memory_type_information() {
        let output = format_memory_type(&MemoryType::Information);
        assert!(!output.is_empty());
    }

    #[test]
    fn test_format_visibility_public() {
        let output = format_visibility(&VisibilityLevel::Public);
        assert!(!output.is_empty());
    }

    #[test]
    fn test_format_visibility_shared() {
        let output = format_visibility(&VisibilityLevel::Shared);
        assert!(!output.is_empty());
    }

    #[test]
    fn test_format_visibility_private() {
        let output = format_visibility(&VisibilityLevel::Private);
        assert!(!output.is_empty());
    }
}
