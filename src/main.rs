use clap::{Parser, Subcommand};

/// MCP - Model Context Protocol
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start the MCP server
    Server {
        /// Path to the configuration file
        #[clap(short, long, default_value = "config/default.toml")]
        config: String,
    },

    /// Use the MCP client
    Client {
        /// Server URL
        #[clap(short, long, default_value = "http://localhost:3000")]
        server: String,

        #[clap(subcommand)]
        command: ClientCommands,
    },
}

#[derive(Subcommand, Debug)]
enum ClientCommands {
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

fn main() {
    println!("MCP - Model Context Protocol");
    println!("Please use the specific binaries:");
    println!("  - mcp-server: for running the server");
    println!("  - mcp-client: for using the client");
    println!();
    println!("Alternatively, you can install the binaries with:");
    println!("  cargo install --path . --bins");
}
