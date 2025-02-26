use crate::domain::{ContextReference, ContextSearchResult, McpResult};
use async_trait::async_trait;

/// Input port for context searching operations
#[async_trait]
pub trait ContextSearchPort {
    /// Search for relevant contexts based on a query string
    async fn search(&self, query: String, limit: usize) -> McpResult<ContextSearchResult>;

    /// Search for relevant contexts based on a query string, filtered by tags
    async fn search_with_tags(
        &self,
        query: String,
        tags: Vec<String>,
        limit: usize,
    ) -> McpResult<ContextSearchResult>;

    /// Retrieve relevant contexts based on provided reference IDs
    async fn retrieve_by_references(
        &self,
        references: Vec<ContextReference>,
    ) -> McpResult<ContextSearchResult>;
}
