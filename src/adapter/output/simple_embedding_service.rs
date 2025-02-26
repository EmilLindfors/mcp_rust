use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

use crate::domain::{ContextChunk, McpResult};
use crate::ports::out_ports::EmbeddingPort;

/// A simple embedding implementation that computes token-based embeddings
/// Used for demonstration and testing purposes
pub struct SimpleEmbeddingService {
    chunk_embeddings: Mutex<HashMap<Uuid, Vec<f32>>>,
    embedding_dimension: usize,
}

impl SimpleEmbeddingService {
    pub fn new(embedding_dimension: usize) -> Self {
        Self {
            chunk_embeddings: Mutex::new(HashMap::new()),
            embedding_dimension,
        }
    }

    /// Create a simple embedding for text by counting word frequencies
    /// This is not a real embedding model, just a toy implementation for demonstration
    fn compute_embedding(&self, text: &str) -> Vec<f32> {
        // Count word frequencies to create a simple embedding
        let mut word_counts: HashMap<String, usize> = HashMap::new();

        // Tokenize by splitting on whitespace and removing punctuation
        for word in text.split_whitespace() {
            let word = word
                .to_lowercase()
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>();

            if !word.is_empty() {
                *word_counts.entry(word).or_insert(0) += 1;
            }
        }

        // Create a simple embedding by hashing words to dimensions
        let mut embedding = vec![0.0; self.embedding_dimension];

        for (word, count) in word_counts {
            // Simple hash function to map words to dimensions
            let dimension = word
                .bytes()
                .fold(0_usize, |acc, b| acc.wrapping_add(b as usize))
                % self.embedding_dimension;
            embedding[dimension] += count as f32;
        }

        // Normalize the embedding
        let magnitude: f32 = embedding.iter().map(|&x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for value in &mut embedding {
                *value /= magnitude;
            }
        }

        embedding
    }

    /// Calculate cosine similarity between two embeddings
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if magnitude_a > 0.0 && magnitude_b > 0.0 {
            dot_product / (magnitude_a * magnitude_b)
        } else {
            0.0
        }
    }
}

#[async_trait]
impl EmbeddingPort for SimpleEmbeddingService {
    async fn embed_chunks(&self, chunks: Vec<ContextChunk>) -> McpResult<Vec<ContextChunk>> {
        let mut result_chunks = Vec::new();
        let mut embeddings = self.chunk_embeddings.lock().unwrap();

        for mut chunk in chunks {
            // Generate embedding for this chunk
            let embedding = self.compute_embedding(&chunk.content);

            // Store embedding in repository
            embeddings.insert(chunk.chunk_id, embedding.clone());

            // Add embedding to chunk and collect
            chunk.embedding = Some(embedding);
            result_chunks.push(chunk);
        }

        Ok(result_chunks)
    }

    async fn find_similar(&self, query: &str, limit: usize) -> McpResult<Vec<(ContextChunk, f32)>> {
        // Generate embedding for the query
        let query_embedding = self.compute_embedding(query);

        // Get all stored embeddings
        let embeddings = self.chunk_embeddings.lock().unwrap();

        // This would be inefficient in a real system, but works for demonstration
        let mut chunk_scores = Vec::new();

        // This is a placeholder - in a real system, we would retrieve the actual chunks
        // based on their IDs, but this is just a simulation
        for (chunk_id, embedding) in &*embeddings {
            // Calculate similarity score
            let score = Self::cosine_similarity(&query_embedding, embedding);

            // Create a real chunk with the actual content
            // This would be retrieved from the repository in a real implementation
            let chunk = ContextChunk {
                chunk_id: *chunk_id,
                context_id: Uuid::new_v4(), // For testing, use a new random ID
                content: "This is content for the embedding search test".to_string(),
                embedding: Some(embedding.clone()),
                position: 0,
            };

            chunk_scores.push((chunk, score));
        }

        // Sort by similarity score descending
        chunk_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top results
        chunk_scores.truncate(limit);

        Ok(chunk_scores)
    }

    async fn find_similar_with_tags(
        &self,
        query: &str,
        _tags: &[String],
        limit: usize,
    ) -> McpResult<Vec<(ContextChunk, f32)>> {
        // In a real implementation, this would filter by tags
        // For now, just delegate to the standard search
        self.find_similar(query, limit).await
    }
}
