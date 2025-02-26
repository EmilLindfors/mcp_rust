use axum::{
    routing::{get, post, put, delete},
    Router,
};
use tower_http::trace::TraceLayer;
use tower_http::cors::{CorsLayer, Any};

use super::handlers::{
    AppState,
    store_context,
    get_context,
    update_context,
    delete_context,
    list_contexts,
    search_contexts,
    retrieve_by_references,
};

/// Create the API router with all endpoints
pub fn create_router(state: AppState) -> Router {
    // Set up CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build the router with all routes
    Router::new()
        // Context management
        .route("/contexts", post(store_context))
        .route("/contexts", get(list_contexts))
        .route("/contexts/:id", get(get_context))
        .route("/contexts/:id", put(update_context))
        .route("/contexts/:id", delete(delete_context))
        
        // Context search
        .route("/search", post(search_contexts))
        .route("/references", post(retrieve_by_references))
        
        // Add middleware
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}