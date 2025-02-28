# assistant-mcp
Pinecone Assistant MCP server
# Pinecone Assistant MCP Server

An MCP server implementation for retrieving information from Pinecone.

## Features

- Connects to Pinecone's Assistant API
- Provides MCP-compatible interface for Claude and other MCP-enabled LLMs

## Prerequisites

- Pinecone API key

### Environment Variables

- `PINECONE_API_KEY` (required): Your Pinecone API key
- `PINECONE_ASSISTANT_HOST` (optional): Pinecone Assistant API host (default: https://prod-1-data.ke.pinecone.io)
- `LOG_LEVEL` (optional): Logging level (default: info)

## Building from Source

1. Make sure you have Rust installed (https://rustup.rs/)
2. Clone this repository
3. Run `cargo build --release`
4. The binary will be available at `target/release/assistant-mcp`

### Test
```sh
export PINECONE_API_KEY=<XXX>
npx @modelcontextprotocol/inspector
```

## License

This project is licensed under the terms specified in the LICENSE file.
