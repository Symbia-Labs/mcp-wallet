//! Symbia Labs MCP Wallet CLI - Standalone MCP server for MCP clients
//!
//! This binary provides the stdio MCP server that can be invoked by MCP clients.
//! It shares the same wallet storage as the Tauri desktop app.
//!
//! Authentication: The CLI uses session tokens created by the desktop app.
//! Simply unlock your wallet in the desktop app, and the CLI will automatically
//! have access for 24 hours (no password needed in config files).

use clap::Parser;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use mcp_server::{McpServer, ServerMode};
use wallet_core::Wallet;

/// Symbia Labs MCP Wallet - Secure API credential manager with MCP protocol support
#[derive(Parser, Debug)]
#[command(name = "symbia-mcp-wallet")]
#[command(author = "Symbia Labs")]
#[command(version = "0.1.0")]
#[command(about = "Symbia Labs MCP Wallet - Secure API credential management via MCP")]
struct Args {
    /// Run in stdio mode (for MCP clients like Claude Desktop)
    #[arg(long)]
    stdio: bool,

    /// Run in HTTP mode with specified port
    #[arg(long)]
    http: bool,

    /// Port for HTTP server (default: 3000)
    #[arg(long, default_value = "3000")]
    port: u16,

    /// Password for wallet unlock (fallback if no session exists)
    /// Prefer using the desktop app to create a session instead.
    #[arg(long, env = "MCP_WALLET_PASSWORD", hide_env_values = true)]
    password: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Initialize logging (only if not in stdio mode to avoid corrupting the protocol)
    if !args.stdio {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::from_default_env()
                    .add_directive(tracing::Level::INFO.into()),
            )
            .init();
    }

    // Create wallet instance (uses shared storage location)
    let mut wallet = Wallet::new().map_err(|e| format!("Failed to initialize wallet: {}", e))?;

    // Check if wallet is initialized
    if wallet.state() == wallet_core::WalletState::NotInitialized {
        if !args.stdio {
            eprintln!("Wallet not initialized. Please run the Symbia Labs MCP Wallet desktop app first to set up your wallet.");
        }
        return Err("Wallet not initialized".into());
    }

    // Try to unlock using session token first (created by desktop app)
    let session_unlock = wallet.unlock_with_session().await;

    if session_unlock.is_ok() {
        if !args.stdio {
            info!("Wallet unlocked via session token");
        }
    } else {
        // Session didn't work - try password fallback
        if let Some(password) = args.password {
            wallet
                .unlock(&password)
                .await
                .map_err(|e| format!("Failed to unlock wallet: {}", e))?;

            if !args.stdio {
                info!("Wallet unlocked via password");
            }
        } else {
            // No session and no password
            if args.stdio {
                return Err(
                    "No valid session. Please unlock the wallet in the desktop app first.".into(),
                );
            } else {
                // Interactive mode - prompt for password
                let password = rpassword::prompt_password("Wallet password: ")?;
                wallet
                    .unlock(&password)
                    .await
                    .map_err(|e| format!("Failed to unlock wallet: {}", e))?;
                info!("Wallet unlocked via password");
            }
        }
    }

    let wallet = Arc::new(RwLock::new(wallet));

    // Determine server mode
    let mode = if args.stdio {
        ServerMode::Stdio
    } else if args.http {
        ServerMode::Http { port: args.port }
    } else {
        // Default to stdio for MCP client compatibility
        ServerMode::Stdio
    };

    // Create and run server
    let server = McpServer::new(wallet).with_mode(mode);

    if !args.stdio {
        match mode {
            ServerMode::Stdio => info!("Starting MCP server in stdio mode"),
            ServerMode::Http { port } => info!("Starting MCP server on http://localhost:{}", port),
        }
    }

    server.run().await?;

    Ok(())
}
