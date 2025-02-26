use crate::domain::{ContextChunk, McpResult};
use async_trait::async_trait;

/// Output port for generating and working with embeddings
#[async_trait]
pub trait EmbeddingPort {
    /// Generate embeddings for a batch of context chunks
    async fn embed_chunks(&self, chunks: Vec<ContextChunk>) -> McpResult<Vec<ContextChunk>>;

    /// Find similar chunks based on a query embedding
    async fn find_similar(&self, query: &str, limit: usize) -> McpResult<Vec<(ContextChunk, f32)>>;

    /// Find similar chunks based on a query embedding, filtered by tags
    async fn find_similar_with_tags(
        &self,
        query: &str,
        tags: &[String],
        limit: usize,
    ) -> McpResult<Vec<(ContextChunk, f32)>>;
}
