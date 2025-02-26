use crate::domain::service::RetrievalService;
use crate::domain::{Context, ContextMatch, ContextReference, ContextSearchResult, McpResult};
use crate::ports::in_ports::ContextSearchPort;
use crate::ports::out_ports::{ContextRepositoryPort, EmbeddingPort};
use async_trait::async_trait;
use std::sync::Arc;

/// Application service implementing the context search use cases
pub struct ContextSearchService {
    context_repository: Arc<dyn ContextRepositoryPort + Send + Sync>,
    embedding_service: Arc<dyn EmbeddingPort + Send + Sync>,
    retrieval_service: RetrievalService,
}

impl ContextSearchService {
    pub fn new(
        context_repository: Arc<dyn ContextRepositoryPort + Send + Sync>,
        embedding_service: Arc<dyn EmbeddingPort + Send + Sync>,
        max_results: usize,
    ) -> Self {
        Self {
            context_repository,
            embedding_service,
            retrieval_service: RetrievalService::new(max_results),
        }
    }

    /// Convert a list of (Context, score) pairs into a ContextSearchResult
    async fn to_search_result(
        &self,
        scored_contexts: Vec<(Context, f32)>,
    ) -> McpResult<ContextSearchResult> {
        let mut matches = Vec::new();

        // For each matching context, get its chunks and create a ContextMatch
        for (context, score) in scored_contexts {
            let chunks = self
                .context_repository
                .find_chunks_by_context_id(context.id)
                .await?;

            matches.push(ContextMatch {
                context,
                chunks: Some(chunks),
                score,
            });
        }

        let total_matches = matches.len();
        Ok(ContextSearchResult {
            matches,
            total_matches,
        })
    }
}

#[async_trait]
impl ContextSearchPort for ContextSearchService {
    async fn search(&self, query: String, limit: usize) -> McpResult<ContextSearchResult> {
        // Use the embedding service to find similar chunks
        let similar_chunks = self.embedding_service.find_similar(&query, limit).await?;

        // Get the contexts for these chunks
        let mut context_ids = std::collections::HashSet::new();
        for (chunk, _) in &similar_chunks {
            context_ids.insert(chunk.context_id);
        }

        // Fetch the full contexts
        let mut contexts = Vec::new();
        for id in context_ids {
            if let Ok(context) = self.context_repository.find_by_id(id).await {
                contexts.push(context);
            }
        }

        // Get all chunks for these contexts
        let mut all_chunks = Vec::new();
        for context in &contexts {
            if let Ok(chunks) = self
                .context_repository
                .find_chunks_by_context_id(context.id)
                .await
            {
                all_chunks.extend(chunks);
            }
        }

        // Use the retrieval service to rank contexts by relevance
        let scored_contexts = self
            .retrieval_service
            .rank_contexts(&query, &contexts, &all_chunks);

        // Convert the results to the expected format
        self.to_search_result(scored_contexts).await
    }

    async fn search_with_tags(
        &self,
        query: String,
        tags: Vec<String>,
        limit: usize,
    ) -> McpResult<ContextSearchResult> {
        // Get contexts with the specified tags
        let tagged_contexts = self.context_repository.find_by_tags(&tags, 1000, 0).await?;

        if tagged_contexts.is_empty() {
            return Ok(ContextSearchResult {
                matches: Vec::new(),
                total_matches: 0,
            });
        }

        // Use the embedding service to find similar chunks with tags
        let _similar_chunks = self
            .embedding_service
            .find_similar_with_tags(&query, &tags, limit)
            .await?;

        // Get all chunks for these contexts
        let mut all_chunks = Vec::new();
        for context in &tagged_contexts {
            if let Ok(chunks) = self
                .context_repository
                .find_chunks_by_context_id(context.id)
                .await
            {
                all_chunks.extend(chunks);
            }
        }

        // Use the retrieval service to rank contexts by relevance
        let scored_contexts =
            self.retrieval_service
                .rank_contexts(&query, &tagged_contexts, &all_chunks);

        // Convert the results to the expected format
        self.to_search_result(scored_contexts).await
    }

    async fn retrieve_by_references(
        &self,
        references: Vec<ContextReference>,
    ) -> McpResult<ContextSearchResult> {
        let mut matches = Vec::new();

        for reference in references {
            // Get the context
            let context = match self
                .context_repository
                .find_by_id(reference.context_id)
                .await
            {
                Ok(ctx) => ctx,
                Err(_) => continue, // Skip invalid references
            };

            // Get the chunks, filtered by chunk_ids if specified
            let chunks = match self
                .context_repository
                .find_chunks_by_context_id(context.id)
                .await
            {
                Ok(all_chunks) => {
                    if let Some(chunk_ids) = &reference.chunk_ids {
                        all_chunks
                            .into_iter()
                            .filter(|chunk| chunk_ids.contains(&chunk.chunk_id))
                            .collect()
                    } else {
                        all_chunks
                    }
                }
                Err(_) => continue, // Skip if chunks can't be retrieved
            };

            // Use the reference weight or default to 1.0
            let score = reference.weight.unwrap_or(1.0);

            matches.push(ContextMatch {
                context,
                chunks: Some(chunks),
                score,
            });
        }

        let total_matches = matches.len();
        Ok(ContextSearchResult {
            matches,
            total_matches,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ContextChunk;
    use crate::domain::ContextMetadata;
    use mockall::mock;
    use mockall::predicate::*;
    use uuid::Uuid;

    mock! {
        ContextRepository {}
        #[async_trait]
        impl ContextRepositoryPort for ContextRepository {
            async fn find_by_id(&self, id: Uuid) -> McpResult<Context>;
            async fn find_chunks_by_context_id(&self, context_id: Uuid) -> McpResult<Vec<ContextChunk>>;
            async fn find_by_tags(&self, tags: &[String], limit: usize, offset: usize) -> McpResult<Vec<Context>>;
            async fn save_context(&self, context: Context) -> McpResult<Context>;
            async fn update(&self, context: Context) -> McpResult<Context>;
            async fn delete(&self, context_id: Uuid) -> McpResult<()>;
            async fn list_all(&self, limit: usize, offset: usize) -> McpResult<Vec<Context>>;
            async fn save_chunks(&self, chunks: Vec<ContextChunk>) -> McpResult<Vec<ContextChunk>>;
            async fn delete_chunks_by_context_id(&self, context_id: Uuid) -> McpResult<()>;
        }
    }

    mock! {
        EmbeddingService {}
        #[async_trait]
        impl EmbeddingPort for EmbeddingService {
            async fn find_similar(&self, query: &str, limit: usize) -> McpResult<Vec<(ContextChunk, f32)>>;
            async fn find_similar_with_tags(&self, query: &str, tags: &[String], limit: usize) -> McpResult<Vec<(ContextChunk, f32)>>;
            async fn embed_chunks(&self, chunks: Vec<ContextChunk>) -> McpResult<Vec<ContextChunk>>;
        }
    }

    fn create_test_context(id: Uuid) -> Context {
        Context {
            id,
            content: format!("Context content {}", id),
            metadata: ContextMetadata::default(),
            created_at: chrono::Utc::now(),
            expires_at: None,
        }
    }

    fn create_test_chunk(context_id: Uuid, chunk_id: Uuid) -> ContextChunk {
        ContextChunk {
            context_id,
            chunk_id,
            content: format!("Chunk content {}", chunk_id),
            embedding: Some(vec![0.1, 0.2, 0.3]),
            position: 0,
        }
    }

    #[tokio::test]
    async fn test_search_success() {
        let mut repo_mock = MockContextRepository::new();
        let mut embedding_mock = MockEmbeddingService::new();

        // Create contexts with fixed IDs
        let context1_id = Uuid::new_v4();
        let context2_id = Uuid::new_v4();
        let context1 = create_test_context(context1_id);
        let context2 = create_test_context(context2_id);

        // Create chunks that reference these contexts
        let chunk1 = create_test_chunk(context1_id, Uuid::new_v4());
        let chunk2 = create_test_chunk(context1_id, Uuid::new_v4());
        let chunk3 = create_test_chunk(context2_id, Uuid::new_v4());

        // Set up expectations for embedding service with exact context IDs
        embedding_mock
            .expect_find_similar()
            .with(eq("test query"), eq(10))
            .times(1)
            .returning(move |_, _| {
                Ok(vec![
                    (chunk1.clone(), 0.9),
                    (chunk2.clone(), 0.8),
                    (chunk3.clone(), 0.7),
                ])
            });

        // Set up expectations for context repository with exact IDs
        repo_mock
            .expect_find_by_id()
            .with(eq(context1_id))
            .times(1)
            .returning(move |_| Ok(context1.clone()));

        repo_mock
            .expect_find_by_id()
            .with(eq(context2_id))
            .times(1)
            .returning(move |_| Ok(context2.clone()));

        // Set up expectations for finding chunks by context ID
        repo_mock
            .expect_find_chunks_by_context_id()
            .with(eq(context1_id))
            .times(2) // Once for context fetching, once for result conversion
            .returning(move |_| {
                Ok(vec![
                    create_test_chunk(context1_id, Uuid::new_v4()),
                    create_test_chunk(context1_id, Uuid::new_v4()),
                ])
            });

        repo_mock
            .expect_find_chunks_by_context_id()
            .with(eq(context2_id))
            .times(2) // Once for context fetching, once for result conversion
            .returning(move |_| Ok(vec![create_test_chunk(context2_id, Uuid::new_v4())]));

        let service = ContextSearchService::new(Arc::new(repo_mock), Arc::new(embedding_mock), 5);

        // Execute the method under test
        let result = service.search("test query".to_string(), 10).await;

        // Verify results
        assert!(result.is_ok());
        let search_result = result.unwrap();
        assert_eq!(search_result.total_matches, 2); // Two contexts matched
        assert_eq!(search_result.matches.len(), 2);
    }

    #[tokio::test]
    async fn test_search_with_tags_success() {
        let mut repo_mock = MockContextRepository::new();
        let mut embedding_mock = MockEmbeddingService::new();

        let chunk_id = Uuid::new_v4();

        let tags = vec!["tag1".to_string()];
        let context1 = create_test_context(chunk_id);

        // Set up expectations for repository
        repo_mock
            .expect_find_by_tags()
            .with(eq(tags.clone()), eq(1000), eq(0))
            .times(1)
            .returning(move |_, _, _| Ok(vec![context1.clone()]));

        repo_mock
            .expect_find_chunks_by_context_id()
            .with(eq(chunk_id))
            .times(2) // Once for fetching chunks, once for result conversion
            .returning(move |_| Ok(vec![create_test_chunk(chunk_id, Uuid::new_v4())]));

        // Set up expectations for embedding service
        embedding_mock
            .expect_find_similar_with_tags()
            .with(eq("test query"), eq(tags.clone()), eq(5))
            .times(1)
            .returning(move |_, _, _| Ok(vec![(create_test_chunk(chunk_id, Uuid::new_v4()), 0.9)]));

        let service = ContextSearchService::new(Arc::new(repo_mock), Arc::new(embedding_mock), 5);

        // Execute the method under test
        let result = service
            .search_with_tags("test query".to_string(), tags, 5)
            .await;

        // Verify results
        assert!(result.is_ok());
        let search_result = result.unwrap();
        assert_eq!(search_result.total_matches, 1);
        assert_eq!(search_result.matches.len(), 1);
    }

    #[tokio::test]
    async fn test_search_with_tags_empty_result() {
        let mut repo_mock = MockContextRepository::new();
        let embedding_mock = MockEmbeddingService::new();

        let tags = vec!["nonexistent_tag".to_string()];

        // Set up expectations for repository to return empty results
        repo_mock
            .expect_find_by_tags()
            .with(eq(tags.clone()), eq(1000), eq(0))
            .times(1)
            .returning(|_, _, _| Ok(Vec::new()));

        let service = ContextSearchService::new(Arc::new(repo_mock), Arc::new(embedding_mock), 5);

        // Execute the method under test
        let result = service
            .search_with_tags("test query".to_string(), tags, 5)
            .await;

        // Verify results
        assert!(result.is_ok());
        let search_result = result.unwrap();
        assert_eq!(search_result.total_matches, 0);
        assert_eq!(search_result.matches.len(), 0);
    }

    #[tokio::test]
    async fn test_to_search_result() {
        let repo_mock = MockContextRepository::new();
        let embedding_mock = MockEmbeddingService::new();

        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        let context1 = create_test_context(id1);
        let context2 = create_test_context(id2);

        let mut repo_mock = MockContextRepository::new();
        repo_mock
            .expect_find_chunks_by_context_id()
            .with(eq(id1))
            .times(1)
            .returning(move |_| Ok(vec![create_test_chunk(id1, Uuid::new_v4())]));

        repo_mock
            .expect_find_chunks_by_context_id()
            .with(eq(id2))
            .times(1)
            .returning(move |_| Ok(vec![create_test_chunk(id2, Uuid::new_v4())]));

        let service = ContextSearchService::new(Arc::new(repo_mock), Arc::new(embedding_mock), 5);

        // Prepare scored contexts
        let scored_contexts = vec![(context1, 0.9), (context2, 0.8)];

        // Execute the method under test
        let result = service.to_search_result(scored_contexts).await;

        // Verify results
        assert!(result.is_ok());
        let search_result = result.unwrap();
        assert_eq!(search_result.total_matches, 2);
        assert_eq!(search_result.matches.len(), 2);
        assert_eq!(search_result.matches[0].score, 0.9);
        assert_eq!(search_result.matches[1].score, 0.8);
    }
}
