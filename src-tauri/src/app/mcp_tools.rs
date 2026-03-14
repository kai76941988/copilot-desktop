use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use tauri::Manager;
use crate::app::mcp_eval::eval_with_result;

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
    pub handler: ToolHandler,
}

type ToolHandler = for<'a> fn(
    Option<&'a serde_json::Value>,
    &'a tauri::AppHandle,
) -> Pin<Box<dyn Future<Output = Result<serde_json::Value, String>> + Send + 'a>>;

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

    tools.insert("browser_get_ui_state".to_string(), Tool {
        name: "browser_get_ui_state".to_string(),
        description: "Get current UI state (URL, title, readyState, and text)".to_string(),
        input_schema: ToolInputSchema {
            type_name: "object".to_string(),
            properties: {
                let mut props = HashMap::new();
                props.insert("selector".to_string(), serde_json::json!({
                    "type": "string",
                    "description": "CSS selector for specific element text (optional)"
                }));
                props.insert("max_len".to_string(), serde_json::json!({
                    "type": "number",
                    "description": "Maximum text length (default 1500)"
                }));
                props
            },
            required: vec![],
        },
        handler: |args, app| Box::pin(async move {
            let selector = args.and_then(|a| a.get("selector")?.as_str());
            let max_len = args
                .and_then(|a| a.get("max_len")?.as_u64())
                .unwrap_or(1500) as usize;
            let selector_js = match selector {
                Some(value) => serde_json::to_string(value).map_err(|e| e.to_string())?,
                None => "null".to_string(),
            };
            let script = format!(
                r#"return (function(){{
  const sel = {selector_js};
  let node = null;
  if (sel) {{
    node = document.querySelector(sel);
  }}
  const text = node ? (node.innerText || "") : (document.body ? (document.body.innerText || "") : "");
  const url = window.location.href;
  const title = document.title;
  const readyState = document.readyState;
  const trimmed = text.length > {max_len} ? text.slice(0, {max_len}) : text;
  return {{ url, title, readyState, text: trimmed }};
}})();"#,
                selector_js = selector_js,
                max_len = max_len
            );
            let state = eval_with_result(app, &script, 10000).await?;
            Ok(serde_json::json!({ "success": true, "state": state }))
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
                let _ = window;
                Ok(serde_json::json!({
                    "success": false,
                    "error": "Screenshot is not supported in this build"
                }))
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
            
            let result = eval_with_result(app, script, 10000).await?;
            Ok(serde_json::json!({ "success": true, "result": result }))
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
                window.eval(&script)
                    .map_err(|e| format!("Click failed: {}", e))?;
                Ok(serde_json::json!({ "success": true }))
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
                window.eval(&script)
                    .map_err(|e| format!("Type failed: {}", e))?;
                Ok(serde_json::json!({ "success": true }))
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

            let selector_js = match selector {
                Some(value) => serde_json::to_string(value).map_err(|e| e.to_string())?,
                None => "null".to_string(),
            };
            let script = format!(
                r#"return (function(){{
  const sel = {selector_js};
  const scrub = (root) => {{
    if (!root) return null;
    const clone = root.cloneNode(true);
    const inputs = clone.querySelectorAll("input, textarea");
    inputs.forEach((el) => {{
      el.setAttribute("value", "");
      if (el.tagName && el.tagName.toLowerCase() === "textarea") {{
        el.textContent = "";
      }}
    }});
    return clone;
  }};
  const node = sel ? document.querySelector(sel) : document.documentElement;
  const cleaned = scrub(node);
  return cleaned ? cleaned.outerHTML : null;
}})();"#,
                selector_js = selector_js
            );
            let content = eval_with_result(app, &script, 10000).await?;
            Ok(serde_json::json!({ "content": content }))
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

            let selector_js = serde_json::to_string(selector).map_err(|e| e.to_string())?;
            let script = format!(
                r#"return await new Promise((resolve) => {{
  const timeout = setTimeout(() => resolve(false), {timeout});
  const check = () => {{
    const el = document.querySelector({selector_js});
    if (el) {{
      clearTimeout(timeout);
      resolve(true);
    }} else {{
      requestAnimationFrame(check);
    }}
  }};
  check();
}});"#,
                timeout = timeout,
                selector_js = selector_js
            );
            let result = eval_with_result(app, &script, timeout + 2000).await?;
            Ok(serde_json::json!({ "success": result }))
        }),
    });

    tools.insert("browser_watch_dom".to_string(), Tool {
        name: "browser_watch_dom".to_string(),
        description: "Watch DOM changes and return recent mutations".to_string(),
        input_schema: ToolInputSchema {
            type_name: "object".to_string(),
            properties: {
                let mut props = HashMap::new();
                props.insert("since".to_string(), serde_json::json!({
                    "type": "number",
                    "description": "Last seen change id (default 0)"
                }));
                props.insert("max".to_string(), serde_json::json!({
                    "type": "number",
                    "description": "Max number of changes to return (default 100)"
                }));
                props
            },
            required: vec![],
        },
        handler: |args, app| Box::pin(async move {
            let since = args
                .and_then(|a| a.get("since")?.as_u64())
                .unwrap_or(0);
            let max = args
                .and_then(|a| a.get("max")?.as_u64())
                .unwrap_or(100);
            let script = format!(
                r#"return (function(){{
  const key = "__MCP_DOM_WATCH__";
  const root = document.documentElement || document.body;
  if (!root) {{
    return {{ lastId: 0, changes: [], url: location.href, title: document.title }};
  }}
  if (!window[key]) {{
    const state = {{ seq: 0, changes: [] }};
    const observer = new MutationObserver((mutations) => {{
      for (const m of mutations) {{
        const entry = {{
          id: ++state.seq,
          type: m.type,
          target: m.target && m.target.nodeName ? m.target.nodeName.toLowerCase() : null,
          attributeName: m.attributeName || null,
          added: m.addedNodes ? m.addedNodes.length : 0,
          removed: m.removedNodes ? m.removedNodes.length : 0
        }};
        state.changes.push(entry);
      }}
      if (state.changes.length > 500) {{
        state.changes = state.changes.slice(-500);
      }}
    }});
    observer.observe(root, {{ childList: true, subtree: true, attributes: true, characterData: true }});
    state.observer = observer;
    window[key] = state;
  }}
  const state = window[key];
  const since = {since};
  const max = {max};
  let changes = state.changes.filter((c) => c.id > since);
  if (max > 0 && changes.length > max) {{
    changes = changes.slice(-max);
  }}
  return {{ lastId: state.seq, changes, url: location.href, title: document.title }};
}})();"#,
                since = since,
                max = max
            );
            let result = eval_with_result(app, &script, 10000).await?;
            Ok(serde_json::json!({ "success": true, "result": result }))
        }),
    });

    tools.insert("browser_watch_events".to_string(), Tool {
        name: "browser_watch_events".to_string(),
        description: "Watch user interactions (click/input/submit) without capturing sensitive values".to_string(),
        input_schema: ToolInputSchema {
            type_name: "object".to_string(),
            properties: {
                let mut props = HashMap::new();
                props.insert("since".to_string(), serde_json::json!({
                    "type": "number",
                    "description": "Last seen event id (default 0)"
                }));
                props.insert("max".to_string(), serde_json::json!({
                    "type": "number",
                    "description": "Max number of events to return (default 100)"
                }));
                props
            },
            required: vec![],
        },
        handler: |args, app| Box::pin(async move {
            let since = args
                .and_then(|a| a.get("since")?.as_u64())
                .unwrap_or(0);
            let max = args
                .and_then(|a| a.get("max")?.as_u64())
                .unwrap_or(100);
            let script = format!(
                r#"return (function(){{
  const key = "__MCP_EVENT_WATCH__";
  if (!window[key]) {{
    const state = {{ seq: 0, events: [] }};
    const push = (type, target) => {{
      const tag = target && target.tagName ? target.tagName.toLowerCase() : null;
      const idAttr = target && target.id ? target.id : null;
      const nameAttr = target && target.getAttribute ? target.getAttribute("name") : null;
      const ariaLabel = target && target.getAttribute ? target.getAttribute("aria-label") : null;
      const classList = target && target.className ? String(target.className).split(/\\s+/).slice(0, 4) : [];
      const entry = {{
        id: ++state.seq,
        type,
        tag,
        id: idAttr,
        name: nameAttr,
        classes: classList,
        ariaLabel
      }};
      state.events.push(entry);
      if (state.events.length > 500) {{
        state.events = state.events.slice(-500);
      }}
    }};
    document.addEventListener("click", (e) => push("click", e.target), true);
    document.addEventListener("submit", (e) => push("submit", e.target), true);
    document.addEventListener("change", (e) => push("change", e.target), true);
    document.addEventListener("input", (e) => push("input", e.target), true);
    window[key] = state;
  }}
  const state = window[key];
  const since = {since};
  const max = {max};
  let events = state.events.filter((e) => e.id > since);
  if (max > 0 && events.length > max) {{
    events = events.slice(-max);
  }}
  return {{ lastId: state.seq, events, url: location.href, title: document.title }};
}})();"#,
                since = since,
                max = max
            );
            let result = eval_with_result(app, &script, 10000).await?;
            Ok(serde_json::json!({ "success": true, "result": result }))
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
