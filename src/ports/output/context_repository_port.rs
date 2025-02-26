use crate::domain::{Context, ContextChunk, McpResult};
use async_trait::async_trait;
use uuid::Uuid;

/// Output port for context storage operations
#[async_trait]
pub trait ContextRepositoryPort {
    /// Save a new context
    async fn save_context(&self, context: Context) -> McpResult<Context>;

    /// Find a context by its ID
    async fn find_by_id(&self, context_id: Uuid) -> McpResult<Context>;

    /// Update an existing context
    async fn update(&self, context: Context) -> McpResult<Context>;

    /// Delete a context
    async fn delete(&self, context_id: Uuid) -> McpResult<()>;

    /// Find contexts by tags
    async fn find_by_tags(
        &self,
        tags: &[String],
        limit: usize,
        offset: usize,
    ) -> McpResult<Vec<Context>>;

    /// List all contexts with pagination
    async fn list_all(&self, limit: usize, offset: usize) -> McpResult<Vec<Context>>;

    /// Save context chunks
    async fn save_chunks(&self, chunks: Vec<ContextChunk>) -> McpResult<Vec<ContextChunk>>;

    /// Find chunks for a context
    async fn find_chunks_by_context_id(&self, context_id: Uuid) -> McpResult<Vec<ContextChunk>>;

    /// Delete all chunks for a context
    async fn delete_chunks_by_context_id(&self, context_id: Uuid) -> McpResult<()>;
}
