use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Request to store a new context
#[derive(Debug, Deserialize)]
pub struct StoreContextRequest {
    /// Content to store
    pub content: String,

    /// Optional source of the content
    pub source: Option<String>,

    /// Optional content type
    pub content_type: Option<String>,

    /// Optional tags for categorization
    pub tags: Option<Vec<String>>,

    /// Optional custom metadata
    pub metadata: Option<HashMap<String, String>>,
}

/// Request to update an existing context
#[derive(Debug, Deserialize)]
pub struct UpdateContextRequest {
    /// New content
    pub content: String,

    /// Optional source of the content
    pub source: Option<String>,

    /// Optional content type
    pub content_type: Option<String>,

    /// Optional tags for categorization
    pub tags: Option<Vec<String>>,

    /// Optional custom metadata
    pub metadata: Option<HashMap<String, String>>,
}

/// Response containing context information
#[derive(Debug, Serialize)]
pub struct ContextResponse {
    /// Context ID
    pub id: Uuid,

    /// Content
    pub content: String,

    /// Source of the content
    pub source: Option<String>,

    /// Content type
    pub content_type: Option<String>,

    /// Tags
    pub tags: Vec<String>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,

    /// When the context was created
    pub created_at: String,

    /// When the context expires, if applicable
    pub expires_at: Option<String>,
}

/// Request to search for contexts
#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    /// Query string
    pub query: String,

    /// Optional tags to filter by
    pub tags: Option<Vec<String>>,

    /// Maximum number of results to return
    pub limit: Option<usize>,
}

/// Request to retrieve contexts by reference
#[derive(Debug, Deserialize)]
pub struct ReferenceRequest {
    /// List of context references to retrieve
    pub references: Vec<ContextReferenceDto>,
}

/// Data transfer object for context references
#[derive(Debug, Deserialize)]
pub struct ContextReferenceDto {
    /// Context ID
    pub context_id: Uuid,

    /// Optional chunk IDs to retrieve specific chunks
    pub chunk_ids: Option<Vec<Uuid>>,

    /// Optional weight for this reference
    pub weight: Option<f32>,
}

/// Response for search operations
#[derive(Debug, Serialize)]
pub struct SearchResponse {
    /// Matched contexts
    pub matches: Vec<ContextMatchDto>,

    /// Total number of matches
    pub total_matches: usize,
}

/// DTO for a context match
#[derive(Debug, Serialize)]
pub struct ContextMatchDto {
    /// The matched context
    pub context: ContextResponse,

    /// The chunks that matched the query, if any
    pub chunks: Option<Vec<ContextChunkDto>>,

    /// Relevance score
    pub score: f32,
}

/// DTO for a context chunk
#[derive(Debug, Serialize)]
pub struct ContextChunkDto {
    /// Chunk ID
    pub id: Uuid,

    /// Content of this chunk
    pub content: String,

    /// Position of this chunk in the original context
    pub position: usize,
}

/// API error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    /// Error message
    pub message: String,

    /// Error code
    pub code: String,
}
