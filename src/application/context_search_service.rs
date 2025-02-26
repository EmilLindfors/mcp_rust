use async_trait::async_trait;
use std::sync::Arc;
use crate::domain::{Context, ContextMatch, ContextReference, ContextSearchResult, McpResult};
use crate::domain::service::RetrievalService;
use crate::ports::in_ports::ContextSearchPort;
use crate::ports::out_ports::{ContextRepositoryPort, EmbeddingPort};

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
    async fn to_search_result(&self, scored_contexts: Vec<(Context, f32)>) -> McpResult<ContextSearchResult> {
        let mut matches = Vec::new();
        
        // For each matching context, get its chunks and create a ContextMatch
        for (context, score) in scored_contexts {
            let chunks = self.context_repository.find_chunks_by_context_id(context.id).await?;
            
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
            if let Ok(chunks) = self.context_repository.find_chunks_by_context_id(context.id).await {
                all_chunks.extend(chunks);
            }
        }
        
        // Use the retrieval service to rank contexts by relevance
        let scored_contexts = self.retrieval_service.rank_contexts(&query, &contexts, &all_chunks);
        
        // Convert the results to the expected format
        self.to_search_result(scored_contexts).await
    }
    
    async fn search_with_tags(&self, query: String, tags: Vec<String>, limit: usize) -> McpResult<ContextSearchResult> {
        // Get contexts with the specified tags
        let tagged_contexts = self.context_repository.find_by_tags(&tags, 1000, 0).await?;
        
        if tagged_contexts.is_empty() {
            return Ok(ContextSearchResult {
                matches: Vec::new(),
                total_matches: 0,
            });
        }
        
        // Use the embedding service to find similar chunks with tags
        let _similar_chunks = self.embedding_service.find_similar_with_tags(&query, &tags, limit).await?;
        
        // Get all chunks for these contexts
        let mut all_chunks = Vec::new();
        for context in &tagged_contexts {
            if let Ok(chunks) = self.context_repository.find_chunks_by_context_id(context.id).await {
                all_chunks.extend(chunks);
            }
        }
        
        // Use the retrieval service to rank contexts by relevance
        let scored_contexts = self.retrieval_service.rank_contexts(&query, &tagged_contexts, &all_chunks);
        
        // Convert the results to the expected format
        self.to_search_result(scored_contexts).await
    }
    
    async fn retrieve_by_references(&self, references: Vec<ContextReference>) -> McpResult<ContextSearchResult> {
        let mut matches = Vec::new();
        
        for reference in references {
            // Get the context
            let context = match self.context_repository.find_by_id(reference.context_id).await {
                Ok(ctx) => ctx,
                Err(_) => continue, // Skip invalid references
            };
            
            // Get the chunks, filtered by chunk_ids if specified
            let chunks = match self.context_repository.find_chunks_by_context_id(context.id).await {
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