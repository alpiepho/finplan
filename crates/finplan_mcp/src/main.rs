use anyhow::Result;
use rmcp::{ServiceExt, transport::stdio};
use tracing_subscriber::{self, EnvFilter};

mod defaults;
mod prompts;
mod resources;
mod schema_text;
mod server;
mod state;
mod tools;

use server::FinplanMcpServer;

#[tokio::main]
async fn main() -> Result<()> {
    // Log to stderr so stdout is reserved for MCP JSON-RPC
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting FinPlan MCP server");

    let server = FinplanMcpServer::new();
    let service = server.serve(stdio()).await.inspect_err(|e| {
        tracing::error!("Server error: {:?}", e);
    })?;

    service.waiting().await?;
    Ok(())
}
