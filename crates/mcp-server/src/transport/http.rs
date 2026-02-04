//! HTTP/SSE transport for MCP

use axum::{
    extract::State,
    http::StatusCode,
    response::sse::{Event, Sse},
    routing::{get, post},
    Json, Router,
};
use futures::stream::Stream;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tracing::{debug, info};

use crate::protocol::{McpMessage, RequestHandler};
use wallet_core::Wallet;

/// Shared state for HTTP handlers
struct AppState {
    handler: RwLock<RequestHandler>,
}

/// HTTP transport for MCP protocol
pub struct HttpTransport {
    wallet: Arc<RwLock<Wallet>>,
    port: u16,
}

impl HttpTransport {
    /// Create a new HTTP transport
    pub fn new(wallet: Arc<RwLock<Wallet>>, port: u16) -> Self {
        Self { wallet, port }
    }

    /// Run the HTTP server
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let state = Arc::new(AppState {
            handler: RwLock::new(RequestHandler::new(self.wallet.clone())),
        });

        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        let app = Router::new()
            .route("/", get(health))
            .route("/health", get(health))
            .route("/mcp", post(handle_mcp_request))
            .route("/mcp/sse", get(handle_mcp_sse))
            .layer(cors)
            .with_state(state);

        let addr = format!("0.0.0.0:{}", self.port);
        info!("Starting MCP HTTP server on {}", addr);

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

/// Health check endpoint
async fn health() -> &'static str {
    "OK"
}

/// Handle MCP JSON-RPC request via HTTP POST
async fn handle_mcp_request(
    State(state): State<Arc<AppState>>,
    Json(message): Json<McpMessage>,
) -> Result<Json<McpMessage>, (StatusCode, String)> {
    debug!("HTTP request: {:?}", message);

    let mut handler = state.handler.write().await;

    match handler.handle(message).await {
        Some(response) => Ok(Json(response)),
        None => {
            // Notification - return empty success
            Ok(Json(McpMessage::response(
                serde_json::json!(null),
                serde_json::json!({}),
            )))
        }
    }
}

/// Handle MCP via Server-Sent Events
async fn handle_mcp_sse(
    State(_state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    info!("SSE connection established");

    // For now, just send a ready event
    // In a full implementation, this would maintain a bidirectional connection
    let stream = async_stream::stream! {
        yield Ok(Event::default().data(r#"{"status":"ready"}"#));
    };

    Sse::new(stream)
}
