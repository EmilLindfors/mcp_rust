use serde::Deserialize;
use std::path::Path;
use config::{Config, ConfigError, File, Environment};

/// Configuration for the MCP server
#[derive(Debug, Deserialize)]
pub struct AppConfig {
    /// Server configuration
    pub server: ServerConfig,
    
    /// Context processing configuration
    pub context: ContextConfig,
    
    /// Embedding configuration
    pub embedding: EmbeddingConfig,
}

/// Server configuration
#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    /// Host to bind to
    pub host: String,
    
    /// Port to listen on
    pub port: u16,
    
    /// API key for authentication (optional)
    pub api_key: Option<String>,
}

/// Context processing configuration
#[derive(Debug, Deserialize)]
pub struct ContextConfig {
    /// Maximum size of a context chunk in characters
    pub max_chunk_size: usize,
    
    /// Overlap between chunks in characters
    pub chunk_overlap: usize,
    
    /// Maximum number of results to return in searches
    pub max_results: usize,
}

/// Embedding configuration
#[derive(Debug, Deserialize)]
pub struct EmbeddingConfig {
    /// Dimension of embeddings to use
    pub dimension: usize,
}

impl AppConfig {
    /// Load configuration from file and environment variables
    pub fn load() -> Result<Self, ConfigError> {
        // Set default configuration
        let config = Config::builder()
            // Start with defaults
            .set_default("server.host", "127.0.0.1")?
            .set_default("server.port", 3000)?
            .set_default("context.max_chunk_size", 1000)?
            .set_default("context.chunk_overlap", 200)?
            .set_default("context.max_results", 10)?
            .set_default("embedding.dimension", 768)?
            
            // Load from config file if it exists
            .add_source(File::from(Path::new("config/default.toml")).required(false))
            
            // Override with environment variables (e.g., MCP_SERVER__PORT=8080)
            .add_source(Environment::with_prefix("MCP").separator("__"))
            
            .build()?;
            
        // Deserialize into AppConfig
        config.try_deserialize()
    }
}