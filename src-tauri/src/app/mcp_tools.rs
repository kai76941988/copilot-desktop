use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;
use std::future::Future;
use tauri::Manager;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolInputSchema {
    #[serde(rename = "type")]
    pub type_name: String,
    pub properties: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub required: Vec<String>,
}

#[derive(Clone)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: ToolInputSchema,
    pub handler: fn(Option<&serde_json::Value>, &tauri::AppHandle) -> Pin<Box<dyn Future<Output = Result<serde_json::Value, String>> + Send + '_>>,
}

impl Serialize for Tool {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("Tool", 3)?;
        s.serialize_field("name", &self.name)?;
        s.serialize_field("description", &self.description)?;
        s.serialize_field("inputSchema", &self.input_schema)?;
        s.end()
    }
}

pub fn register_tools() -> HashMap<String, Tool> {
    let mut tools = HashMap::new();

    // 浏览器导航
    tools.insert("browser_navigate".to_string(), Tool {
        name: "browser_navigate".to_string(),
        description: "Navigate to a specified URL".to_string(),
        input_schema: ToolInputSchema {
            type_name: "object".to_string(),
            properties: {
                let mut props = HashMap::new();
                props.insert("url".to_string(), serde_json::json!({
                    "type": "string",
                    "description": "The URL to navigate to"
                }));
                props
            },
            required: vec!["url".to_string()],
        },
        handler: |args, app| Box::pin(async move {
            let url = args
                .and_then(|a| a.get("url")?.as_str())
                .ok_or("Missing url parameter")?;
            
            if let Some(window) = app.get_webview_window("pake") {
                let script = format!(r#"window.location.href = "{}""#, url);
                window.eval(&script)
                    .map_err(|e| format!("Failed to navigate: {}", e))?;
                Ok(serde_json::json!({ "success": true, "url": url }))
            } else {
                Err("Window not found".to_string())
            }
        }),
    });

    // 获取当前 URL
    tools.insert("browser_get_url".to_string(), Tool {
        name: "browser_get_url".to_string(),
        description: "Get the current page URL".to_string(),
        input_schema: ToolInputSchema {
            type_name: "object".to_string(),
            properties: HashMap::new(),
            required: vec![],
        },
        handler: |_args, app| Box::pin(async move {
            if let Some(window) = app.get_webview_window("pake") {
                let url = window.url()
                    .map(|u| u.to_string())
                    .unwrap_or_default();
                Ok(serde_json::json!({ "url": url }))
            } else {
                Err("Window not found".to_string())
            }
        }),
    });

    // 获取页面标题
    tools.insert("browser_get_title".to_string(), Tool {
        name: "browser_get_title".to_string(),
        description: "Get the current page title".to_string(),
        input_schema: ToolInputSchema {
            type_name: "object".to_string(),
            properties: HashMap::new(),
            required: vec![],
        },
        handler: |_args, app| Box::pin(async move {
            if let Some(window) = app.get_webview_window("pake") {
                let title = window.title()
                    .unwrap_or_default();
                Ok(serde_json::json!({ "title": title }))
            } else {
                Err("Window not found".to_string())
            }
        }),
    });

    // 截图
    tools.insert("browser_screenshot".to_string(), Tool {
        name: "browser_screenshot".to_string(),
        description: "Take a screenshot of the current page".to_string(),
        input_schema: ToolInputSchema {
            type_name: "object".to_string(),
            properties: {
                let mut props = HashMap::new();
                props.insert("selector".to_string(), serde_json::json!({
                    "type": "string",
                    "description": "CSS selector for specific element (optional)"
                }));
                props
            },
            required: vec![],
        },
        handler: |_args, app| Box::pin(async move {
            if let Some(window) = app.get_webview_window("pake") {
                #[cfg(target_os = "windows")]
                {
                    use tauri::WebviewWindow;
                    let image = window.capture_image()
                        .map_err(|e| format!("Failed to capture: {}", e))?;
                    
                    let bytes = image.as_bytes();
                    let base64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes);
                    
                    Ok(serde_json::json!({
                        "success": true,
                        "image": base64,
                        "mimeType": "image/png"
                    }))
                }
                #[cfg(not(target_os = "windows"))]
                {
                    Ok(serde_json::json!({
                        "success": false,
                        "error": "Screenshot only supported on Windows"
                    }))
                }
            } else {
                Err("Window not found".to_string())
            }
        }),
    });

    // 执行 JavaScript
    tools.insert("browser_evaluate".to_string(), Tool {
        name: "browser_evaluate".to_string(),
        description: "Execute JavaScript code in the page".to_string(),
        input_schema: ToolInputSchema {
            type_name: "object".to_string(),
            properties: {
                let mut props = HashMap::new();
                props.insert("script".to_string(), serde_json::json!({
                    "type": "string",
                    "description": "JavaScript code to execute"
                }));
                props
            },
            required: vec!["script".to_string()],
        },
        handler: |args, app| Box::pin(async move {
            let script = args
                .and_then(|a| a.get("script")?.as_str())
                .ok_or("Missing script parameter")?;
            
            if let Some(window) = app.get_webview_window("pake") {
                let result = window.eval(script)
                    .map_err(|e| format!("Script execution failed: {}", e))?;
                Ok(serde_json::json!({ "success": true, "result": result }))
            } else {
                Err("Window not found".to_string())
            }
        }),
    });

    // 点击元素
    tools.insert("browser_click".to_string(), Tool {
        name: "browser_click".to_string(),
        description: "Click an element on the page".to_string(),
        input_schema: ToolInputSchema {
            type_name: "object".to_string(),
            properties: {
                let mut props = HashMap::new();
                props.insert("selector".to_string(), serde_json::json!({
                    "type": "string",
                    "description": "CSS selector for the element to click"
                }));
                props
            },
            required: vec!["selector".to_string()],
        },
        handler: |args, app| Box::pin(async move {
            let selector = args
                .and_then(|a| a.get("selector")?.as_str())
                .ok_or("Missing selector parameter")?;
            
            if let Some(window) = app.get_webview_window("pake") {
                let script = format!(
                    r#"(function() {{
                        const el = document.querySelector("{}");
                        if (el) {{ el.click(); return true; }}
                        return false;
                    }})()"#,
                    selector
                );
                let result = window.eval(&script)
                    .map_err(|e| format!("Click failed: {}", e))?;
                Ok(serde_json::json!({ "success": result == "true" }))
            } else {
                Err("Window not found".to_string())
            }
        }),
    });

    // 输入文本
    tools.insert("browser_type".to_string(), Tool {
        name: "browser_type".to_string(),
        description: "Type text into an input element".to_string(),
        input_schema: ToolInputSchema {
            type_name: "object".to_string(),
            properties: {
                let mut props = HashMap::new();
                props.insert("selector".to_string(), serde_json::json!({
                    "type": "string",
                    "description": "CSS selector for the input element"
                }));
                props.insert("text".to_string(), serde_json::json!({
                    "type": "string",
                    "description": "Text to type"
                }));
                props
            },
            required: vec!["selector".to_string(), "text".to_string()],
        },
        handler: |args, app| Box::pin(async move {
            let selector = args
                .and_then(|a| a.get("selector")?.as_str())
                .ok_or("Missing selector parameter")?;
            let text = args
                .and_then(|a| a.get("text")?.as_str())
                .ok_or("Missing text parameter")?;
            
            // Escape quotes in text
            let escaped_text = text.replace("\\", "\\\\").replace("\"", "\\\"");
            
            if let Some(window) = app.get_webview_window("pake") {
                let script = format!(
                    r#"(function() {{
                        const el = document.querySelector("{}");
                        if (el) {{
                            el.value = "{}";
                            el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                            return true;
                        }}
                        return false;
                    }})()"#,
                    selector, escaped_text
                );
                let result = window.eval(&script)
                    .map_err(|e| format!("Type failed: {}", e))?;
                Ok(serde_json::json!({ "success": result == "true" }))
            } else {
                Err("Window not found".to_string())
            }
        }),
    });

    // 滚动
    tools.insert("browser_scroll".to_string(), Tool {
        name: "browser_scroll".to_string(),
        description: "Scroll the page".to_string(),
        input_schema: ToolInputSchema {
            type_name: "object".to_string(),
            properties: {
                let mut props = HashMap::new();
                props.insert("direction".to_string(), serde_json::json!({
                    "type": "string",
                    "enum": ["up", "down", "top", "bottom"],
                    "description": "Scroll direction"
                }));
                props.insert("amount".to_string(), serde_json::json!({
                    "type": "number",
                    "description": "Scroll amount in pixels (optional)"
                }));
                props
            },
            required: vec!["direction".to_string()],
        },
        handler: |args, app| Box::pin(async move {
            let direction = args
                .and_then(|a| a.get("direction")?.as_str())
                .ok_or("Missing direction parameter")?;
            let amount = args
                .and_then(|a| a.get("amount")?.as_u64())
                .unwrap_or(300) as i32;
            
            if let Some(window) = app.get_webview_window("pake") {
                let script = match direction {
                    "up" => format!("window.scrollBy(0, -{})", amount),
                    "down" => format!("window.scrollBy(0, {})", amount),
                    "top" => "window.scrollTo(0, 0)".to_string(),
                    "bottom" => "window.scrollTo(0, document.body.scrollHeight)".to_string(),
                    _ => return Err("Invalid direction".to_string()),
                };
                window.eval(&script)
                    .map_err(|e| format!("Scroll failed: {}", e))?;
                Ok(serde_json::json!({ "success": true }))
            } else {
                Err("Window not found".to_string())
            }
        }),
    });

    // 后退
    tools.insert("browser_go_back".to_string(), Tool {
        name: "browser_go_back".to_string(),
        description: "Go back to the previous page".to_string(),
        input_schema: ToolInputSchema {
            type_name: "object".to_string(),
            properties: HashMap::new(),
            required: vec![],
        },
        handler: |_args, app| Box::pin(async move {
            if let Some(window) = app.get_webview_window("pake") {
                window.eval("window.history.back()")
                    .map_err(|e| format!("Go back failed: {}", e))?;
                Ok(serde_json::json!({ "success": true }))
            } else {
                Err("Window not found".to_string())
            }
        }),
    });

    // 前进
    tools.insert("browser_go_forward".to_string(), Tool {
        name: "browser_go_forward".to_string(),
        description: "Go forward to the next page".to_string(),
        input_schema: ToolInputSchema {
            type_name: "object".to_string(),
            properties: HashMap::new(),
            required: vec![],
        },
        handler: |_args, app| Box::pin(async move {
            if let Some(window) = app.get_webview_window("pake") {
                window.eval("window.history.forward()")
                    .map_err(|e| format!("Go forward failed: {}", e))?;
                Ok(serde_json::json!({ "success": true }))
            } else {
                Err("Window not found".to_string())
            }
        }),
    });

    // 刷新
    tools.insert("browser_refresh".to_string(), Tool {
        name: "browser_refresh".to_string(),
        description: "Refresh the current page".to_string(),
        input_schema: ToolInputSchema {
            type_name: "object".to_string(),
            properties: HashMap::new(),
            required: vec![],
        },
        handler: |_args, app| Box::pin(async move {
            if let Some(window) = app.get_webview_window("pake") {
                window.eval("window.location.reload()")
                    .map_err(|e| format!("Refresh failed: {}", e))?;
                Ok(serde_json::json!({ "success": true }))
            } else {
                Err("Window not found".to_string())
            }
        }),
    });

    // 获取页面内容
    tools.insert("browser_get_content".to_string(), Tool {
        name: "browser_get_content".to_string(),
        description: "Get the page content (HTML)".to_string(),
        input_schema: ToolInputSchema {
            type_name: "object".to_string(),
            properties: {
                let mut props = HashMap::new();
                props.insert("selector".to_string(), serde_json::json!({
                    "type": "string",
                    "description": "CSS selector for specific element (optional)"
                }));
                props
            },
            required: vec![],
        },
        handler: |args, app| Box::pin(async move {
            let selector = args.and_then(|a| a.get("selector")?.as_str());
            
            if let Some(window) = app.get_webview_window("pake") {
                let script = if let Some(sel) = selector {
                    format!(r#"(function() {{
                        const el = document.querySelector("{}");
                        return el ? el.outerHTML : null;
                    }})()"#, sel)
                } else {
                    "document.documentElement.outerHTML".to_string()
                };
                let content = window.eval(&script)
                    .map_err(|e| format!("Get content failed: {}", e))?;
                Ok(serde_json::json!({ "content": content }))
            } else {
                Err("Window not found".to_string())
            }
        }),
    });

    // 等待元素
    tools.insert("browser_wait_for".to_string(), Tool {
        name: "browser_wait_for".to_string(),
        description: "Wait for an element to appear on the page".to_string(),
        input_schema: ToolInputSchema {
            type_name: "object".to_string(),
            properties: {
                let mut props = HashMap::new();
                props.insert("selector".to_string(), serde_json::json!({
                    "type": "string",
                    "description": "CSS selector for the element to wait for"
                }));
                props.insert("timeout".to_string(), serde_json::json!({
                    "type": "number",
                    "description": "Timeout in milliseconds (default 5000)"
                }));
                props
            },
            required: vec!["selector".to_string()],
        },
        handler: |args, app| Box::pin(async move {
            let selector = args
                .and_then(|a| a.get("selector")?.as_str())
                .ok_or("Missing selector parameter")?;
            let timeout = args
                .and_then(|a| a.get("timeout")?.as_u64())
                .unwrap_or(5000);
            
            if let Some(window) = app.get_webview_window("pake") {
                let script = format!(
                    r#"new Promise((resolve, reject) => {{
                        const timeout = setTimeout(() => reject(new Error("Timeout")), {});
                        const check = () => {{
                            const el = document.querySelector("{}");
                            if (el) {{
                                clearTimeout(timeout);
                                resolve(true);
                            }} else {{
                                requestAnimationFrame(check);
                            }}
                        }};
                        check();
                    }})"#,
                    timeout, selector
                );
                window.eval(&script)
                    .map_err(|e| format!("Wait failed: {}", e))?;
                Ok(serde_json::json!({ "success": true }))
            } else {
                Err("Window not found".to_string())
            }
        }),
    });

    // 发送通知
    tools.insert("browser_notification".to_string(), Tool {
        name: "browser_notification".to_string(),
        description: "Send a desktop notification".to_string(),
        input_schema: ToolInputSchema {
            type_name: "object".to_string(),
            properties: {
                let mut props = HashMap::new();
                props.insert("title".to_string(), serde_json::json!({
                    "type": "string",
                    "description": "Notification title"
                }));
                props.insert("body".to_string(), serde_json::json!({
                    "type": "string",
                    "description": "Notification body"
                }));
                props
            },
            required: vec!["title".to_string(), "body".to_string()],
        },
        handler: |args, app| Box::pin(async move {
            use tauri_plugin_notification::NotificationExt;
            
            let title = args
                .and_then(|a| a.get("title")?.as_str())
                .ok_or("Missing title parameter")?;
            let body = args
                .and_then(|a| a.get("body")?.as_str())
                .ok_or("Missing body parameter")?;
            
            app.notification()
                .builder()
                .title(title)
                .body(body)
                .show()
                .map_err(|e| format!("Notification failed: {}", e))?;
            
            Ok(serde_json::json!({ "success": true }))
        }),
    });

    // 显示/隐藏窗口
    tools.insert("browser_show".to_string(), Tool {
        name: "browser_show".to_string(),
        description: "Show the browser window".to_string(),
        input_schema: ToolInputSchema {
            type_name: "object".to_string(),
            properties: HashMap::new(),
            required: vec![],
        },
        handler: |_args, app| Box::pin(async move {
            if let Some(window) = app.get_webview_window("pake") {
                window.show()
                    .map_err(|e| format!("Show failed: {}", e))?;
                window.set_focus()
                    .map_err(|e| format!("Focus failed: {}", e))?;
                Ok(serde_json::json!({ "success": true }))
            } else {
                Err("Window not found".to_string())
            }
        }),
    });

    tools.insert("browser_hide".to_string(), Tool {
        name: "browser_hide".to_string(),
        description: "Hide the browser window".to_string(),
        input_schema: ToolInputSchema {
            type_name: "object".to_string(),
            properties: HashMap::new(),
            required: vec![],
        },
        handler: |_args, app| Box::pin(async move {
            if let Some(window) = app.get_webview_window("pake") {
                window.hide()
                    .map_err(|e| format!("Hide failed: {}", e))?;
                Ok(serde_json::json!({ "success": true }))
            } else {
                Err("Window not found".to_string())
            }
        }),
    });

    tools
}
