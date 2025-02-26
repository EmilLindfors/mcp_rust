# MCP Basic Usage Example

This example shows how to start the server and use the client to interact with it.

## 1. Start the Server

In one terminal, start the MCP server:

```sh
# Using cargo
cargo run --bin mcp-server

# Or using the installed binary
mcp-server
```

You should see output like:

```
2024-02-25T12:34:56Z INFO  mcp_server > Initializing MCP components...
2024-02-25T12:34:56Z INFO  mcp_server > Starting MCP server at 127.0.0.1:3000
```

## 2. Store a Context

In another terminal, use the client to store a new context:

```sh
# Using cargo
cargo run --bin mcp-client -- store --content "This is a test context for the Model Context Protocol" --tags "test,example"

# Or using the installed binary
mcp-client store --content "This is a test context for the Model Context Protocol" --tags "test,example"
```

The client will output the ID of the stored context:

```
Storing new context...
Context stored successfully!
ID: a1b2c3d4-...
Content: This is a test context for the Model Context Protocol
Tags: ["test", "example"]
Created at: 2024-02-25T12:35:00Z
```

Make note of the ID for future operations.

## 3. List All Contexts

```sh
cargo run --bin mcp-client -- list

# Or
mcp-client list
```

Output:

```
Listing contexts...
Found 1 contexts:

--- Context 1 ---
ID: a1b2c3d4-...
Content: This is a test context for the Model Context Protocol
Tags: ["test", "example"]
```

## 4. Search for Contexts

```sh
cargo run --bin mcp-client -- search --query "protocol"

# Or
mcp-client search --query "protocol"
```

Output:

```
Searching for contexts with query: "protocol"...
Found 1 matches (out of 1 total):

--- Match 1 (score: 1.00) ---
ID: a1b2c3d4-...
Content: This is a test context for the Model Context Protocol
Tags: ["test", "example"]
Matching chunks: 1
  - This is a test context for the Model Context Protocol
```

## 5. Update a Context

```sh
cargo run --bin mcp-client -- update --id "a1b2c3d4-..." --content "This is an updated test context for MCP" --tags "test,updated"

# Or
mcp-client update --id "a1b2c3d4-..." --content "This is an updated test context for MCP" --tags "test,updated"
```

Output:

```
Updating context with ID: a1b2c3d4-...
Context updated successfully!
ID: a1b2c3d4-...
New content: This is an updated test context for MCP
Tags: ["test", "updated"]
```

## 6. Delete a Context

```sh
cargo run --bin mcp-client -- delete --id "a1b2c3d4-..."

# Or
mcp-client delete --id "a1b2c3d4-..."
```

Output:

```
Deleting context with ID: a1b2c3d4-...
Context deleted successfully!
```

## 7. Interactive Mode

For a more user-friendly experience, try the interactive mode:

```sh
cargo run --bin mcp-client -- interactive

# Or
mcp-client interactive
```

This will provide a step-by-step interface to guide you through the various operations.

## Notes

- The server stores all contexts in memory by default, so they will be lost when the server is restarted
- For production usage, implement a persistent storage adapter
- The chunking and embedding functionality uses a simplified implementation for demonstration purposes