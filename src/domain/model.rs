use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// The Model Context Protocol core entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    /// Unique identifier for this context
    pub id: Uuid,

    /// Content of the context
    pub content: String,

    /// Metadata associated with this context
    pub metadata: ContextMetadata,

    /// When this context was created
    pub created_at: DateTime<Utc>,

    /// Optional expiry time
    pub expires_at: Option<DateTime<Utc>>,
}

/// Metadata associated with a context
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContextMetadata {
    /// Source of the context (e.g., document, conversation)
    pub source: Option<String>,

    /// Type of the context (e.g., text, code, image)
    pub content_type: Option<String>,

    /// Content hash for deduplication
    pub content_hash: Option<String>,

    /// User-defined tags
    pub tags: Vec<String>,

    /// Additional arbitrary metadata
    pub custom: HashMap<String, String>,
}

/// Represents a chunk of context that can be addressed individually
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextChunk {
    /// ID of the parent context
    pub context_id: Uuid,

    /// Unique identifier for this chunk
    pub chunk_id: Uuid,

    /// Content of this chunk
    pub content: String,

    /// Embedding of this chunk, if available
    pub embedding: Option<Vec<f32>>,

    /// Position of this chunk in the original context
    pub position: usize,
}

/// A reference to a context that can be used in a prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextReference {
    /// ID of the referenced context
    pub context_id: Uuid,

    /// Optional chunk IDs if referencing specific chunks
    pub chunk_ids: Option<Vec<Uuid>>,

    /// Weight to apply to this context in retrieval
    pub weight: Option<f32>,
}

/// Represents a result from a context search operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSearchResult {
    /// The matching contexts or chunks
    pub matches: Vec<ContextMatch>,

    /// Total number of matching contexts in the system
    pub total_matches: usize,
}

/// A single match from a context search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMatch {
    /// The matched context
    pub context: Context,

    /// The specific chunks that matched, if any
    pub chunks: Option<Vec<ContextChunk>>,

    /// Relevance score of this match
    pub score: f32,
}
