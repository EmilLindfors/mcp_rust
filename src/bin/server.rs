use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

use mcp::adapter::in_adapters::{create_router, AppState};
use mcp::adapter::out_adapters::{InMemoryContextRepository, SimpleEmbeddingService};
use mcp::application::{ContextManagementService, ContextSearchService};
use mcp::config::AppConfig;

/// Command line arguments for the MCP server
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Path to the configuration file
    #[clap(short, long, default_value = "config/default.toml")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Parse command line arguments
    let cli = Cli::parse();

    // Load configuration
    let config = match AppConfig::load() {
        Ok(config) => config,
        Err(err) => {
            error!("Failed to load configuration: {}", err);
            return Err(err.into());
        }
    };

    // Set up the hexagonal architecture
    info!("Initializing MCP components...");

    // Initialize adapters
    let context_repository = Arc::new(InMemoryContextRepository::new());
    let embedding_service = Arc::new(SimpleEmbeddingService::new(config.embedding.dimension));

    // Initialize application services
    let context_manager = Arc::new(ContextManagementService::new(
        context_repository.clone(),
        embedding_service.clone(),
        config.context.max_chunk_size,
        config.context.chunk_overlap,
    ));

    let context_search = Arc::new(ContextSearchService::new(
        context_repository.clone(),
        embedding_service.clone(),
        config.context.max_results,
    ));

    // Initialize the REST API
    let app_state = AppState {
        context_manager,
        context_search,
    };

    // Create the API router
    let app = create_router(app_state);

    // Set up the server address
    let addr = SocketAddr::new(config.server.host.parse()?, config.server.port);

    // Start the server
    info!("Starting MCP server at {}", addr);
    axum::serve(TcpListener::bind(addr).await?, app).await?;

    Ok(())
}
