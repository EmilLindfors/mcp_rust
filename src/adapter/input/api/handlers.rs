use axum::{
    extract::{Path, State, Json},
    http::StatusCode,
    response::{Response, IntoResponse},
};
use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;

use crate::domain::{Context, ContextMetadata, ContextReference, McpError};
use crate::ports::in_ports::{ContextManagementPort, ContextSearchPort};
use super::models::{
    StoreContextRequest, UpdateContextRequest, ContextResponse,
    SearchRequest, ReferenceRequest, SearchResponse,
    ContextMatchDto, ContextChunkDto, ErrorResponse,
};

/// Application state shared between handlers
#[derive(Clone)]
pub struct AppState {
    pub context_manager: Arc<dyn ContextManagementPort + Send + Sync>,
    pub context_search: Arc<dyn ContextSearchPort + Send + Sync>,
}

/// Convert a domain Context to a ContextResponse DTO
fn context_to_response(context: &Context) -> ContextResponse {
    ContextResponse {
        id: context.id,
        content: context.content.clone(),
        source: context.metadata.source.clone(),
        content_type: context.metadata.content_type.clone(),
        tags: context.metadata.tags.clone(),
        metadata: context.metadata.custom.clone(),
        created_at: context.created_at.to_rfc3339(),
        expires_at: context.expires_at.map(|dt| dt.to_rfc3339()),
    }
}

/// Handler for storing a new context
pub async fn store_context(
    State(state): State<AppState>,
    Json(request): Json<StoreContextRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Prepare metadata from request
    let metadata = ContextMetadata {
        source: request.source,
        content_type: request.content_type,
        content_hash: None,
        tags: request.tags.unwrap_or_default(),
        custom: request.metadata.unwrap_or_default(),
    };
    
    // Store context
    let context = state.context_manager.store_context(request.content, metadata).await?;
    
    // Return response
    Ok((StatusCode::CREATED, Json(context_to_response(&context))))
}

/// Handler for retrieving a context by ID
pub async fn get_context(
    State(state): State<AppState>,
    Path(context_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let context = state.context_manager.get_context(context_id).await?;
    Ok((StatusCode::OK, Json(context_to_response(&context))))
}

/// Handler for updating a context
pub async fn update_context(
    State(state): State<AppState>,
    Path(context_id): Path<Uuid>,
    Json(request): Json<UpdateContextRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Prepare metadata from request
    let metadata = ContextMetadata {
        source: request.source,
        content_type: request.content_type,
        content_hash: None,
        tags: request.tags.unwrap_or_default(),
        custom: request.metadata.unwrap_or_default(),
    };
    
    // Update context
    let context = state.context_manager.update_context(context_id, request.content, metadata).await?;
    
    // Return response
    Ok((StatusCode::OK, Json(context_to_response(&context))))
}

/// Handler for deleting a context
pub async fn delete_context(
    State(state): State<AppState>,
    Path(context_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state.context_manager.delete_context(context_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Handler for listing contexts
pub async fn list_contexts(
    State(state): State<AppState>,
    Json(params): Json<HashMap<String, String>>,
) -> Result<impl IntoResponse, ApiError> {
    // Extract optional parameters
    let tags = params.get("tags").map(|t| {
        t.split(',').map(|s| s.trim().to_string()).collect::<Vec<_>>()
    });
    
    let limit = params.get("limit")
        .and_then(|l| l.parse::<usize>().ok())
        .unwrap_or(100);
        
    let offset = params.get("offset")
        .and_then(|o| o.parse::<usize>().ok())
        .unwrap_or(0);
    
    // List contexts
    let contexts = state.context_manager.list_contexts(tags, limit, offset).await?;
    
    // Convert to responses
    let responses: Vec<ContextResponse> = contexts.iter().map(context_to_response).collect();
    
    Ok((StatusCode::OK, Json(responses)))
}

/// Handler for searching contexts
pub async fn search_contexts(
    State(state): State<AppState>,
    Json(request): Json<SearchRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let limit = request.limit.unwrap_or(10);
    
    let search_result = match request.tags {
        Some(tags) if !tags.is_empty() => {
            state.context_search.search_with_tags(request.query, tags, limit).await?
        }
        _ => {
            state.context_search.search(request.query, limit).await?
        }
    };
    
    // Convert domain model to DTO
    let matches = search_result.matches.into_iter().map(|m| {
        let context_response = context_to_response(&m.context);
        
        let chunks = m.chunks.map(|chunks| {
            chunks.into_iter().map(|chunk| {
                ContextChunkDto {
                    id: chunk.chunk_id,
                    content: chunk.content,
                    position: chunk.position,
                }
            }).collect()
        });
        
        ContextMatchDto {
            context: context_response,
            chunks,
            score: m.score,
        }
    }).collect();
    
    let response = SearchResponse {
        matches,
        total_matches: search_result.total_matches,
    };
    
    Ok((StatusCode::OK, Json(response)))
}

/// Handler for retrieving contexts by reference
pub async fn retrieve_by_references(
    State(state): State<AppState>,
    Json(request): Json<ReferenceRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Convert DTOs to domain model
    let references = request.references.into_iter().map(|r| {
        ContextReference {
            context_id: r.context_id,
            chunk_ids: r.chunk_ids,
            weight: r.weight,
        }
    }).collect();
    
    let search_result = state.context_search.retrieve_by_references(references).await?;
    
    // Convert domain model to DTO
    let matches = search_result.matches.into_iter().map(|m| {
        let context_response = context_to_response(&m.context);
        
        let chunks = m.chunks.map(|chunks| {
            chunks.into_iter().map(|chunk| {
                ContextChunkDto {
                    id: chunk.chunk_id,
                    content: chunk.content,
                    position: chunk.position,
                }
            }).collect()
        });
        
        ContextMatchDto {
            context: context_response,
            chunks,
            score: m.score,
        }
    }).collect();
    
    let response = SearchResponse {
        matches,
        total_matches: search_result.total_matches,
    };
    
    Ok((StatusCode::OK, Json(response)))
}

/// Error type for API handlers
#[derive(Debug)]
pub struct ApiError(McpError);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        // Convert the error to status code, error code and message
        let (status, error_code, error_message) = match self.0 {
            McpError::ContextNotFound(_) => 
                (StatusCode::NOT_FOUND, "CONTEXT_NOT_FOUND", "Context not found".to_string()),
                
            McpError::ChunkNotFound(_) => 
                (StatusCode::NOT_FOUND, "CHUNK_NOT_FOUND", "Chunk not found".to_string()),
                
            McpError::InvalidContextReference(msg) => 
                (StatusCode::BAD_REQUEST, "INVALID_REFERENCE", msg),
                
            McpError::ContextAlreadyExists(_) => 
                (StatusCode::CONFLICT, "CONTEXT_EXISTS", "Context already exists".to_string()),
                
            McpError::ValidationError(msg) => 
                (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg),
                
            McpError::AuthenticationError(msg) => 
                (StatusCode::UNAUTHORIZED, "AUTH_ERROR", msg),
                
            McpError::AuthorizationError(msg) => 
                (StatusCode::FORBIDDEN, "FORBIDDEN", msg),
                
            McpError::RateLimitExceeded => 
                (StatusCode::TOO_MANY_REQUESTS, "RATE_LIMIT", "Rate limit exceeded".to_string()),
                
            McpError::ContextLimitExceeded => 
                (StatusCode::TOO_MANY_REQUESTS, "CONTEXT_LIMIT", "Context limit exceeded".to_string()),
                
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "Internal server error".to_string()),
        };
        
        // Create error response
        let error_response = ErrorResponse {
            message: error_message,
            code: error_code.to_string(),
        };
        
        // Return as JSON with appropriate status code
        (status, Json(error_response)).into_response()
    }
}

impl From<McpError> for ApiError {
    fn from(err: McpError) -> Self {
        Self(err)
    }
}