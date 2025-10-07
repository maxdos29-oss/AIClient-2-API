/*!
 * AIClient-2-API Rust Version
 *
 * A powerful proxy that converts multiple AI client APIs (Gemini CLI, Qwen Code Plus, Kiro Claude...)
 * to OpenAI-compatible endpoints.
 *
 * Copyright 2025 AIClient-2-API Contributors
 * License: GPL-3.0
 */

pub mod config;
pub mod server;
pub mod common;
pub mod adapter;
pub mod convert;
pub mod convert_detailed;
pub mod providers;
pub mod pool_manager;
pub mod strategies;
pub mod system_prompt;
pub mod logger;

use anyhow::Result;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "aiclient2api_rust=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting AIClient-2-API Rust Server...");

    // Load configuration
    let config = match config::Config::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    info!("Configuration loaded successfully");
    info!("  Host: {}", config.host);
    info!("  Port: {}", config.port);
    info!("  Model Provider: {}", config.model_provider);

    // Start the server
    if let Err(e) = server::start_server(config).await {
        error!("Server error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}

