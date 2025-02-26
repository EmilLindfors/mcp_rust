use crate::domain::model::{Context, ContextChunk};
use uuid::Uuid;

/// Core domain service for chunking content into manageable pieces
pub struct ChunkingService {
    max_chunk_size: usize,
    overlap: usize,
}

impl ChunkingService {
    pub fn new(max_chunk_size: usize, overlap: usize) -> Self {
        Self {
            max_chunk_size,
            overlap,
        }
    }
    
    /// Split a context into chunks with optional overlap
    pub fn chunk_context(&self, context: &Context) -> Vec<ContextChunk> {
        let content = &context.content;
        
        // Simple chunking strategy - split by max_chunk_size with overlap
        let mut chunks = Vec::new();
        let mut position = 0;
        
        while position < content.len() {
            let end = std::cmp::min(position + self.max_chunk_size, content.len());
            let chunk_content = content[position..end].to_string();
            
            chunks.push(ContextChunk {
                context_id: context.id,
                chunk_id: Uuid::new_v4(),
                content: chunk_content,
                embedding: None,
                position,
            });
            
            // Move position forward, accounting for overlap
            if end == content.len() {
                break;
            }
            position = position + self.max_chunk_size - self.overlap;
        }
        
        chunks
    }
}

/// Core domain service for ranking and retrieving contexts
pub struct RetrievalService {
    max_results: usize,
}

impl RetrievalService {
    pub fn new(max_results: usize) -> Self {
        Self { max_results }
    }
    
    /// Rank contexts by relevance and return the top matching results
    pub fn rank_contexts(
        &self,
        query: &str,
        available_contexts: &[Context],
        _context_chunks: &[ContextChunk],
    ) -> Vec<(Context, f32)> {
        // In a real implementation, this would use semantic search or other 
        // sophisticated ranking algorithms. For this example, we'll use a simple
        // implementation based on text matching.
        
        let mut scored_contexts: Vec<(Context, f32)> = available_contexts
            .iter()
            .map(|ctx| {
                // Simple scoring: ratio of query terms found in context
                let query_terms: Vec<&str> = query.split_whitespace().collect();
                let mut matches = 0;
                
                for term in &query_terms {
                    if ctx.content.to_lowercase().contains(&term.to_lowercase()) {
                        matches += 1;
                    }
                }
                
                let score = if query_terms.is_empty() {
                    0.0
                } else {
                    matches as f32 / query_terms.len() as f32
                };
                
                (ctx.clone(), score)
            })
            .collect();
        
        // Sort by score descending
        scored_contexts.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Return top results
        scored_contexts.truncate(self.max_results);
        scored_contexts
    }
}