use std::collections::HashMap;
use std::sync::Arc;

use crate::adapter::output::{InMemoryContextRepository, SimpleEmbeddingService};
use crate::application::ContextManagementService;
use crate::domain::{Context, ContextMetadata};
use crate::ports::in_ports::ContextManagementPort;

#[tokio::test]
async fn test_store_and_retrieve_context() {
    // Initialize adapters
    let context_repository = Arc::new(InMemoryContextRepository::new());
    let embedding_service = Arc::new(SimpleEmbeddingService::new(128));

    // Initialize application service
    let context_service = Arc::new(ContextManagementService::new(
        context_repository.clone(),
        embedding_service.clone(),
        1000, // max_chunk_size
        200,  // chunk_overlap
    ));

    // Test storing a context
    let content = "This is a test context for the Model Context Protocol implementation";
    let metadata = ContextMetadata {
        source: Some("test".to_string()),
        content_type: Some("text/plain".to_string()),
        content_hash: None,
        tags: vec!["test".to_string(), "example".to_string()],
        custom: HashMap::new(),
    };

    // Store context
    let stored_context = context_service
        .store_context(content.to_string(), metadata)
        .await
        .expect("Failed to store context");

    // Verify context was stored
    assert_eq!(stored_context.content, content);
    assert_eq!(stored_context.metadata.tags, vec!["test", "example"]);
    assert_eq!(stored_context.metadata.source, Some("test".to_string()));

    // Retrieve context by ID
    let retrieved_context = context_service
        .get_context(stored_context.id)
        .await
        .expect("Failed to retrieve context");

    // Verify retrieved context matches stored context
    assert_eq!(retrieved_context.id, stored_context.id);
    assert_eq!(retrieved_context.content, stored_context.content);
    assert_eq!(
        retrieved_context.metadata.tags,
        stored_context.metadata.tags
    );

    // List contexts with tags
    let contexts_with_tags = context_service
        .list_contexts(Some(vec!["test".to_string()]), 10, 0)
        .await
        .expect("Failed to list contexts");

    // Verify contexts with tags
    assert_eq!(contexts_with_tags.len(), 1);
    assert_eq!(contexts_with_tags[0].id, stored_context.id);

    // Test updating a context
    let updated_content = "This is an updated test context";
    let updated_metadata = ContextMetadata {
        source: Some("test-updated".to_string()),
        content_type: Some("text/plain".to_string()),
        content_hash: None,
        tags: vec!["test".to_string(), "updated".to_string()],
        custom: HashMap::new(),
    };

    let updated_context = context_service
        .update_context(
            stored_context.id,
            updated_content.to_string(),
            updated_metadata,
        )
        .await
        .expect("Failed to update context");

    // Verify updated context
    assert_eq!(updated_context.id, stored_context.id);
    assert_eq!(updated_context.content, updated_content);
    assert_eq!(updated_context.metadata.tags, vec!["test", "updated"]);

    // Test deleting a context
    context_service
        .delete_context(stored_context.id)
        .await
        .expect("Failed to delete context");

    // Verify context was deleted
    let result = context_service.get_context(stored_context.id).await;
    assert!(result.is_err(), "Context should have been deleted");
}
