# Model Context Protocol (MCP) Implementation in Rust

A Rust implementation of the Model Context Protocol, designed to handle the storage, retrieval, and management of context for large language models.

## Architecture

This project follows the hexagonal architecture pattern (also known as ports and adapters):

- **Domain**: Core business logic and entities
- **Ports**: Interfaces defining how the application interacts with the outside world
- **Application**: Use cases that orchestrate the domain logic
- **Adapters**: Implementations of ports that connect to external systems

## Features

- Context storage and management 
- Intelligent context chunking and embedding
- Semantic search capabilities
- RESTful API for integrating with LLM systems
- High performance Rust implementation

## Getting Started

### Prerequisites

- Rust 1.76 or later
- Cargo

### Installation

1. Clone the repository:
   ```sh
   git clone https://github.com/your-username/mcp.git
   cd mcp
   ```

2. Build the project:
   ```sh
   cargo build --release
   ```

## Usage

### Running the Server

```sh
cargo run --bin mcp-server
# Or, if installed:
mcp-server
```

This will start the MCP server on the default port (3000).

### Using the Client

There are several ways to use the client:

1. Interactive mode:
   ```sh
   cargo run --bin mcp-client -- interactive
   # Or, if installed:
   mcp-client interactive
   ```

2. Command line usage:
   ```sh
   # Store a new context
   cargo run --bin mcp-client -- store --content "This is a test context" --tags "test,example"
   
   # Search for contexts
   cargo run --bin mcp-client -- search --query "test" --limit 5
   
   # Get a context by ID
   cargo run --bin mcp-client -- get --id "<context-id>"
   
   # List all contexts
   cargo run --bin mcp-client -- list
   
   # Update a context
   cargo run --bin mcp-client -- update --id "<context-id>" --content "Updated content"
   
   # Delete a context
   cargo run --bin mcp-client -- delete --id "<context-id>"
   ```

3. Connect to a different server:
   ```sh
   cargo run --bin mcp-client -- --server "http://other-server:3000" interactive
   ```

### Configuration

Configuration can be provided via:
- A config file (default: `config/default.toml`)
- Environment variables (prefixed with `MCP__`)

Example configuration:

```toml
[server]
host = "127.0.0.1"
port = 3000

[context]
max_chunk_size = 1000
chunk_overlap = 200
max_results = 10

[embedding]
dimension = 768
```

## API Endpoints

### Context Management

- `POST /contexts` - Store a new context
- `GET /contexts/:id` - Retrieve a context by ID
- `PUT /contexts/:id` - Update an existing context
- `DELETE /contexts/:id` - Delete a context
- `GET /contexts` - List all contexts

### Context Search

- `POST /search` - Search for contexts using semantic search
- `POST /references` - Retrieve contexts by reference

## Testing

### Unit Tests

Run unit tests with:

```sh
cargo test
```

### Integration Tests

The project includes comprehensive integration tests that validate the interaction between the client and server components. These tests start a server instance on a random port, send requests using the client code, and verify the responses.

To run the integration tests with cargo-nextest (for better reporting and parallelism):

```sh
# Run the provided script
./run_integration_tests.sh

# Or manually:
cargo nextest run --profile integration
```

The integration tests cover:
- Basic CRUD operations on contexts
- Search functionality with different queries and filters
- Error handling and edge cases
- Context chunking and embedding

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.