use crate::domain::{Context, ContextMetadata, McpResult};
use async_trait::async_trait;
use uuid::Uuid;

/// Input port for context management operations
#[async_trait]
pub trait ContextManagementPort {
    /// Store a new context
    async fn store_context(&self, content: String, metadata: ContextMetadata)
        -> McpResult<Context>;

    /// Retrieve a context by its ID
    async fn get_context(&self, context_id: Uuid) -> McpResult<Context>;

    /// Update an existing context
    async fn update_context(
        &self,
        context_id: Uuid,
        content: String,
        metadata: ContextMetadata,
    ) -> McpResult<Context>;

    /// Delete a context
    async fn delete_context(&self, context_id: Uuid) -> McpResult<()>;

    /// List all contexts, optionally filtered by tags
    async fn list_contexts(
        &self,
        tags: Option<Vec<String>>,
        limit: usize,
        offset: usize,
    ) -> McpResult<Vec<Context>>;
}
