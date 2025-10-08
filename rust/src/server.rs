/*!
 * HTTP server implementation
 */

use crate::adapter::{create_adapter, ApiServiceAdapter};
use crate::common::*;
use crate::config::Config;
use anyhow::Result;
use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response, Sse},
    response::sse::Event,
    routing::{get, post},
    Json, Router,
};
use futures::StreamExt;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};

/// Application state
pub struct AppState {
    pub config: Config,
    pub adapter: Box<dyn ApiServiceAdapter>,
}

/// Start the HTTP server
pub async fn start_server(config: Config) -> Result<()> {
    let host = config.host.clone();
    let port = config.port;
    let addr = format!("{}:{}", host, port);

    // Create adapter
    let provider = ModelProvider::from_str(&config.model_provider)
        .ok_or_else(|| anyhow::anyhow!("Invalid model provider: {}", config.model_provider))?;
    let adapter = create_adapter(provider, &config).await?;

    // Create application state
    let state = Arc::new(AppState { 
        config: config.clone(),
        adapter,
    });
    let state_clone = state.clone();

    // Build CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers(Any);

    // Build application router
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/v1/chat/completions", post(openai_chat_handler))
        .route("/v1/models", get(openai_models_handler))
        .route("/v1/messages", post(claude_messages_handler))
        .route("/v1beta/models", get(gemini_models_handler))
        .route(
            "/v1beta/models/:model/:action",
            post(gemini_content_handler),
        )
        .route("/:provider/v1/chat/completions", post(openai_chat_handler))
        .route("/:provider/v1/models", get(openai_models_handler))
        .route("/:provider/v1/messages", post(claude_messages_handler))
        .with_state(state)
        .layer(cors);

    // Create TCP listener
    let listener = TcpListener::bind(&addr).await?;

    info!("--- Unified API Server Configuration ---");
    info!("  Host: {}", host);
    info!("  Port: {}", port);
    info!("  Primary Model Provider: {}", state_clone.config.model_provider);
    info!("------------------------------------------");
    info!("\nUnified API Server running on http://{}", addr);
    info!("Supports multiple API formats:");
    info!("  • OpenAI-compatible: /v1/chat/completions, /v1/models");
    info!("  • Gemini-compatible: /v1beta/models, /v1beta/models/{{model}}:generateContent");
    info!("  • Claude-compatible: /v1/messages");
    info!("  • Health check: /health");

    // Start serving
    axum::serve(listener, app).await?;

    Ok(())
}

/// Health check handler
async fn health_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "provider": state.config.model_provider
    }))
}

/// OpenAI chat completions handler
async fn openai_chat_handler(
    State(state): State<Arc<AppState>>,
    provider_path: Option<Path<String>>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
    Json(body): Json<Value>,
) -> Result<Response, AppError> {
    // Check authorization
    let auth_header = headers.get("authorization").and_then(|v| v.to_str().ok());
    let api_key_header = headers.get("x-api-key").and_then(|v| v.to_str().ok());
    let goog_api_key = headers.get("x-goog-api-key").and_then(|v| v.to_str().ok());
    let query_key = params.get("key").map(|s| s.as_str());

    if !is_authorized(
        auth_header,
        api_key_header,
        goog_api_key,
        query_key,
        &state.config.required_api_key,
    ) {
        return Err(AppError::Unauthorized);
    }

    info!("Received OpenAI chat request");

    // TODO: Implement actual request handling
    Ok(Json(json!({
        "id": "chatcmpl-123",
        "object": "chat.completion",
        "created": chrono::Utc::now().timestamp(),
        "model": "test-model",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "This is a placeholder response from Rust implementation"
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 0,
            "completion_tokens": 0,
            "total_tokens": 0
        }
    }))
    .into_response())
}

/// OpenAI models list handler
async fn openai_models_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Response, AppError> {
    // Check authorization
    let auth_header = headers.get("authorization").and_then(|v| v.to_str().ok());
    let api_key_header = headers.get("x-api-key").and_then(|v| v.to_str().ok());
    let goog_api_key = headers.get("x-goog-api-key").and_then(|v| v.to_str().ok());
    let query_key = params.get("key").map(|s| s.as_str());

    if !is_authorized(
        auth_header,
        api_key_header,
        goog_api_key,
        query_key,
        &state.config.required_api_key,
    ) {
        return Err(AppError::Unauthorized);
    }

    info!("Received OpenAI models list request");

    // TODO: Implement actual model listing
    Ok(Json(json!({
        "object": "list",
        "data": []
    }))
    .into_response())
}

/// Claude messages handler
async fn claude_messages_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
    Json(body): Json<Value>,
) -> Result<Response, AppError> {
    // Check authorization
    let auth_header = headers.get("authorization").and_then(|v| v.to_str().ok());
    let api_key_header = headers.get("x-api-key").and_then(|v| v.to_str().ok());
    let goog_api_key = headers.get("x-goog-api-key").and_then(|v| v.to_str().ok());
    let query_key = params.get("key").map(|s| s.as_str());

    if !is_authorized(
        auth_header,
        api_key_header,
        goog_api_key,
        query_key,
        &state.config.required_api_key,
    ) {
        return Err(AppError::Unauthorized);
    }

    info!("Received Claude messages request");

    // Extract model from request  
    let model = body.get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("claude-3-5-sonnet-20241022")
        .to_string();

    // Check if streaming is requested
    let stream = body.get("stream")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if stream {
        // Handle streaming response
        info!("Streaming response requested for Claude messages");
        
        match state.adapter.generate_content_stream(&model, body).await {
            Ok(stream) => {
                // Convert the stream to SSE format
                // Claude API uses simple SSE format with only 'data:' lines
                let sse_stream = stream.map(|result| {
                    match result {
                        Ok(chunk) => {
                            // Format as SSE event with event type based on chunk type
                            let data = serde_json::to_string(&chunk).unwrap_or_default();
                            let event_type = chunk.get("type").and_then(|t| t.as_str()).unwrap_or("message");
                            Ok::<_, Infallible>(Event::default().event(event_type).data(data))
                        }
                        Err(e) => {
                            error!("Stream error: {}", e);
                            // Send error event
                            let error_data = json!({
                                "type": "error",
                                "error": {
                                    "message": e.to_string()
                                }
                            });
                            Ok(Event::default().event("error").data(serde_json::to_string(&error_data).unwrap_or_default()))
                        }
                    }
                });
                
                Ok(Sse::new(sse_stream).into_response())
            }
            Err(e) => {
                error!("Failed to start streaming: {}", e);
                Err(AppError::InternalError(e))
            }
        }
    } else {
        // Handle non-streaming response
        match state.adapter.generate_content(&model, body).await {
            Ok(response) => {
                info!("Claude messages request completed successfully");
                Ok(Json(response).into_response())
            }
            Err(e) => {
                error!("Claude messages request failed: {}", e);
                Err(AppError::InternalError(e))
            }
        }
    }
}

/// Gemini models list handler
async fn gemini_models_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Response, AppError> {
    // Check authorization
    let auth_header = headers.get("authorization").and_then(|v| v.to_str().ok());
    let api_key_header = headers.get("x-api-key").and_then(|v| v.to_str().ok());
    let goog_api_key = headers.get("x-goog-api-key").and_then(|v| v.to_str().ok());
    let query_key = params.get("key").map(|s| s.as_str());

    if !is_authorized(
        auth_header,
        api_key_header,
        goog_api_key,
        query_key,
        &state.config.required_api_key,
    ) {
        return Err(AppError::Unauthorized);
    }

    info!("Received Gemini models list request");

    // TODO: Implement actual model listing
    Ok(Json(json!({
        "models": []
    }))
    .into_response())
}

/// Gemini content generation handler
async fn gemini_content_handler(
    State(state): State<Arc<AppState>>,
    Path((model, action)): Path<(String, String)>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
    Json(body): Json<Value>,
) -> Result<Response, AppError> {
    // Check authorization
    let auth_header = headers.get("authorization").and_then(|v| v.to_str().ok());
    let api_key_header = headers.get("x-api-key").and_then(|v| v.to_str().ok());
    let goog_api_key = headers.get("x-goog-api-key").and_then(|v| v.to_str().ok());
    let query_key = params.get("key").map(|s| s.as_str());

    if !is_authorized(
        auth_header,
        api_key_header,
        goog_api_key,
        query_key,
        &state.config.required_api_key,
    ) {
        return Err(AppError::Unauthorized);
    }

    info!("Received Gemini content request for model: {}, action: {}", model, action);

    // TODO: Implement actual request handling
    Ok(Json(json!({
        "candidates": [{
            "content": {
                "parts": [{
                    "text": "This is a placeholder response from Rust implementation"
                }],
                "role": "model"
            },
            "finishReason": "STOP"
        }]
    }))
    .into_response())
}

/// Application error type
#[derive(Debug)]
pub enum AppError {
    Unauthorized,
    BadRequest(String),
    InternalError(anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "Unauthorized: API key is invalid or missing.".to_string(),
            ),
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            Self::InternalError(e) => {
                error!("Internal error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
        };

        (status, Json(json!({ "error": { "message": message } }))).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        Self::InternalError(err)
    }
}

