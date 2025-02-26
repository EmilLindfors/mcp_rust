use axum::{
    routing::{delete, get, post, put},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use super::handlers::{
    delete_context, get_context, list_contexts, retrieve_by_references, search_contexts,
    store_context, update_context, AppState,
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
