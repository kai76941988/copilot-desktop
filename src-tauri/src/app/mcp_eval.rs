use std::sync::atomic::{AtomicU64, Ordering};
use tauri::{AppHandle, Manager, WebviewWindow};
use tokio::time::{sleep, Duration, Instant};

static NEXT_EVAL_ID: AtomicU64 = AtomicU64::new(1);

pub async fn eval_with_result(
    app: &AppHandle,
    script: &str,
    timeout_ms: u64,
) -> Result<serde_json::Value, String> {
    let window: WebviewWindow = app.get_webview_window("pake").ok_or("Window not found")?;

    let original_title = window.title().unwrap_or_default();
    let id = NEXT_EVAL_ID.fetch_add(1, Ordering::Relaxed);
    let prefix = format!("__MCP_EVAL__{}:", id);
    let prefix_js = serde_json::to_string(&prefix).map_err(|e| e.to_string())?;
    let user_code = serde_json::to_string(script).map_err(|e| e.to_string())?;

    let js = format!(
        r#"(async () => {{
  const prefix = {prefix_js};
  const userCode = {user_code};
  const run = async () => {{
    const fn = new Function("return (async () => {{ " + userCode + " }})()");
    return await fn();
  }};
  try {{
    const value = await run();
    const payload = JSON.stringify({{ ok: true, value: (value === undefined ? null : value) }});
    document.title = prefix + payload;
  }} catch (e) {{
    const payload = JSON.stringify({{ ok: false, error: String(e) }});
    document.title = prefix + payload;
  }}
}})();"#,
        prefix_js = prefix_js,
        user_code = user_code
    );

    window
        .eval(&js)
        .map_err(|e| format!("Eval failed: {}", e))?;

    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        if Instant::now() > deadline {
            return Err("Eval timed out".to_string());
        }

        let title = window.title().unwrap_or_default();
        if let Some(rest) = title.strip_prefix(&prefix) {
            let _ = window.set_title(&original_title);
            let parsed: serde_json::Value =
                serde_json::from_str(rest).map_err(|e| e.to_string())?;
            if parsed
                .get("ok")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                return Ok(parsed.get("value").cloned().unwrap_or(serde_json::Value::Null));
            }
            let msg = parsed
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
            return Err(msg.to_string());
        }

        sleep(Duration::from_millis(50)).await;
    }
}
