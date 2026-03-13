use serde::Deserialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Event, Listener, Manager, WebviewWindow};
use tokio::sync::oneshot;
use tokio::time::{timeout, Duration};

#[derive(Clone)]
pub struct McpEvalStore {
    inner: Arc<McpEvalStoreInner>,
}

struct McpEvalStoreInner {
    next_id: AtomicU64,
    pending: Mutex<HashMap<u64, oneshot::Sender<Result<serde_json::Value, String>>>>,
}

impl McpEvalStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(McpEvalStoreInner {
                next_id: AtomicU64::new(1),
                pending: Mutex::new(HashMap::new()),
            }),
        }
    }

    fn register(&self) -> (u64, oneshot::Receiver<Result<serde_json::Value, String>>) {
        let id = self.inner.next_id.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = oneshot::channel();
        self.inner
            .pending
            .lock()
            .expect("MCP eval store poisoned")
            .insert(id, tx);
        (id, rx)
    }

    fn complete(&self, id: u64, result: Result<serde_json::Value, String>) {
        if let Some(tx) = self
            .inner
            .pending
            .lock()
            .expect("MCP eval store poisoned")
            .remove(&id)
        {
            let _ = tx.send(result);
        }
    }
}

#[derive(Deserialize)]
struct EvalEventPayload {
    id: u64,
    ok: bool,
    value: Option<serde_json::Value>,
    error: Option<String>,
}

pub fn install_eval_listener(app: &AppHandle) {
    let store = app.state::<McpEvalStore>().clone();
    app.listen("mcp-eval-result", move |event: Event| {
        let payload = event.payload();

        let Ok(parsed) = serde_json::from_str::<EvalEventPayload>(payload) else {
            return;
        };

        let result = if parsed.ok {
            Ok(parsed.value.unwrap_or(serde_json::Value::Null))
        } else {
            Err(parsed.error.unwrap_or_else(|| "Unknown error".to_string()))
        };

        store.complete(parsed.id, result);
    });
}

pub async fn eval_with_result(
    app: &AppHandle,
    script: &str,
    timeout_ms: u64,
) -> Result<serde_json::Value, String> {
    let window: WebviewWindow = app.get_webview_window("pake").ok_or("Window not found")?;
    let store = app.state::<McpEvalStore>().clone();
    let (id, rx) = store.register();

    let user_code = serde_json::to_string(script).map_err(|e| e.to_string())?;
    let js = format!(
        r#"(async () => {{
  const userCode = {user_code};
  const run = async () => {{
    const fn = new Function("return (async () => {{ " + userCode + " }})()");
    return await fn();
  }};
  try {{
    const value = await run();
    const payload = {{ id: {id}, ok: true, value: (value === undefined ? null : value) }};
    window.__TAURI__.event.emit("mcp-eval-result", payload);
  }} catch (e) {{
    window.__TAURI__.event.emit("mcp-eval-result", {{ id: {id}, ok: false, error: String(e) }});
  }}
}})();"#,
        user_code = user_code,
        id = id
    );

    window
        .eval(&js)
        .map_err(|e| format!("Eval failed: {}", e))?;

    let result = timeout(Duration::from_millis(timeout_ms), rx)
        .await
        .map_err(|_| "Eval timed out".to_string())?
        .map_err(|_| "Eval response channel closed".to_string())??;

    Ok(result)
}
