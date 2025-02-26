use async_trait::async_trait;
use uuid::Uuid;
use chrono::Utc;
use std::sync::Arc;

use crate::domain::{Context, ContextMetadata, McpResult};
use crate::domain::service::ChunkingService;
use crate::ports::in_ports::ContextManagementPort;
use crate::ports::out_ports::{ContextRepositoryPort, EmbeddingPort};

/// Application service implementing the context management use cases
pub struct ContextManagementService {
    context_repository: Arc<dyn ContextRepositoryPort + Send + Sync>,
    embedding_service: Arc<dyn EmbeddingPort + Send + Sync>,
    chunking_service: ChunkingService,
}

impl ContextManagementService {
    pub fn new(
        context_repository: Arc<dyn ContextRepositoryPort + Send + Sync>,
        embedding_service: Arc<dyn EmbeddingPort + Send + Sync>,
        max_chunk_size: usize,
        chunk_overlap: usize,
    ) -> Self {
        Self {
            context_repository,
            embedding_service,
            chunking_service: ChunkingService::new(max_chunk_size, chunk_overlap),
        }
    }
    
    /// Process a context by chunking it and generating embeddings
    async fn process_context(&self, context: Context) -> McpResult<Context> {
        // Split context into chunks
        let chunks = self.chunking_service.chunk_context(&context);
        
        // Generate embeddings for chunks
        let chunks_with_embeddings = self.embedding_service.embed_chunks(chunks).await?;
        
        // Store chunks
        self.context_repository.save_chunks(chunks_with_embeddings).await?;
        
        Ok(context)
    }
}

#[async_trait]
impl ContextManagementPort for ContextManagementService {
    async fn store_context(&self, content: String, metadata: ContextMetadata) -> McpResult<Context> {
        // Create a new context entity
        let context = Context {
            id: Uuid::new_v4(),
            content,
            metadata,
            created_at: Utc::now(),
            expires_at: None,
        };
        
        // Save the context
        let saved_context = self.context_repository.save_context(context).await?;
        
        // Process the context (chunk and embed)
        self.process_context(saved_context).await
    }
    
    async fn get_context(&self, context_id: Uuid) -> McpResult<Context> {
        self.context_repository.find_by_id(context_id).await
    }
    
    async fn update_context(&self, context_id: Uuid, content: String, metadata: ContextMetadata) -> McpResult<Context> {
        // Find the existing context
        let mut context = self.context_repository.find_by_id(context_id).await?;
        
        // Update its fields
        context.content = content;
        context.metadata = metadata;
        
        // Delete old chunks
        self.context_repository.delete_chunks_by_context_id(context_id).await?;
        
        // Save the updated context
        let updated_context = self.context_repository.update(context).await?;
        
        // Re-process the context
        self.process_context(updated_context).await
    }
    
    async fn delete_context(&self, context_id: Uuid) -> McpResult<()> {
        // Delete chunks first
        self.context_repository.delete_chunks_by_context_id(context_id).await?;
        
        // Then delete the context
        self.context_repository.delete(context_id).await
    }
    
    async fn list_contexts(&self, tags: Option<Vec<String>>, limit: usize, offset: usize) -> McpResult<Vec<Context>> {
        match tags {
            Some(tags) if !tags.is_empty() => {
                self.context_repository.find_by_tags(&tags, limit, offset).await
            }
            _ => self.context_repository.list_all(limit, offset).await,
        }
    }
}