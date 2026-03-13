use axum::{
    extract::State,
    response::{sse::Event, Sse},
    routing::{get, post},
    Json, Router,
};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};

pub mod mcp_tools;
pub mod mcp_resources;
pub mod mcp_prompts;

use crate::app::config::McpConfig;

#[derive(Clone)]
pub struct McpState {
    pub app_handle: tauri::AppHandle,
    pub tools: Arc<RwLock<HashMap<String, mcp_tools::Tool>>>,
    pub resources: Arc<RwLock<HashMap<String, mcp_resources::Resource>>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    jsonrpc: String,
    #[serde(default)]
    id: Option<serde_json::Value>,
    method: String,
    #[serde(default)]
    params: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    code: i32,
    message: String,
}

pub async fn start_mcp_server(app_handle: tauri::AppHandle, config: McpConfig) {
    if !config.enabled {
        println!("MCP Server is disabled");
        return;
    }

    let state = McpState {
        app_handle,
        tools: Arc::new(RwLock::new(mcp_tools::register_tools())),
        resources: Arc::new(RwLock::new(mcp_resources::register_resources())),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/sse", get(sse_handler))
        .route("/message", post(message_handler))
        .route("/health", get(health_handler))
        .layer(cors)
        .with_state(state);

    let addr = format!("{}:{}", config.host, config.port);
    
    match tokio::net::TcpListener::bind(&addr).await {
        Ok(listener) => {
            println!("MCP Server started at http://{}", addr);
            if let Err(e) = axum::serve(listener, app).await {
                eprintln!("MCP Server error: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Failed to bind MCP server to {}: {}", addr, e);
        }
    }
}

async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "version": "1.0.0"
    }))
}

async fn sse_handler(
    State(_state): State<McpState>,
) -> Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>> {
    let stream = futures::stream::once(async move {
        Ok(Event::default().data("connected"))
    });
    Sse::new(stream)
}

async fn message_handler(
    State(state): State<McpState>,
    Json(request): Json<JsonRpcRequest>,
) -> Json<JsonRpcResponse> {
    let response = handle_jsonrpc(request, state).await;
    Json(response)
}

async fn handle_jsonrpc(request: JsonRpcRequest, state: McpState) -> JsonRpcResponse {
    match request.method.as_str() {
        "initialize" => handle_initialize(&request),
        "tools/list" => handle_tools_list(&request, state).await,
        "tools/call" => handle_tools_call(&request, state).await,
        "resources/list" => handle_resources_list(&request, state).await,
        "resources/read" => handle_resources_read(&request, state).await,
        "prompts/list" => handle_prompts_list(&request),
        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: "Method not found".to_string(),
            }),
        },
    }
}

fn handle_initialize(request: &JsonRpcRequest) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id.clone(),
        result: Some(serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {},
                "resources": {},
                "prompts": {}
            },
            "serverInfo": {
                "name": "copilot-browser-mcp",
                "version": "1.0.0"
            }
        })),
        error: None,
    }
}

async fn handle_tools_list(request: &JsonRpcRequest, state: McpState) -> JsonRpcResponse {
    let tools = state.tools.read().await;
    let tool_list: Vec<serde_json::Value> = tools
        .values()
        .map(|t| serde_json::to_value(t).unwrap())
        .collect();
    
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id.clone(),
        result: Some(serde_json::json!({
            "tools": tool_list
        })),
        error: None,
    }
}

async fn handle_tools_call(request: &JsonRpcRequest, state: McpState) -> JsonRpcResponse {
    let params = request.params.as_object();
    let tool_name = params.and_then(|p| p.get("name")?.as_str());
    let arguments = params.and_then(|p| p.get("arguments"));

    if let Some(name) = tool_name {
        let tools = state.tools.read().await;
        if let Some(tool) = tools.get(name) {
            match (tool.handler)(arguments, &state.app_handle).await {
                Ok(result) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id.clone(),
                    result: Some(result),
                    error: None,
                },
                Err(e) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id.clone(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32000,
                        message: e,
                    }),
                },
            }
        } else {
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.clone(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: format!("Tool not found: {}", name),
                }),
            }
        }
    } else {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id.clone(),
            result: None,
            error: Some(JsonRpcError {
                code: -32602,
                message: "Missing tool name".to_string(),
            }),
        }
    }
}

async fn handle_resources_list(request: &JsonRpcRequest, state: McpState) -> JsonRpcResponse {
    let resources = state.resources.read().await;
    let resource_list: Vec<serde_json::Value> = resources
        .values()
        .map(|r| serde_json::to_value(r).unwrap())
        .collect();
    
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id.clone(),
        result: Some(serde_json::json!({
            "resources": resource_list
        })),
        error: None,
    }
}

async fn handle_resources_read(request: &JsonRpcRequest, state: McpState) -> JsonRpcResponse {
    let params = request.params.as_object();
    let uri = params.and_then(|p| p.get("uri")?.as_str());

    if let Some(uri_str) = uri {
        let resources = state.resources.read().await;
        if let Some(resource) = resources.get(uri_str) {
            match (resource.handler)(&state.app_handle).await {
                Ok(content) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id.clone(),
                    result: Some(serde_json::json!({
                        "contents": [{
                            "uri": uri_str,
                            "mimeType": resource.mime_type,
                            "text": content
                        }]
                    })),
                    error: None,
                },
                Err(e) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id.clone(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32000,
                        message: e,
                    }),
                },
            }
        } else {
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.clone(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: format!("Resource not found: {}", uri_str),
                }),
            }
        }
    } else {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id.clone(),
            result: None,
            error: Some(JsonRpcError {
                code: -32602,
                message: "Missing resource URI".to_string(),
            }),
        }
    }
}

fn handle_prompts_list(request: &JsonRpcRequest) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id.clone(),
        result: Some(serde_json::json!({
            "prompts": mcp_prompts::get_prompts()
        })),
        error: None,
    }
}
