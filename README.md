# Pinecone Assistant MCP Server

An MCP server implementation for retrieving information from Pinecone Assistant.

## Features

- Retrieves information from Pinecone Assistant
- Supports multiple results retrieval with a configurable number of results

## Prerequisites

- Docker installed on your system
- Pinecone API key - obtain from the [Pinecone Console](https://app.pinecone.io)
- Pinecone Assistant API host - after creating an Assistant (e.g. in Pinecone Console), you can find the host in the Assistant details page

## Building with Docker

To build the Docker image:

```sh
docker build -t pinecone/assistant-mcp .
```

## Running with Docker

Run the server with your Pinecone API key:

```sh
docker run -i --rm \
  -e PINECONE_API_KEY=<YOUR_PINECONE_API_KEY_HERE> \
  -e PINECONE_ASSISTANT_HOST=<YOUR_PINECONE_ASSISTANT_HOST_HERE> \
  pinecone/assistant-mcp
```

### Environment Variables

- `PINECONE_API_KEY` (required): Your Pinecone API key
- `PINECONE_ASSISTANT_HOST` (optional): Pinecone Assistant API host (default: https://prod-1-data.ke.pinecone.io)
- `LOG_LEVEL` (optional): Logging level (default: info)

## Usage with Claude Desktop

Add this to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "pinecone-assistant": {
      "command": "docker",
      "args": [
        "run", 
        "-i", 
        "--rm", 
        "-e", 
        "PINECONE_API_KEY", 
        "-e", 
        "PINECONE_ASSISTANT_HOST", 
        "pinecone/assistant-mcp"
      ],
      "env": {
        "PINECONE_API_KEY": "<YOUR_PINECONE_API_KEY_HERE>",
        "PINECONE_ASSISTANT_HOST": "<YOUR_PINECONE_ASSISTANT_HOST_HERE>"
      }
    }
  }
}
```

## Building from Source

If you prefer to build from source without Docker:

1. Make sure you have Rust installed (https://rustup.rs/)
2. Clone this repository
3. Run `cargo build --release`
4. The binary will be available at `target/release/assistant-mcp`

### Testing with the inspector
```sh
export PINECONE_API_KEY=<YOUR_PINECONE_API_KEY_HERE>
export PINECONE_ASSISTANT_HOST=<YOUR_PINECONE_ASSISTANT_HOST_HERE>
# Run the inspector alone
npx @modelcontextprotocol/inspector cargo run
# Or run with Docker directly through the inspector
npx @modelcontextprotocol/inspector -- docker run -i --rm -e PINECONE_API_KEY -e PINECONE_ASSISTANT_HOST pinecone/assistant-mcp
```

## License

This project is licensed under the terms specified in the LICENSE file.
