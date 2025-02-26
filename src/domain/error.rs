use thiserror::Error;
use std::io;
use uuid::Uuid;

/// Domain-specific errors for the MCP implementation
#[derive(Error, Debug)]
pub enum McpError {
    #[error("Context not found: {0}")]
    ContextNotFound(Uuid),
    
    #[error("Chunk not found: {0}")]
    ChunkNotFound(Uuid),
    
    #[error("Invalid context reference: {0}")]
    InvalidContextReference(String),
    
    #[error("Context already exists: {0}")]
    ContextAlreadyExists(Uuid),
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Embedding error: {0}")]
    EmbeddingError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    
    #[error("Request validation error: {0}")]
    ValidationError(String),
    
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    
    #[error("Authorization error: {0}")]
    AuthorizationError(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Context limit exceeded")]
    ContextLimitExceeded,
    
    #[error("External service error: {0}")]
    ExternalServiceError(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Result type for MCP operations
pub type McpResult<T> = Result<T, McpError>;