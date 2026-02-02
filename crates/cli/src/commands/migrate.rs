//! Migrate command implementation.
//!
//! Re-embeds all memories with the current embedding model.

use std::sync::Arc;

use anyhow::Result;

use berry::config::load_config;
use berry::store::{ChromaStore, EmbeddingService, VectorStore, create_embedding_service};
use berry::types::CreateMemoryRequest;

/// Arguments for the migrate command.
#[derive(Debug)]
pub struct MigrateArgs {
    /// Dry run - only show what would be migrated.
    pub dry_run: bool,
    /// New collection name (if different from current).
    pub new_collection: Option<String>,
}

/// Run the migrate command.
pub async fn run(args: MigrateArgs) -> Result<()> {
    let config = load_config().unwrap_or_default();

    // Create embedding service
    let embedding_service: Arc<dyn EmbeddingService> =
        Arc::from(create_embedding_service(&config.embedding)?);

    println!("Using embedding model: {}", config.embedding.model);
    println!("Embedding dimensions: {}", embedding_service.dimension());
    println!();

    // Create store with current config to read existing data
    let source_store = ChromaStore::new(&config.chroma, embedding_service.clone());

    // Initialize and list all memories
    println!(
        "Fetching all memories from collection '{}'...",
        config.chroma.collection
    );
    source_store.initialize().await?;

    let memories = source_store.list_all().await?;
    println!("Found {} memories to migrate", memories.len());
    println!();

    if memories.is_empty() {
        println!("No memories to migrate.");
        return Ok(());
    }

    if args.dry_run {
        println!("Dry run - would migrate the following memories:");
        for memory in &memories {
            println!(
                "  - {} ({}): {}...",
                memory.id,
                memory.memory_type,
                &memory.content[..memory.content.len().min(50)]
            );
        }
        println!();
        println!("To perform the migration, run without --dry-run");
        return Ok(());
    }

    // Determine target collection
    let target_collection = args
        .new_collection
        .unwrap_or_else(|| format!("{}_migrated", config.chroma.collection));

    println!("Target collection: {}", target_collection);
    println!();

    // Create target store with new collection name
    let mut target_config = config.chroma.clone();
    target_config.collection = target_collection.clone();
    let target_store = ChromaStore::new(&target_config, embedding_service);

    // Initialize target collection
    println!("Creating target collection...");
    target_store.initialize().await?;

    // Migrate each memory
    println!("Migrating memories...");
    let mut success_count = 0;
    let mut error_count = 0;

    for memory in memories {
        print!("  Migrating {}... ", memory.id);

        // Create a new memory request (will generate new embedding)
        let request = CreateMemoryRequest {
            content: memory.content,
            memory_type: memory.memory_type,
            tags: memory.tags,
            created_by: memory.created_by,
            references: vec![], // References not preserved in migration
            visibility: memory.visibility,
            shared_with: memory.shared_with,
        };

        match target_store.create(request).await {
            Ok(new_memory) => {
                println!("OK (new id: {})", new_memory.id);
                success_count += 1;
            }
            Err(e) => {
                println!("ERROR: {}", e);
                error_count += 1;
            }
        }
    }

    println!();
    println!("Migration complete:");
    println!("  Successful: {}", success_count);
    println!("  Errors: {}", error_count);
    println!();
    println!(
        "The migrated memories are in collection '{}'.",
        target_collection
    );
    println!();
    println!("To use the new collection, update CHROMA_COLLECTION in your .env file:");
    println!("  CHROMA_COLLECTION={}", target_collection);
    println!();
    println!("Once verified, you can delete the old collection through the ChromaDB dashboard.");

    Ok(())
}
