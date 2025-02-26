use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use std::collections::HashMap;
use uuid::Uuid;

use mcp::application::{ContextManagementService, ContextSearchService};
use mcp::adapter::out_adapters::{InMemoryContextRepository, SimpleEmbeddingService};
use mcp::adapter::in_adapters::{create_router, AppState};
use mcp::domain::ContextMetadata;

/// Setup a test server on a random port for testing
async fn setup_test_server() -> (SocketAddr, oneshot::Sender<()>, JoinHandle<()>) {
    // Set up a random available port for the server
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let server_addr = listener.local_addr().unwrap();
    
    // Set up channels for shutting down the server
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    
    // Initialize adapters
    let context_repository = Arc::new(InMemoryContextRepository::new());
    let embedding_service = Arc::new(SimpleEmbeddingService::new(128));
    
    // Initialize application services
    let context_manager = Arc::new(ContextManagementService::new(
        context_repository.clone(),
        embedding_service.clone(),
        1000, // max_chunk_size
        200,  // chunk_overlap
    ));
    
    let context_search = Arc::new(ContextSearchService::new(
        context_repository.clone(),
        embedding_service.clone(),
        10, // max_results
    ));
    
    // Set up the app state
    let app_state = AppState {
        context_manager,
        context_search,
    };
    
    // Create the router
    let app = create_router(app_state);
    
    // Start the server in a separate task
    let server_handle = tokio::spawn(async move {
        let server = axum::serve(listener, app)
            .with_graceful_shutdown(async {
                shutdown_rx.await.ok();
            });
            
        server.await.unwrap();
    });
    
    // Wait a moment for the server to be fully running
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    (server_addr, shutdown_tx, server_handle)
}

#[tokio::test]
async fn test_client_server_interaction() {
    // Start a test server
    let (server_addr, shutdown_tx, server_handle) = setup_test_server().await;
    let base_url = format!("http://{}", server_addr);
    
    // Create an HTTP client
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
        
    // Test 1: Store a context
    let content = "This is a test context for the integration test";
    let test_metadata = ContextMetadata {
        source: Some("test".to_string()),
        content_type: Some("text/plain".to_string()),
        content_hash: None,
        tags: vec!["test".to_string(), "integration".to_string()],
        custom: HashMap::new(),
    };
    
    // Convert to the request structure expected by the API
    let store_request = serde_json::json!({
        "content": content,
        "source": test_metadata.source,
        "content_type": test_metadata.content_type,
        "tags": test_metadata.tags,
        "metadata": test_metadata.custom,
    });
    
    // Send the request to store a context
    let response = client.post(&format!("{}/contexts", base_url))
        .json(&store_request)
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 201); // Created
    
    // Parse the response
    let context_response: serde_json::Value = response.json().await.unwrap();
    
    // Extract context ID from the response
    let context_id = context_response["id"].as_str().unwrap();
    let saved_content = context_response["content"].as_str().unwrap();
    
    // Verify the content was stored correctly
    assert_eq!(saved_content, content);
    
    // Verify the ID is a valid UUID
    let uuid = Uuid::parse_str(context_id).unwrap();
    assert!(uuid.as_u128() > 0);
    
    // Test 2: Get the stored context
    let response = client.get(&format!("{}/contexts/{}", base_url, context_id))
        .send()
        .await
        .unwrap();
        
    assert_eq!(response.status(), 200); // OK
    
    let get_response: serde_json::Value = response.json().await.unwrap();
    assert_eq!(get_response["id"].as_str().unwrap(), context_id);
    assert_eq!(get_response["content"].as_str().unwrap(), content);
    
    // Verify the tags were stored
    let tags = get_response["tags"].as_array().unwrap();
    assert!(tags.contains(&serde_json::Value::String("test".to_string())));
    assert!(tags.contains(&serde_json::Value::String("integration".to_string())));
    
    // Test 3: List contexts
    let response = client.get(&format!("{}/contexts", base_url))
        .json(&serde_json::json!({
            "limit": 10,
            "offset": 0
        }))
        .send()
        .await
        .unwrap();
        
    assert_eq!(response.status(), 200); // OK
    
    let list_response: Vec<serde_json::Value> = response.json().await.unwrap();
    assert!(!list_response.is_empty());
    
    // Should contain our created context
    let found = list_response.iter().any(|ctx| ctx["id"].as_str().unwrap() == context_id);
    assert!(found, "Created context should be in the list response");
    
    // Test 4: Search contexts
    let search_query = "integration test";
    let response = client.post(&format!("{}/search", base_url))
        .json(&serde_json::json!({
            "query": search_query,
            "limit": 5
        }))
        .send()
        .await
        .unwrap();
        
    assert_eq!(response.status(), 200); // OK
    
    let search_response: serde_json::Value = response.json().await.unwrap();
    let matches = search_response["matches"].as_array().unwrap();
    assert!(!matches.is_empty());
    
    // Should have a reasonable score
    let score = matches[0]["score"].as_f64().unwrap();
    assert!(score > 0.0);
    
    // Test 5: Update a context
    let updated_content = "This is an updated test context for the integration test";
    let update_request = serde_json::json!({
        "content": updated_content,
        "source": test_metadata.source,
        "content_type": test_metadata.content_type,
        "tags": ["test", "updated"],
    });
    
    let response = client.put(&format!("{}/contexts/{}", base_url, context_id))
        .json(&update_request)
        .send()
        .await
        .unwrap();
        
    assert_eq!(response.status(), 200); // OK
    
    let update_response: serde_json::Value = response.json().await.unwrap();
    assert_eq!(update_response["content"].as_str().unwrap(), updated_content);
    
    // Get the updated context to verify
    let response = client.get(&format!("{}/contexts/{}", base_url, context_id))
        .send()
        .await
        .unwrap();
        
    assert_eq!(response.status(), 200); // OK
    
    let get_updated_response: serde_json::Value = response.json().await.unwrap();
    assert_eq!(get_updated_response["content"].as_str().unwrap(), updated_content);
    
    // Verify the tags were updated
    let tags = get_updated_response["tags"].as_array().unwrap();
    assert!(tags.contains(&serde_json::Value::String("test".to_string())));
    assert!(tags.contains(&serde_json::Value::String("updated".to_string())));
    assert!(!tags.contains(&serde_json::Value::String("integration".to_string())));
    
    // Test 6: Delete the context
    let response = client.delete(&format!("{}/contexts/{}", base_url, context_id))
        .send()
        .await
        .unwrap();
        
    assert_eq!(response.status(), 204); // No Content
    
    // Verify context is deleted by trying to get it
    let response = client.get(&format!("{}/contexts/{}", base_url, context_id))
        .send()
        .await
        .unwrap();
        
    assert_eq!(response.status(), 404); // Not Found
    
    // Shutdown the server
    shutdown_tx.send(()).unwrap();
    let _ = server_handle.await;
}

#[tokio::test]
async fn test_client_error_handling() {
    // Start a test server
    let (server_addr, shutdown_tx, server_handle) = setup_test_server().await;
    let base_url = format!("http://{}", server_addr);
    
    // Create HTTP client
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    
    // Test 1: Get a non-existent context
    let non_existent_id = Uuid::new_v4().to_string();
    let response = client.get(&format!("{}/contexts/{}", base_url, non_existent_id))
        .send()
        .await
        .unwrap();
        
    assert_eq!(response.status(), 404); // Not Found
    
    let error_response: serde_json::Value = response.json().await.unwrap();
    assert_eq!(error_response["code"], "CONTEXT_NOT_FOUND");
    
    // Test 2: Invalid search request (missing query)
    let response = client.post(&format!("{}/search", base_url))
        .json(&serde_json::json!({
            // Missing required "query" field
            "limit": 5
        }))
        .send()
        .await
        .unwrap();
        
    assert!(response.status().is_client_error()); 
    
    // Test 3: Update a non-existent context
    let non_existent_id = Uuid::new_v4().to_string();
    let update_request = serde_json::json!({
        "content": "This won't work",
        "tags": ["test"],
    });
    
    let response = client.put(&format!("{}/contexts/{}", base_url, non_existent_id))
        .json(&update_request)
        .send()
        .await
        .unwrap();
        
    assert_eq!(response.status(), 404); // Not Found
    
    // Shutdown the server
    shutdown_tx.send(()).unwrap();
    let _ = server_handle.await;
}

#[tokio::test]
async fn test_context_search_functionality() {
    // Start a test server
    let (server_addr, shutdown_tx, server_handle) = setup_test_server().await;
    let base_url = format!("http://{}", server_addr);
    
    // Create an HTTP client
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
        
    // Store multiple contexts with different content and tags
    let contexts = vec![
        (
            "Neural networks are a class of machine learning models inspired by the human brain.",
            vec!["ai", "machine_learning"]
        ),
        (
            "Transformers have revolutionized natural language processing and understanding.",
            vec!["ai", "nlp", "transformers"]
        ),
        (
            "Claude is an AI assistant created by Anthropic to be helpful, harmless, and honest.",
            vec!["ai", "assistant", "anthropic"]
        ),
        (
            "Rust is a systems programming language focused on safety, speed, and concurrency.",
            vec!["programming", "systems", "rust"]
        ),
        (
            "Python is a popular high-level programming language known for its readability.",
            vec!["programming", "python"]
        ),
    ];
    
    // Store all the contexts
    let mut context_ids = Vec::new();
    
    for (content, tags) in contexts {
        let store_request = serde_json::json!({
            "content": content,
            "tags": tags,
        });
        
        let response = client.post(&format!("{}/contexts", base_url))
            .json(&store_request)
            .send()
            .await
            .unwrap();
            
        assert_eq!(response.status(), 201);
        
        let context_response: serde_json::Value = response.json().await.unwrap();
        context_ids.push(context_response["id"].as_str().unwrap().to_string());
    }
    
    // Test 1: Search by content
    let response = client.post(&format!("{}/search", base_url))
        .json(&serde_json::json!({
            "query": "ai language",
            "limit": 10
        }))
        .send()
        .await
        .unwrap();
        
    assert_eq!(response.status(), 200);
    
    let search_response: serde_json::Value = response.json().await.unwrap();
    let matches = search_response["matches"].as_array().unwrap();
    
    // Should have matches related to AI and language
    assert!(!matches.is_empty());
    
    // Test 2: Search with tag filtering
    let response = client.post(&format!("{}/search", base_url))
        .json(&serde_json::json!({
            "query": "programming",
            "tags": ["rust"],
            "limit": 10
        }))
        .send()
        .await
        .unwrap();
        
    assert_eq!(response.status(), 200);
    
    let search_response: serde_json::Value = response.json().await.unwrap();
    let matches = search_response["matches"].as_array().unwrap();
    
    // Should only match the Rust programming context
    assert!(!matches.is_empty());
    let matched_content = matches[0]["context"]["content"].as_str().unwrap();
    assert!(matched_content.contains("Rust"));
    
    // Test 3: List contexts filtered by tags
    let response = client.get(&format!("{}/contexts", base_url))
        .json(&serde_json::json!({
            "tags": "ai,nlp",
            "limit": 10
        }))
        .send()
        .await
        .unwrap();
        
    assert_eq!(response.status(), 200);
    
    let list_response: Vec<serde_json::Value> = response.json().await.unwrap();
    
    // Should only return contexts with both "ai" and "nlp" tags
    assert!(!list_response.is_empty());
    assert_eq!(list_response.len(), 1); // Only the transformers context has both tags
    
    let matched_content = list_response[0]["content"].as_str().unwrap();
    assert!(matched_content.contains("Transformers"));
    
    // Clean up - delete all created contexts
    for id in context_ids {
        let response = client.delete(&format!("{}/contexts/{}", base_url, id))
            .send()
            .await
            .unwrap();
            
        assert_eq!(response.status(), 204); // No Content
    }
    
    // Shutdown the server
    shutdown_tx.send(()).unwrap();
    let _ = server_handle.await;
}