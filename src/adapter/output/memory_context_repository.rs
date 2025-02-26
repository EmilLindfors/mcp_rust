use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

use crate::domain::{Context, ContextChunk, McpError, McpResult};
use crate::ports::out_ports::ContextRepositoryPort;

/// In-memory implementation of the context repository
/// Used for testing and as a simple reference implementation
pub struct InMemoryContextRepository {
    contexts: Mutex<HashMap<Uuid, Context>>,
    chunks: Mutex<HashMap<Uuid, Vec<ContextChunk>>>,
}

impl InMemoryContextRepository {
    pub fn new() -> Self {
        Self {
            contexts: Mutex::new(HashMap::new()),
            chunks: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl ContextRepositoryPort for InMemoryContextRepository {
    async fn save_context(&self, context: Context) -> McpResult<Context> {
        let mut contexts = self.contexts.lock().unwrap();
        let context_id = context.id;

        if contexts.contains_key(&context_id) {
            return Err(McpError::ContextAlreadyExists(context_id));
        }

        contexts.insert(context_id, context.clone());
        Ok(context)
    }

    async fn find_by_id(&self, context_id: Uuid) -> McpResult<Context> {
        let contexts = self.contexts.lock().unwrap();

        contexts
            .get(&context_id)
            .cloned()
            .ok_or_else(|| McpError::ContextNotFound(context_id))
    }

    async fn update(&self, context: Context) -> McpResult<Context> {
        let mut contexts = self.contexts.lock().unwrap();
        let context_id = context.id;

        if !contexts.contains_key(&context_id) {
            return Err(McpError::ContextNotFound(context_id));
        }

        contexts.insert(context_id, context.clone());
        Ok(context)
    }

    async fn delete(&self, context_id: Uuid) -> McpResult<()> {
        let mut contexts = self.contexts.lock().unwrap();

        if !contexts.contains_key(&context_id) {
            return Err(McpError::ContextNotFound(context_id));
        }

        contexts.remove(&context_id);
        Ok(())
    }

    async fn find_by_tags(
        &self,
        tags: &[String],
        limit: usize,
        offset: usize,
    ) -> McpResult<Vec<Context>> {
        let contexts = self.contexts.lock().unwrap();

        let matching_contexts: Vec<Context> = contexts
            .values()
            .filter(|context| tags.iter().all(|tag| context.metadata.tags.contains(tag)))
            .cloned()
            .skip(offset)
            .take(limit)
            .collect();

        Ok(matching_contexts)
    }

    async fn list_all(&self, limit: usize, offset: usize) -> McpResult<Vec<Context>> {
        let contexts = self.contexts.lock().unwrap();

        let all_contexts: Vec<Context> = contexts
            .values()
            .cloned()
            .skip(offset)
            .take(limit)
            .collect();

        Ok(all_contexts)
    }

    async fn save_chunks(&self, chunks: Vec<ContextChunk>) -> McpResult<Vec<ContextChunk>> {
        if chunks.is_empty() {
            return Ok(vec![]);
        }

        let context_id = chunks[0].context_id;
        let mut chunks_map = self.chunks.lock().unwrap();

        // Store chunks by context ID
        chunks_map.insert(context_id, chunks.clone());

        Ok(chunks)
    }

    async fn find_chunks_by_context_id(&self, context_id: Uuid) -> McpResult<Vec<ContextChunk>> {
        let chunks_map = self.chunks.lock().unwrap();

        chunks_map
            .get(&context_id)
            .cloned()
            .ok_or_else(|| McpError::ContextNotFound(context_id))
    }

    async fn delete_chunks_by_context_id(&self, context_id: Uuid) -> McpResult<()> {
        let mut chunks_map = self.chunks.lock().unwrap();
        chunks_map.remove(&context_id);
        Ok(())
    }
}
