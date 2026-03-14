use serde::Serialize;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use tauri::Manager;
use serde_json::Value;
use crate::app::mcp_eval::eval_with_result;

#[derive(Clone)]
pub struct Resource {
    pub uri: String,
    pub name: String,
    pub description: String,
    pub mime_type: String,
    pub handler: ResourceHandler,
}

type ResourceHandler = for<'a> fn(
    &'a tauri::AppHandle,
) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + 'a>>;

impl Serialize for Resource {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("Resource", 4)?;
        s.serialize_field("uri", &self.uri)?;
        s.serialize_field("name", &self.name)?;
        s.serialize_field("description", &self.description)?;
        s.serialize_field("mimeType", &self.mime_type)?;
        s.end()
    }
}

pub fn register_resources() -> HashMap<String, Resource> {
    let mut resources = HashMap::new();

    // 当前 URL
    resources.insert("browser://current-url".to_string(), Resource {
        uri: "browser://current-url".to_string(),
        name: "Current URL".to_string(),
        description: "The current page URL".to_string(),
        mime_type: "text/plain".to_string(),
        handler: |app| Box::pin(async move {
            if let Some(window) = app.get_webview_window("pake") {
                window.url()
                    .map(|u| u.to_string())
                    .map_err(|e| format!("Failed to get URL: {}", e))
            } else {
                Err("Window not found".to_string())
            }
        }),
    });

    // 页面标题
    resources.insert("browser://title".to_string(), Resource {
        uri: "browser://title".to_string(),
        name: "Page Title".to_string(),
        description: "The current page title".to_string(),
        mime_type: "text/plain".to_string(),
        handler: |app| Box::pin(async move {
            if let Some(window) = app.get_webview_window("pake") {
                window.title()
                    .map_err(|e| format!("Failed to get title: {}", e))
            } else {
                Err("Window not found".to_string())
            }
        }),
    });

    // 页面内容
    resources.insert("browser://content".to_string(), Resource {
        uri: "browser://content".to_string(),
        name: "Page Content".to_string(),
        description: "The page HTML content".to_string(),
        mime_type: "text/html".to_string(),
        handler: |app| Box::pin(async move {
            if let Some(window) = app.get_webview_window("pake") {
                let _ = window;
                let script = r#"return (function(){
  const root = document.documentElement;
  if (!root) return null;
  const clone = root.cloneNode(true);
  const inputs = clone.querySelectorAll("input, textarea");
  inputs.forEach((el) => {
    el.setAttribute("value", "");
    if (el.tagName && el.tagName.toLowerCase() === "textarea") {
      el.textContent = "";
    }
  });
  return clone.outerHTML;
})();"#;
                let value = eval_with_result(app, script, 10000)
                    .await
                    .map_err(|e| format!("Failed to get content: {}", e))?;
                match value {
                    Value::String(text) => Ok(text),
                    other => Ok(other.to_string()),
                }
            } else {
                Err("Window not found".to_string())
            }
        }),
    });

    // 页面文本
    resources.insert("browser://text".to_string(), Resource {
        uri: "browser://text".to_string(),
        name: "Page Text".to_string(),
        description: "The visible text content of the page".to_string(),
        mime_type: "text/plain".to_string(),
        handler: |app| Box::pin(async move {
            if let Some(window) = app.get_webview_window("pake") {
                let _ = window;
                let value = eval_with_result(app, "return document.body.innerText;", 10000)
                    .await
                    .map_err(|e| format!("Failed to get text: {}", e))?;
                match value {
                    Value::String(text) => Ok(text),
                    other => Ok(other.to_string()),
                }
            } else {
                Err("Window not found".to_string())
            }
        }),
    });

    // 窗口状态
    resources.insert("browser://window-state".to_string(), Resource {
        uri: "browser://window-state".to_string(),
        name: "Window State".to_string(),
        description: "The current window state (visible, maximized, etc.)".to_string(),
        mime_type: "application/json".to_string(),
        handler: |app| Box::pin(async move {
            if let Some(window) = app.get_webview_window("pake") {
                let is_visible = window.is_visible().unwrap_or(false);
                let is_maximized = window.is_maximized().unwrap_or(false);
                let is_fullscreen = window.is_fullscreen().unwrap_or(false);
                let is_focused = window.is_focused().unwrap_or(false);
                
                Ok(serde_json::to_string(&serde_json::json!({
                    "visible": is_visible,
                    "maximized": is_maximized,
                    "fullscreen": is_fullscreen,
                    "focused": is_focused
                })).unwrap_or_default())
            } else {
                Err("Window not found".to_string())
            }
        }),
    });

    resources
}
