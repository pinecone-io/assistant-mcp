use assistant_mcp::config::Config;
use assistant_mcp::router::PineconeAssistantRouter;
use is_terminal::IsTerminal;
use mcp_server::router::RouterService;
use mcp_server::{ByteTransport, Server, ServerError};
use thiserror::Error;
use tokio::io::{stdin, stdout};
use tracing_subscriber::EnvFilter;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("MCP server error: {0}")]
    Server(#[from] ServerError),
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            "info,assistant_mcp=debug"
                .parse()
                .expect("Invalid default filter")
        }))
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(std::io::stderr().is_terminal())
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("Starting Pinecone MCP server");

    let config = Config::from_env();
    tracing::info!("Configuration loaded successfully");

    let router = RouterService(PineconeAssistantRouter::new(config));
    let server = Server::new(router);
    let transport = ByteTransport::new(stdin(), stdout());

    tracing::info!("Server initialized and ready to handle requests");
    server.run(transport).await.map_err(AppError::from)
}
