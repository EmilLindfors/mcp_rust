use clap::{Parser, Subcommand};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, Write};
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

/// MCP client for interacting with the Model Context Protocol server
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Server URL
    #[clap(short, long, default_value = "http://localhost:3000")]
    server: String,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Store a new context
    Store {
        /// Content to store
        #[clap(short, long)]
        content: String,

        /// Source of the content (optional)
        #[clap(short, long)]
        source: Option<String>,

        /// Content type (optional)
        #[clap(short, long)]
        content_type: Option<String>,

        /// Tags (comma-separated, optional)
        #[clap(short, long)]
        tags: Option<String>,
    },

    /// Retrieve a context by ID
    Get {
        /// Context ID to retrieve
        #[clap(short, long)]
        id: String,
    },

    /// List all contexts
    List {
        /// Filter by tags (comma-separated, optional)
        #[clap(short, long)]
        tags: Option<String>,

        /// Maximum number of contexts to return
        #[clap(short, long, default_value = "10")]
        limit: usize,
    },

    /// Search for contexts by content
    Search {
        /// Query string
        #[clap(short, long)]
        query: String,

        /// Filter by tags (comma-separated, optional)
        #[clap(short, long)]
        tags: Option<String>,

        /// Maximum number of results to return
        #[clap(short, long, default_value = "5")]
        limit: usize,
    },

    /// Update an existing context
    Update {
        /// Context ID to update
        #[clap(short, long)]
        id: String,

        /// New content
        #[clap(short, long)]
        content: String,

        /// Source of the content (optional)
        #[clap(long)]
        source: Option<String>,

        /// Content type (optional)
        #[clap(long)]
        content_type: Option<String>,

        /// Tags (comma-separated, optional)
        #[clap(short, long)]
        tags: Option<String>,
    },

    /// Delete a context
    Delete {
        /// Context ID to delete
        #[clap(short, long)]
        id: String,
    },

    /// Interactive mode to explore the MCP capabilities
    Interactive,
}

// Request and response types for API interaction

#[derive(Debug, Serialize)]
struct StoreContextRequest {
    content: String,
    source: Option<String>,
    content_type: Option<String>,
    tags: Option<Vec<String>>,
    metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
struct UpdateContextRequest {
    content: String,
    source: Option<String>,
    content_type: Option<String>,
    tags: Option<Vec<String>>,
    metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
struct SearchRequest {
    query: String,
    tags: Option<Vec<String>>,
    limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct ContextResponse {
    id: Uuid,
    content: String,
    source: Option<String>,
    content_type: Option<String>,
    tags: Vec<String>,
    metadata: HashMap<String, String>,
    created_at: String,
    expires_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ContextChunkDto {
    id: Uuid,
    content: String,
    position: usize,
}

#[derive(Debug, Deserialize)]
struct ContextMatchDto {
    context: ContextResponse,
    chunks: Option<Vec<ContextChunkDto>>,
    score: f32,
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    matches: Vec<ContextMatchDto>,
    total_matches: usize,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    message: String,
    code: String,
}

// Helper function to parse comma-separated tags
fn parse_tags(tags_str: Option<String>) -> Option<Vec<String>> {
    tags_str.map(|s| {
        s.split(',')
            .map(|tag| tag.trim().to_string())
            .filter(|tag| !tag.is_empty())
            .collect()
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let cli = Cli::parse();

    // Create HTTP client
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    // Process command
    match cli.command {
        Command::Store {
            content,
            source,
            content_type,
            tags,
        } => {
            store_context(
                &client,
                &cli.server,
                content,
                source,
                content_type,
                parse_tags(tags),
            )
            .await?;
        }

        Command::Get { id } => {
            get_context(&client, &cli.server, &id).await?;
        }

        Command::List { tags, limit } => {
            list_contexts(&client, &cli.server, parse_tags(tags), limit).await?;
        }

        Command::Search { query, tags, limit } => {
            search_contexts(&client, &cli.server, query, parse_tags(tags), limit).await?;
        }

        Command::Update {
            id,
            content,
            source,
            content_type,
            tags,
        } => {
            update_context(
                &client,
                &cli.server,
                &id,
                content,
                source,
                content_type,
                parse_tags(tags),
            )
            .await?;
        }

        Command::Delete { id } => {
            delete_context(&client, &cli.server, &id).await?;
        }

        Command::Interactive => {
            run_interactive_mode(&client, &cli.server).await?;
        }
    }

    Ok(())
}

// API interaction functions

async fn store_context(
    client: &Client,
    server: &str,
    content: String,
    source: Option<String>,
    content_type: Option<String>,
    tags: Option<Vec<String>>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Storing new context...");

    let request = StoreContextRequest {
        content,
        source,
        content_type,
        tags,
        metadata: None,
    };

    let response = client
        .post(&format!("{}/contexts", server))
        .json(&request)
        .send()
        .await?;

    if response.status().is_success() {
        let context: ContextResponse = response.json().await?;
        println!("Context stored successfully!");
        println!("ID: {}", context.id);
        println!("Content: {}", context.content);
        println!("Tags: {:?}", context.tags);
        println!("Created at: {}", context.created_at);
    } else {
        handle_error_response(response).await?;
    }

    Ok(())
}

async fn get_context(
    client: &Client,
    server: &str,
    id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Retrieving context with ID: {}...", id);

    let response = client
        .get(&format!("{}/contexts/{}", server, id))
        .send()
        .await?;

    if response.status().is_success() {
        let context: ContextResponse = response.json().await?;
        println!("Context retrieved successfully!");
        println!("ID: {}", context.id);
        println!("Content: {}", context.content);
        println!("Source: {:?}", context.source);
        println!("Content type: {:?}", context.content_type);
        println!("Tags: {:?}", context.tags);
        println!("Created at: {}", context.created_at);
    } else {
        handle_error_response(response).await?;
    }

    Ok(())
}

async fn list_contexts(
    client: &Client,
    server: &str,
    tags: Option<Vec<String>>,
    limit: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Listing contexts...");

    // Create parameters
    let mut params = HashMap::new();
    if let Some(tags) = tags {
        params.insert("tags".to_string(), tags.join(","));
    }
    params.insert("limit".to_string(), limit.to_string());

    let response = client
        .get(&format!("{}/contexts", server))
        .json(&params)
        .send()
        .await?;

    if response.status().is_success() {
        let contexts: Vec<ContextResponse> = response.json().await?;
        println!("Found {} contexts:", contexts.len());

        for (i, context) in contexts.iter().enumerate() {
            println!("\n--- Context {} ---", i + 1);
            println!("ID: {}", context.id);
            println!("Content: {}", context.content);
            println!("Tags: {:?}", context.tags);
        }
    } else {
        handle_error_response(response).await?;
    }

    Ok(())
}

async fn search_contexts(
    client: &Client,
    server: &str,
    query: String,
    tags: Option<Vec<String>>,
    limit: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Searching for contexts with query: \"{}\"...", query);

    let request = SearchRequest {
        query,
        tags,
        limit: Some(limit),
    };

    let response = client
        .post(&format!("{}/search", server))
        .json(&request)
        .send()
        .await?;

    if response.status().is_success() {
        let search_result: SearchResponse = response.json().await?;
        println!(
            "Found {} matches (out of {} total):",
            search_result.matches.len(),
            search_result.total_matches
        );

        for (i, match_item) in search_result.matches.iter().enumerate() {
            println!("\n--- Match {} (score: {:.2}) ---", i + 1, match_item.score);
            println!("ID: {}", match_item.context.id);
            println!("Content: {}", match_item.context.content);
            println!("Tags: {:?}", match_item.context.tags);

            if let Some(chunks) = &match_item.chunks {
                println!("Matching chunks: {}", chunks.len());
                for chunk in chunks.iter().take(2) {
                    println!("  - {}", chunk.content);
                }
                if chunks.len() > 2 {
                    println!("  ... {} more chunks", chunks.len() - 2);
                }
            }
        }
    } else {
        handle_error_response(response).await?;
    }

    Ok(())
}

async fn update_context(
    client: &Client,
    server: &str,
    id: &str,
    content: String,
    source: Option<String>,
    content_type: Option<String>,
    tags: Option<Vec<String>>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Updating context with ID: {}...", id);

    let request = UpdateContextRequest {
        content,
        source,
        content_type,
        tags,
        metadata: None,
    };

    let response = client
        .put(&format!("{}/contexts/{}", server, id))
        .json(&request)
        .send()
        .await?;

    if response.status().is_success() {
        let context: ContextResponse = response.json().await?;
        println!("Context updated successfully!");
        println!("ID: {}", context.id);
        println!("New content: {}", context.content);
        println!("Tags: {:?}", context.tags);
    } else {
        handle_error_response(response).await?;
    }

    Ok(())
}

async fn delete_context(
    client: &Client,
    server: &str,
    id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Deleting context with ID: {}...", id);

    let response = client
        .delete(&format!("{}/contexts/{}", server, id))
        .send()
        .await?;

    if response.status().is_success() {
        println!("Context deleted successfully!");
    } else {
        handle_error_response(response).await?;
    }

    Ok(())
}

async fn handle_error_response(
    response: reqwest::Response,
) -> Result<(), Box<dyn std::error::Error>> {
    let status = response.status();

    match response.json::<ErrorResponse>().await {
        Ok(error) => {
            eprintln!("Error ({}): {} ({})", status, error.message, error.code);
        }
        Err(_) => {
            eprintln!("Error ({}): Failed to parse error response", status);
        }
    }

    Ok(())
}

// Interactive mode
async fn run_interactive_mode(
    client: &Client,
    server: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MCP Interactive Client ===");
    println!("Server: {}", server);
    println!();

    // Try connecting to the server
    println!("Checking server connection...");
    match client
        .get(&format!("{}/contexts", server))
        .timeout(Duration::from_secs(5))
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {
            println!("Server connection successful!");
        }
        Ok(response) => {
            println!(
                "Connected to server but received status code: {}",
                response.status()
            );
        }
        Err(e) => {
            println!("Failed to connect to server: {}", e);
            println!("Please make sure the server is running at {}", server);
            return Ok(());
        }
    }

    println!();
    println!("Available commands:");
    println!("  1. Store a new context");
    println!("  2. Get a context by ID");
    println!("  3. List contexts");
    println!("  4. Search contexts");
    println!("  5. Update a context");
    println!("  6. Delete a context");
    println!("  q. Quit");
    println!();

    loop {
        print!("Enter command (1-6, q): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        match input {
            "1" => {
                // Store context
                print!("Enter content: ");
                io::stdout().flush()?;
                let mut content = String::new();
                io::stdin().read_line(&mut content)?;

                print!("Enter source (optional): ");
                io::stdout().flush()?;
                let mut source = String::new();
                io::stdin().read_line(&mut source)?;
                let source = if source.trim().is_empty() {
                    None
                } else {
                    Some(source.trim().to_string())
                };

                print!("Enter content type (optional): ");
                io::stdout().flush()?;
                let mut content_type = String::new();
                io::stdin().read_line(&mut content_type)?;
                let content_type = if content_type.trim().is_empty() {
                    None
                } else {
                    Some(content_type.trim().to_string())
                };

                print!("Enter tags (comma-separated, optional): ");
                io::stdout().flush()?;
                let mut tags_str = String::new();
                io::stdin().read_line(&mut tags_str)?;
                let tags = parse_tags(if tags_str.trim().is_empty() {
                    None
                } else {
                    Some(tags_str.trim().to_string())
                });

                store_context(
                    client,
                    server,
                    content.trim().to_string(),
                    source,
                    content_type,
                    tags,
                )
                .await?;
            }

            "2" => {
                // Get context
                print!("Enter context ID: ");
                io::stdout().flush()?;
                let mut id = String::new();
                io::stdin().read_line(&mut id)?;

                get_context(client, server, id.trim()).await?;
            }

            "3" => {
                // List contexts
                print!("Enter tags to filter (comma-separated, optional): ");
                io::stdout().flush()?;
                let mut tags_str = String::new();
                io::stdin().read_line(&mut tags_str)?;
                let tags = parse_tags(if tags_str.trim().is_empty() {
                    None
                } else {
                    Some(tags_str.trim().to_string())
                });

                print!("Enter limit (default 10): ");
                io::stdout().flush()?;
                let mut limit_str = String::new();
                io::stdin().read_line(&mut limit_str)?;
                let limit = limit_str.trim().parse::<usize>().unwrap_or(10);

                list_contexts(client, server, tags, limit).await?;
            }

            "4" => {
                // Search contexts
                print!("Enter search query: ");
                io::stdout().flush()?;
                let mut query = String::new();
                io::stdin().read_line(&mut query)?;

                print!("Enter tags to filter (comma-separated, optional): ");
                io::stdout().flush()?;
                let mut tags_str = String::new();
                io::stdin().read_line(&mut tags_str)?;
                let tags = parse_tags(if tags_str.trim().is_empty() {
                    None
                } else {
                    Some(tags_str.trim().to_string())
                });

                print!("Enter limit (default 5): ");
                io::stdout().flush()?;
                let mut limit_str = String::new();
                io::stdin().read_line(&mut limit_str)?;
                let limit = limit_str.trim().parse::<usize>().unwrap_or(5);

                search_contexts(client, server, query.trim().to_string(), tags, limit).await?;
            }

            "5" => {
                // Update context
                print!("Enter context ID to update: ");
                io::stdout().flush()?;
                let mut id = String::new();
                io::stdin().read_line(&mut id)?;

                print!("Enter new content: ");
                io::stdout().flush()?;
                let mut content = String::new();
                io::stdin().read_line(&mut content)?;

                print!("Enter new source (optional): ");
                io::stdout().flush()?;
                let mut source = String::new();
                io::stdin().read_line(&mut source)?;
                let source = if source.trim().is_empty() {
                    None
                } else {
                    Some(source.trim().to_string())
                };

                print!("Enter new content type (optional): ");
                io::stdout().flush()?;
                let mut content_type = String::new();
                io::stdin().read_line(&mut content_type)?;
                let content_type = if content_type.trim().is_empty() {
                    None
                } else {
                    Some(content_type.trim().to_string())
                };

                print!("Enter new tags (comma-separated, optional): ");
                io::stdout().flush()?;
                let mut tags_str = String::new();
                io::stdin().read_line(&mut tags_str)?;
                let tags = parse_tags(if tags_str.trim().is_empty() {
                    None
                } else {
                    Some(tags_str.trim().to_string())
                });

                update_context(
                    client,
                    server,
                    id.trim(),
                    content.trim().to_string(),
                    source,
                    content_type,
                    tags,
                )
                .await?;
            }

            "6" => {
                // Delete context
                print!("Enter context ID to delete: ");
                io::stdout().flush()?;
                let mut id = String::new();
                io::stdin().read_line(&mut id)?;

                print!("Are you sure you want to delete this context? (y/n): ");
                io::stdout().flush()?;
                let mut confirm = String::new();
                io::stdin().read_line(&mut confirm)?;

                if confirm.trim().to_lowercase() == "y" {
                    delete_context(client, server, id.trim()).await?;
                } else {
                    println!("Delete operation cancelled.");
                }
            }

            "q" | "quit" | "exit" => {
                println!("Goodbye!");
                break;
            }

            _ => {
                println!("Invalid command. Please enter a number from 1-6 or 'q' to quit.");
            }
        }

        println!();
        println!("Press Enter to continue...");
        let mut pause = String::new();
        io::stdin().read_line(&mut pause)?;
        println!();
    }

    Ok(())
}
