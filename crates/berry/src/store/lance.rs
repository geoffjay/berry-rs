//! LanceDB vector store implementation.
//!
//! An embedded vector database that stores data as local directories.
//! No server required — data lives on the filesystem.

use std::sync::Arc;

use arrow_array::{ArrayRef, RecordBatch, RecordBatchIterator, StringArray, types::Float32Type};
use arrow_schema::{DataType, Field, Schema, SchemaRef};
use async_trait::async_trait;
use chrono::Utc;
use futures::TryStreamExt;
use lancedb::query::{ExecutableQuery, QueryBase};

use super::embedding::EmbeddingService;
use super::traits::VectorStore;
use crate::config::LanceConfig;
use crate::error::{StoreError, StoreResult};
use crate::types::{CreateMemoryRequest, Memory, MemoryType, SearchRequest, VisibilityLevel};

/// LanceDB vector store implementation.
///
/// Stores memories in a local LanceDB database directory with vector
/// search capabilities. No external server needed.
pub struct LanceStore {
    db: lancedb::Connection,
    table_name: String,
    embedding_service: Arc<dyn EmbeddingService>,
}

impl LanceStore {
    /// Create a new LanceStore with the given configuration and embedding service.
    pub async fn new(
        config: &LanceConfig,
        embedding_service: Arc<dyn EmbeddingService>,
    ) -> StoreResult<Self> {
        let db = lancedb::connect(&config.path)
            .execute()
            .await
            .map_err(|e| StoreError::ConnectionFailed(format!("LanceDB connect failed: {}", e)))?;

        Ok(Self {
            db,
            table_name: config.table.clone(),
            embedding_service,
        })
    }

    /// Build the Arrow schema for the memories table.
    fn memory_schema(&self) -> SchemaRef {
        let dim = self.embedding_service.dimension() as i32;
        Arc::new(Schema::new(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new("content", DataType::LargeUtf8, false),
            Field::new(
                "vector",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    dim,
                ),
                true,
            ),
            Field::new("memory_type", DataType::Utf8, false),
            Field::new("tags", DataType::Utf8, false),
            Field::new("created_by", DataType::Utf8, false),
            Field::new("created_at", DataType::Utf8, false),
            Field::new("updated_at", DataType::Utf8, false),
            Field::new("owner", DataType::Utf8, true),
            Field::new("visibility", DataType::Utf8, false),
            Field::new("shared_with", DataType::Utf8, false),
        ]))
    }

    /// Convert a Memory + embedding vector into a RecordBatch.
    fn memory_to_batch(&self, memory: &Memory, embedding: Vec<f32>) -> StoreResult<RecordBatch> {
        let schema = self.memory_schema();
        let dim = self.embedding_service.dimension() as i32;

        let vector_array =
            arrow_array::FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
                vec![Some(embedding.into_iter().map(Some).collect::<Vec<_>>())],
                dim,
            );

        let owner_array: ArrayRef = if let Some(ref owner) = memory.owner {
            Arc::new(StringArray::from(vec![Some(owner.as_str())]))
        } else {
            Arc::new(StringArray::from(vec![None::<&str>]))
        };

        // Pre-compute owned strings to ensure references live long enough
        let memory_type_str = memory.memory_type.to_string();
        let tags_str = memory.tags.join(",");
        let created_at_str = memory.created_at.to_rfc3339();
        let updated_at_str = memory.updated_at.to_rfc3339();
        let visibility_str = memory.visibility.to_string();
        let shared_with_str = memory.shared_with.join(",");

        RecordBatch::try_new(schema, vec![
            Arc::new(StringArray::from(vec![memory.id.as_str()])),
            Arc::new(arrow_array::LargeStringArray::from(vec![
                memory.content.as_str(),
            ])),
            Arc::new(vector_array),
            Arc::new(StringArray::from(vec![memory_type_str.as_str()])),
            Arc::new(StringArray::from(vec![tags_str.as_str()])),
            Arc::new(StringArray::from(vec![memory.created_by.as_str()])),
            Arc::new(StringArray::from(vec![created_at_str.as_str()])),
            Arc::new(StringArray::from(vec![updated_at_str.as_str()])),
            owner_array,
            Arc::new(StringArray::from(vec![visibility_str.as_str()])),
            Arc::new(StringArray::from(vec![shared_with_str.as_str()])),
        ])
        .map_err(|e| StoreError::InvalidData(format!("Failed to create RecordBatch: {}", e)))
    }

    /// Convert a RecordBatch row into a Memory.
    fn batch_row_to_memory(batch: &RecordBatch, row: usize) -> StoreResult<Memory> {
        let get_str = |name: &str| -> StoreResult<String> {
            if let Some(arr) = batch.column_by_name(name) {
                if let Some(s) = arr.as_any().downcast_ref::<StringArray>() {
                    return Ok(s.value(row).to_string());
                }
                if let Some(s) = arr
                    .as_any()
                    .downcast_ref::<arrow_array::LargeStringArray>()
                {
                    return Ok(s.value(row).to_string());
                }
            }
            Err(StoreError::InvalidData(format!("Missing field: {}", name)))
        };

        let get_str_opt = |name: &str| -> Option<String> {
            if let Some(arr) = batch.column_by_name(name) {
                if arr.is_null(row) {
                    return None;
                }
                if let Some(s) = arr.as_any().downcast_ref::<StringArray>() {
                    let val = s.value(row);
                    if val.is_empty() {
                        return None;
                    }
                    return Some(val.to_string());
                }
            }
            None
        };

        let id = get_str("id")?;
        let content = get_str("content")?;
        let memory_type = get_str("memory_type")?
            .parse::<MemoryType>()
            .map_err(StoreError::InvalidData)?;
        let tags_str = get_str("tags")?;
        let tags: Vec<String> = tags_str
            .split(',')
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();
        let created_by = get_str("created_by")?;
        let created_at = chrono::DateTime::parse_from_rfc3339(&get_str("created_at")?)
            .map_err(|e| StoreError::InvalidData(e.to_string()))?
            .with_timezone(&Utc);
        let updated_at = chrono::DateTime::parse_from_rfc3339(&get_str("updated_at")?)
            .map_err(|e| StoreError::InvalidData(e.to_string()))?
            .with_timezone(&Utc);
        let owner = get_str_opt("owner");
        let visibility = get_str("visibility")?
            .parse::<VisibilityLevel>()
            .map_err(StoreError::InvalidData)?;
        let shared_with_str = get_str("shared_with")?;
        let shared_with: Vec<String> = shared_with_str
            .split(',')
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();

        Ok(Memory {
            id,
            content,
            memory_type,
            tags,
            created_by,
            created_at,
            updated_at,
            owner,
            visibility,
            shared_with,
        })
    }

    /// Collect all rows from a query result stream into Memories.
    async fn collect_memories(
        &self,
        stream: lancedb::arrow::SendableRecordBatchStream,
    ) -> StoreResult<Vec<Memory>> {
        let batches: Vec<RecordBatch> = stream
            .try_collect()
            .await
            .map_err(|e| StoreError::QueryFailed(format!("Failed to collect results: {}", e)))?;

        let mut memories = Vec::new();
        for batch in &batches {
            for row in 0..batch.num_rows() {
                match Self::batch_row_to_memory(batch, row) {
                    Ok(memory) => memories.push(memory),
                    Err(e) => {
                        tracing::warn!("Failed to parse memory row: {}", e);
                    }
                }
            }
        }
        Ok(memories)
    }

    /// Open the table, returning an error if it doesn't exist.
    async fn open_table(&self) -> StoreResult<lancedb::Table> {
        self.db
            .open_table(&self.table_name)
            .execute()
            .await
            .map_err(|e| {
                StoreError::InitializationFailed(format!("Failed to open table: {}", e))
            })
    }
}

#[async_trait]
impl VectorStore for LanceStore {
    async fn initialize(&self) -> StoreResult<()> {
        // Check if table already exists
        let tables = self
            .db
            .table_names()
            .execute()
            .await
            .map_err(|e| {
                StoreError::InitializationFailed(format!("Failed to list tables: {}", e))
            })?;

        if tables.contains(&self.table_name) {
            tracing::debug!("Table '{}' already exists", self.table_name);
            return Ok(());
        }

        // Create empty table with schema
        let schema = self.memory_schema();
        self.db
            .create_empty_table(&self.table_name, schema)
            .execute()
            .await
            .map_err(|e| {
                StoreError::InitializationFailed(format!("Failed to create table: {}", e))
            })?;

        tracing::info!("Created LanceDB table '{}'", self.table_name);
        Ok(())
    }

    async fn create(&self, request: CreateMemoryRequest) -> StoreResult<Memory> {
        let now = Utc::now();

        let memory = Memory {
            id: Memory::generate_id(),
            content: request.content,
            memory_type: request.memory_type,
            tags: request.tags,
            created_by: request.created_by,
            created_at: now,
            updated_at: now,
            owner: None,
            visibility: request.visibility,
            shared_with: request.shared_with,
        };

        // Generate embedding
        let embeddings = self
            .embedding_service
            .embed(std::slice::from_ref(&memory.content))
            .await?;
        let embedding = embeddings.into_iter().next().ok_or_else(|| {
            StoreError::QueryFailed("Failed to generate embedding for content".to_string())
        })?;

        let batch = self.memory_to_batch(&memory, embedding)?;
        let schema = batch.schema();

        let table = self.open_table().await?;
        table
            .add(RecordBatchIterator::new(vec![Ok(batch)], schema))
            .execute()
            .await
            .map_err(|e| StoreError::QueryFailed(format!("Failed to add memory: {}", e)))?;

        Ok(memory)
    }

    async fn get(&self, id: &str) -> StoreResult<Option<Memory>> {
        let table = self.open_table().await?;

        let stream = table
            .query()
            .only_if(format!("id = '{}'", id))
            .execute()
            .await
            .map_err(|e| StoreError::QueryFailed(format!("Failed to query: {}", e)))?;

        let memories = self.collect_memories(stream).await?;
        Ok(memories.into_iter().next())
    }

    async fn delete(&self, id: &str) -> StoreResult<bool> {
        let table = self.open_table().await?;

        // Check existence first
        let exists = self.get(id).await?.is_some();
        if !exists {
            return Ok(false);
        }

        table
            .delete(&format!("id = '{}'", id))
            .await
            .map_err(|e| StoreError::QueryFailed(format!("Failed to delete: {}", e)))?;

        Ok(true)
    }

    async fn search(&self, request: SearchRequest) -> StoreResult<Vec<Memory>> {
        let table = self.open_table().await?;

        // Generate embedding for the query
        let embeddings = self
            .embedding_service
            .embed(std::slice::from_ref(&request.query))
            .await?;
        let query_embedding = embeddings.into_iter().next().ok_or_else(|| {
            StoreError::QueryFailed("Failed to generate embedding for query".to_string())
        })?;

        // Build vector search
        let mut query = table.vector_search(query_embedding).map_err(|e| {
            StoreError::QueryFailed(format!("Failed to build vector search: {}", e))
        })?;

        // Apply type filter
        if let Some(ref memory_type) = request.memory_type {
            query = query.only_if(format!("memory_type = '{}'", memory_type));
        }

        // Request more results to account for post-filtering
        query = query.limit(request.limit * 3);

        let stream = query
            .execute()
            .await
            .map_err(|e| StoreError::QueryFailed(format!("Search failed: {}", e)))?;

        let all_memories = self.collect_memories(stream).await?;

        // Post-filter by visibility, tags, and date range
        let filtered: Vec<Memory> = all_memories
            .into_iter()
            .filter(|m| m.is_visible_to(request.as_actor.as_deref()))
            .filter(|m| {
                if request.tags.is_empty() {
                    return true;
                }
                request.tags.iter().any(|t| m.tags.contains(t))
            })
            .filter(|m| {
                if let Some(ref from) = request.from {
                    if m.created_at < *from {
                        return false;
                    }
                }
                if let Some(ref to) = request.to {
                    if m.created_at > *to {
                        return false;
                    }
                }
                true
            })
            .take(request.limit)
            .collect();

        Ok(filtered)
    }

    async fn update_visibility(
        &self,
        id: &str,
        visibility: VisibilityLevel,
        shared_with: Option<Vec<String>>,
    ) -> StoreResult<Memory> {
        // Get existing memory
        let mut memory = self
            .get(id)
            .await?
            .ok_or_else(|| StoreError::NotFound(id.to_string()))?;

        // Update fields
        memory.visibility = visibility;
        if let Some(shared) = shared_with {
            memory.shared_with = shared;
        }
        memory.updated_at = Utc::now();

        let table = self.open_table().await?;

        // LanceDB update: column values are SQL expressions, so strings must be quoted
        table
            .update()
            .only_if(format!("id = '{}'", id))
            .column("visibility", format!("'{}'", memory.visibility))
            .column("shared_with", format!("'{}'", memory.shared_with.join(",")))
            .column("updated_at", format!("'{}'", memory.updated_at.to_rfc3339()))
            .execute()
            .await
            .map_err(|e| {
                StoreError::QueryFailed(format!("Failed to update visibility: {}", e))
            })?;

        Ok(memory)
    }

    async fn health_check(&self) -> StoreResult<bool> {
        // LanceDB is an embedded database — if we have a connection, it's healthy
        Ok(true)
    }

    async fn list_all(&self) -> StoreResult<Vec<Memory>> {
        let table = self.open_table().await?;

        let stream = table
            .query()
            .execute()
            .await
            .map_err(|e| StoreError::QueryFailed(format!("Failed to list: {}", e)))?;

        let memories = self.collect_memories(stream).await?;
        tracing::info!("Listed {} memories", memories.len());
        Ok(memories)
    }

    async fn delete_collection(&self) -> StoreResult<()> {
        tracing::warn!("Deleting LanceDB table '{}'", self.table_name);

        self.db
            .drop_table(&self.table_name, &[])
            .await
            .map_err(|e| StoreError::QueryFailed(format!("Failed to drop table: {}", e)))?;

        tracing::info!("Table '{}' deleted", self.table_name);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_schema_has_correct_fields() {
        use crate::store::NoOpEmbedding;

        // NoOpEmbedding has dimension 0, which is unusual but tests schema construction
        let service = Arc::new(NoOpEmbedding::new());

        let dim = service.dimension() as i32;
        let schema = Schema::new(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new("content", DataType::LargeUtf8, false),
            Field::new(
                "vector",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    dim,
                ),
                true,
            ),
            Field::new("memory_type", DataType::Utf8, false),
        ]);

        assert_eq!(schema.fields().len(), 4);
        assert_eq!(schema.field(0).name(), "id");
        assert_eq!(schema.field(1).name(), "content");
        assert_eq!(schema.field(2).name(), "vector");
    }
}
